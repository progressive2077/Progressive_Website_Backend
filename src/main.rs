use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

mod db;
mod handlers;
mod middleware;
mod models;

use handlers::{
    auth, board_members, content, employees, gallery, hero, products, upload,
};

async fn health_check(pool: web::Data<sqlx::PgPool>) -> HttpResponse {
    match sqlx::query("SELECT 1").execute(pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "ok",
            "database": "connected"
        })),
        Err(_) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "degraded",
            "database": "unreachable"
        })),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid u16 number");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable is missing");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET environment variable is missing");
    let allowed_origins: Vec<String> = env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000,http://localhost:3001".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    log::info!("Connecting to PostgreSQL database...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    log::info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    log::info!("Starting server on 0.0.0.0:{}", port);

    let pool_data = web::Data::new(pool);
    let jwt_secret_data = web::Data::new(jwt_secret);

    let login_governor_conf = GovernorConfigBuilder::default()
        .seconds_per_request(12)
        .burst_size(5)
        .finish()
        .expect("Failed to build rate limiter config");

    HttpServer::new(move || {
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .supports_credentials()
            .max_age(3600);

        for origin in &allowed_origins {
            cors = cors.allowed_origin(origin);
        }

        App::new()
            .app_data(pool_data.clone())
            .app_data(jwt_secret_data.clone())
            .app_data(web::JsonConfig::default().limit(10 * 1024 * 1024))
            .wrap(cors)
            .wrap(Logger::default())
            // Root endpoints for platform health checks
            .route("/", web::get().to(|| async { HttpResponse::Ok().body("Server is operational") }))
            .route("/", web::head().to(|| async { HttpResponse::Ok().finish() }))
            .route("/health", web::get().to(health_check))
            .route("/api/media/{id}", web::get().to(upload::get_media_by_id))
            .service(
                web::scope("/api/auth")
                    .service(
                        web::resource("/login")
                            .wrap(Governor::new(&login_governor_conf))
                            .route(web::post().to(auth::login)),
                    )
                    .route("/logout", web::post().to(auth::logout))
                    .route("/me", web::get().to(auth::me))
                    .route("/refresh", web::post().to(auth::refresh_token)),
            )
            .service(
                web::scope("/api/public")
                    .route("/hero", web::get().to(hero::get_hero))
                    .route("/products", web::get().to(products::list_products_public))
                    .route("/products/{slug}", web::get().to(products::get_product_public))
                    .route("/gallery", web::get().to(gallery::list_gallery_public))
                    .route("/content/{key}", web::get().to(content::get_content))
                    .route("/about", web::get().to(content::get_about))
                    .route("/board_members", web::get().to(board_members::list_public))
                    .route("/employees", web::get().to(employees::get_public_employees))
                    .route("/contact-info", web::get().to(content::get_contact_info)),
            )
            .service(
                web::scope("/api/admin")
                    .wrap(middleware::auth::JwtMiddleware)
                    .route("/upload", web::post().to(upload::upload_image))
                    .route("/hero", web::put().to(hero::update_hero))
                    .route("/products", web::get().to(products::list_products))
                    .route("/products", web::post().to(products::create_product))
                    .route("/products/{id}", web::get().to(products::get_product))
                    .route("/products/{id}", web::put().to(products::update_product))
                    .route("/products/{id}", web::delete().to(products::delete_product))
                    .route("/gallery", web::get().to(gallery::list_gallery))
                    .route("/gallery", web::post().to(gallery::create_gallery_item))
                    .route("/gallery/{id}", web::put().to(gallery::update_gallery_item))
                    .route("/gallery/{id}", web::delete().to(gallery::delete_gallery_item))
                    .route("/content", web::get().to(content::list_content))
                    .route("/content/{key}", web::put().to(content::update_content))
                    .route("/employees", web::get().to(employees::list_employees))
                    .route("/employees", web::post().to(employees::create_employee))
                    .route("/employees/{id}", web::get().to(employees::get_employee))
                    .route("/employees/{id}", web::put().to(employees::update_employee))
                    .route("/employees/{id}", web::delete().to(employees::delete_employee))
                    .route("/employees/{id}/role", web::patch().to(employees::update_role))
                    .route("/employees/{id}/reset-password", web::post().to(employees::reset_password))
                    .route("/stats", web::get().to(handlers::stats::get_stats))
                    .route("/board_members", web::get().to(board_members::list_admin))
                    .route("/board_members", web::post().to(board_members::create))
                    .route("/board_members/{id}", web::put().to(board_members::update))
                    .route("/board_members/{id}", web::delete().to(board_members::delete)),
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
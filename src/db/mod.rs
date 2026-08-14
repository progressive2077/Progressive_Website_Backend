pub mod pool {
    use sqlx::postgres::PgPoolOptions;
    use std::env;

    #[allow(dead_code)]
    pub async fn create_pool() -> sqlx::PgPool {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await
            .expect("Failed to connect to PostgreSQL")
    }
}

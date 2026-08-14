use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    ApiError, ApiResponse, Claims, CreateProductRequest, UpdateProductRequest,
};

pub async fn list_products_public(pool: web::Data<PgPool>) -> HttpResponse {
    let products = sqlx::query_as!(
        crate::models::Product,
        r#"
        SELECT id, name, slug, description, short_description, category,
               image_url, is_published, sort_order, features, created_at, updated_at, created_by
        FROM products
        WHERE is_published = true
        ORDER BY sort_order ASC, created_at DESC
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    match products {
        Ok(products) => HttpResponse::Ok().json(ApiResponse::success(products)),
        Err(e) => {
            log::error!("Error fetching products: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to fetch products"))
        }
    }
}

pub async fn get_product_public(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> HttpResponse {
    let slug = path.into_inner();

    let product = sqlx::query_as!(
        crate::models::Product,
        r#"
        SELECT id, name, slug, description, short_description, category,
               image_url, is_published, sort_order, features, created_at, updated_at, created_by
        FROM products WHERE slug = $1 AND is_published = true
        "#,
        slug
    )
    .fetch_optional(pool.get_ref())
    .await;

    match product {
        Ok(Some(product)) => HttpResponse::Ok().json(ApiResponse::success(product)),
        Ok(None) => HttpResponse::NotFound().json(ApiError::new("Product not found")),
        Err(e) => {
            log::error!("Error fetching product: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to fetch product"))
        }
    }
}

pub async fn list_products(pool: web::Data<PgPool>) -> HttpResponse {
    let products = sqlx::query_as!(
        crate::models::Product,
        r#"
        SELECT id, name, slug, description, short_description, category,
               image_url, is_published, sort_order, features, created_at, updated_at, created_by
        FROM products
        ORDER BY sort_order ASC, created_at DESC
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    match products {
        Ok(products) => HttpResponse::Ok().json(ApiResponse::success(products)),
        Err(e) => {
            log::error!("Error fetching products: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to fetch products"))
        }
    }
}

pub async fn get_product(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let id = path.into_inner();

    let product = sqlx::query_as!(
        crate::models::Product,
        r#"
        SELECT id, name, slug, description, short_description, category,
               image_url, is_published, sort_order, features, created_at, updated_at, created_by
        FROM products WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match product {
        Ok(Some(product)) => HttpResponse::Ok().json(ApiResponse::success(product)),
        Ok(None) => HttpResponse::NotFound().json(ApiError::new("Product not found")),
        Err(e) => {
            log::error!("Error fetching product: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to fetch product"))
        }
    }
}

pub async fn create_product(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CreateProductRequest>,
) -> HttpResponse {
    let user_id = {
        let ext = req.extensions();
        ext.get::<Claims>()
            .and_then(|c| Uuid::parse_str(&c.sub).ok())
    };

    let user_id = match user_id {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiError::new("Unauthorized")),
    };

    let features = body
        .features
        .clone()
        .unwrap_or_else(|| serde_json::Value::Array(vec![]));

    let product = sqlx::query_as!(
        crate::models::Product,
        r#"
        INSERT INTO products (name, slug, description, short_description, category,
                              image_url, is_published, sort_order, features, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, name, slug, description, short_description, category,
                  image_url, is_published, sort_order, features, created_at, updated_at, created_by
        "#,
        body.name,
        body.slug,
        body.description,
        body.short_description,
        body.category,
        body.image_url,
        body.is_published.unwrap_or(false),
        body.sort_order.unwrap_or(0),
        features,
        user_id
    )
    .fetch_one(pool.get_ref())
    .await;

    match product {
        Ok(product) => HttpResponse::Created()
            .json(ApiResponse::success_message(product, "Product created successfully")),
        Err(e) => {
            log::error!("Error creating product: {}", e);
            if e.to_string().contains("unique") {
                HttpResponse::Conflict().json(ApiError::new("Product slug already exists"))
            } else {
                HttpResponse::InternalServerError().json(ApiError::new("Failed to create product"))
            }
        }
    }
}

pub async fn update_product(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateProductRequest>,
) -> HttpResponse {
    let id = path.into_inner();

    let existing = sqlx::query!("SELECT id FROM products WHERE id = $1", id)
        .fetch_optional(pool.get_ref())
        .await;

    if let Ok(None) = existing {
        return HttpResponse::NotFound().json(ApiError::new("Product not found"));
    }

    let product = sqlx::query_as!(
        crate::models::Product,
        r#"
        UPDATE products SET
            name = COALESCE($1, name),
            slug = COALESCE($2, slug),
            description = COALESCE($3, description),
            short_description = COALESCE($4, short_description),
            category = COALESCE($5, category),
            image_url = COALESCE($6, image_url),
            is_published = COALESCE($7, is_published),
            sort_order = COALESCE($8, sort_order),
            features = COALESCE($9, features),
            updated_at = NOW()
        WHERE id = $10
        RETURNING id, name, slug, description, short_description, category,
                  image_url, is_published, sort_order, features, created_at, updated_at, created_by
        "#,
        body.name,
        body.slug,
        body.description,
        body.short_description,
        body.category,
        body.image_url,
        body.is_published,
        body.sort_order,
        body.features,
        id
    )
    .fetch_one(pool.get_ref())
    .await;

    match product {
        Ok(product) => HttpResponse::Ok()
            .json(ApiResponse::success_message(product, "Product updated successfully")),
        Err(e) => {
            log::error!("Error updating product: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to update product"))
        }
    }
}

pub async fn delete_product(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let id = path.into_inner();

    let result = sqlx::query!("DELETE FROM products WHERE id = $1 RETURNING id", id)
        .fetch_optional(pool.get_ref())
        .await;

    match result {
        Ok(Some(_)) => HttpResponse::Ok()
            .json(ApiResponse::success(serde_json::json!({"message": "Product deleted"}))),
        Ok(None) => HttpResponse::NotFound().json(ApiError::new("Product not found")),
        Err(e) => {
            log::error!("Error deleting product: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to delete product"))
        }
    }
}

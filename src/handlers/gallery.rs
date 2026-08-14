use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    ApiError, ApiResponse, Claims, CreateGalleryItemRequest, UpdateGalleryItemRequest,
};

pub async fn list_gallery_public(pool: web::Data<PgPool>) -> HttpResponse {
    let items = sqlx::query_as!(
        crate::models::GalleryItem,
        r#"
        SELECT id, title, description, image_url, category, sort_order, is_published, created_at, updated_at, created_by
        FROM gallery_items
        WHERE is_published = true
        ORDER BY sort_order ASC, created_at DESC
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    match items {
        Ok(items) => HttpResponse::Ok().json(ApiResponse::success(items)),
        Err(e) => {
            log::error!("Error fetching gallery: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to fetch gallery"))
        }
    }
}

pub async fn list_gallery(pool: web::Data<PgPool>) -> HttpResponse {
    let items = sqlx::query_as!(
        crate::models::GalleryItem,
        r#"
        SELECT id, title, description, image_url, category, sort_order, is_published, created_at, updated_at, created_by
        FROM gallery_items
        ORDER BY sort_order ASC, created_at DESC
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    match items {
        Ok(items) => HttpResponse::Ok().json(ApiResponse::success(items)),
        Err(e) => {
            log::error!("Error fetching gallery: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to fetch gallery"))
        }
    }
}

pub async fn create_gallery_item(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CreateGalleryItemRequest>,
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

    let item = sqlx::query_as!(
        crate::models::GalleryItem,
        r#"
        INSERT INTO gallery_items (title, description, image_url, category, sort_order, is_published, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, title, description, image_url, category, sort_order, is_published, created_at, updated_at, created_by
        "#,
        body.title,
        body.description,
        body.image_url,
        body.category,
        body.sort_order.unwrap_or(0),
        body.is_published.unwrap_or(true),
        user_id
    )
    .fetch_one(pool.get_ref())
    .await;

    match item {
        Ok(item) => HttpResponse::Created()
            .json(ApiResponse::success_message(item, "Gallery item created")),
        Err(e) => {
            log::error!("Error creating gallery item: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to create gallery item"))
        }
    }
}

pub async fn update_gallery_item(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateGalleryItemRequest>,
) -> HttpResponse {
    let id = path.into_inner();

    let item = sqlx::query_as!(
        crate::models::GalleryItem,
        r#"
        UPDATE gallery_items SET
            title = COALESCE($1, title),
            description = COALESCE($2, description),
            image_url = COALESCE($3, image_url),
            category = COALESCE($4, category),
            sort_order = COALESCE($5, sort_order),
            is_published = COALESCE($6, is_published),
            updated_at = NOW()
        WHERE id = $7
        RETURNING id, title, description, image_url, category, sort_order, is_published, created_at, updated_at, created_by
        "#,
        body.title,
        body.description,
        body.image_url,
        body.category,
        body.sort_order,
        body.is_published,
        id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match item {
        Ok(Some(item)) => HttpResponse::Ok()
            .json(ApiResponse::success_message(item, "Gallery item updated")),
        Ok(None) => HttpResponse::NotFound().json(ApiError::new("Gallery item not found")),
        Err(e) => {
            log::error!("Error updating gallery item: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to update gallery item"))
        }
    }
}

pub async fn delete_gallery_item(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let id = path.into_inner();

    let result = sqlx::query!(
        "DELETE FROM gallery_items WHERE id = $1 RETURNING id",
        id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(_)) => HttpResponse::Ok()
            .json(ApiResponse::success(serde_json::json!({"message": "Gallery item deleted"}))),
        Ok(None) => HttpResponse::NotFound().json(ApiError::new("Gallery item not found")),
        Err(e) => {
            log::error!("Error deleting gallery item: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to delete gallery item"))
        }
    }
}

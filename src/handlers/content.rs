use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{ApiError, ApiResponse, Claims, UpdateContentRequest};

pub async fn get_content(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> HttpResponse {
    let key = path.into_inner();

    let content = sqlx::query_as!(
        crate::models::ContentBlock,
        r#"SELECT id, key, title, content, content_type, metadata, is_published, updated_at, updated_by
           FROM content_blocks WHERE key = $1 AND is_published = true"#,
        key
    )
    .fetch_optional(pool.get_ref())
    .await;

    match content {
        Ok(Some(content)) => HttpResponse::Ok().json(ApiResponse::success(content)),
        Ok(None) => HttpResponse::NotFound().json(ApiError::new("Content not found")),
        Err(e) => {
            log::error!("Error fetching content: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to fetch content"))
        }
    }
}

pub async fn get_about(pool: web::Data<PgPool>) -> HttpResponse {
    let content = sqlx::query_as!(
        crate::models::ContentBlock,
        r#"SELECT id, key, title, content, content_type, metadata, is_published, updated_at, updated_by
           FROM content_blocks WHERE key LIKE 'about%' AND is_published = true ORDER BY key"#
    )
    .fetch_all(pool.get_ref())
    .await;

    match content {
        Ok(content) => HttpResponse::Ok().json(ApiResponse::success(content)),
        Err(e) => {
            log::error!("Error fetching about content: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to fetch about content"))
        }
    }
}

pub async fn get_contact_info(pool: web::Data<PgPool>) -> HttpResponse {
    let content = sqlx::query_as!(
        crate::models::ContentBlock,
        r#"SELECT id, key, title, content, content_type, metadata, is_published, updated_at, updated_by
           FROM content_blocks WHERE key = 'contact_info' AND is_published = true"#
    )
    .fetch_optional(pool.get_ref())
    .await;

    match content {
        Ok(Some(content)) => HttpResponse::Ok().json(ApiResponse::success(content)),
        Ok(None) => HttpResponse::NotFound().json(ApiError::new("Contact info not found")),
        Err(e) => {
            log::error!("Error fetching contact info: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to fetch contact info"))
        }
    }
}

pub async fn list_content(pool: web::Data<PgPool>) -> HttpResponse {
    let content = sqlx::query_as!(
        crate::models::ContentBlock,
        r#"SELECT id, key, title, content, content_type, metadata, is_published, updated_at, updated_by
           FROM content_blocks ORDER BY key"#
    )
    .fetch_all(pool.get_ref())
    .await;

    match content {
        Ok(content) => HttpResponse::Ok().json(ApiResponse::success(content)),
        Err(e) => {
            log::error!("Error fetching content: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to fetch content"))
        }
    }
}

pub async fn update_content(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    body: web::Json<UpdateContentRequest>,
) -> HttpResponse {
    let key = path.into_inner();

    let user_id = {
        let ext = req.extensions();
        ext.get::<Claims>()
            .and_then(|c| Uuid::parse_str(&c.sub).ok())
    };

    let content = sqlx::query_as!(
        crate::models::ContentBlock,
        r#"
        UPDATE content_blocks SET
            title = COALESCE($1, title),
            content = COALESCE($2, content),
            metadata = COALESCE($3, metadata),
            is_published = COALESCE($4, is_published),
            updated_at = NOW(),
            updated_by = $5
        WHERE key = $6
        RETURNING id, key, title, content, content_type, metadata, is_published, updated_at, updated_by
        "#,
        body.title,
        body.content,
        body.metadata,
        body.is_published,
        user_id,
        key
    )
    .fetch_optional(pool.get_ref())
    .await;

    match content {
        Ok(Some(content)) => HttpResponse::Ok()
            .json(ApiResponse::success_message(content, "Content updated successfully")),
        Ok(None) => HttpResponse::NotFound().json(ApiError::new("Content block not found")),
        Err(e) => {
            log::error!("Error updating content: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to update content"))
        }
    }
}

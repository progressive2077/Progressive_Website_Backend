use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, Responder};
use futures::TryStreamExt;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::Media;

pub async fn upload_image(
    pool: web::Data<PgPool>,
    public_api_url: web::Data<String>,
    mut payload: Multipart,
) -> impl Responder {
    let mut file_data = Vec::new();
    let mut file_name = String::from("upload.bin");
    let mut file_type = String::from("application/octet-stream");

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();

        if let Some(cd) = content_disposition {
            if let Some(name) = cd.get_filename() {
                file_name = name.to_string();
            }
        }

        if let Some(mime) = field.content_type() {
            file_type = mime.to_string();
        }

        while let Ok(Some(chunk)) = field.try_next().await {
            file_data.extend_from_slice(&chunk);
        }
    }

    if file_data.is_empty() {
        return HttpResponse::BadRequest().json("No file uploaded or file empty");
    }

    let media_id = Uuid::new_v4();
    let full_image_url = format!("{}/api/media/{}", public_api_url.get_ref(), media_id);

    let result = sqlx::query!(
        r#"
        INSERT INTO media (id, file_name, file_type, file_data)
        VALUES ($1, $2, $3, $4)
        "#,
        media_id,
        file_name,
        file_type,
        file_data
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "id": media_id,
            "url": full_image_url
        })),
        Err(e) => {
            log::error!("Database insert error: {:?}", e);
            HttpResponse::InternalServerError().json("Failed to store image")
        }
    }
}

pub async fn get_media_by_id(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
) -> impl Responder {
    let media_id = id.into_inner();

    let record = sqlx::query_as!(
        Media,
        r#"
        SELECT 
            id, 
            file_name as "file_name!", 
            file_type as "file_type!", 
            file_data as "file_data!", 
            CONCAT('/api/media/', id::text) as "file_url!",
            created_at as "created_at!"
        FROM media 
        WHERE id = $1
        "#,
        media_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match record {
        Ok(Some(media)) => HttpResponse::Ok()
            .content_type(media.file_type)
            .append_header(("Cache-Control", "public, max-age=31536000, immutable"))
            .body(media.file_data),
        Ok(None) => HttpResponse::NotFound().json("Media not found"),
        Err(e) => {
            log::error!("Database query error: {:?}", e);
            HttpResponse::InternalServerError().json("Database error")
        }
    }
}
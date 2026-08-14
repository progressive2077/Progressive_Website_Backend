use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::models::{ApiError, ApiResponse, UpdateHeroRequest};

pub async fn get_hero(pool: web::Data<PgPool>) -> HttpResponse {
    let hero = sqlx::query_as!(
        crate::models::HeroSection,
        r#"SELECT id, heading, subheading, description, primary_cta_text, primary_cta_link,
                  secondary_cta_text, secondary_cta_link, background_image_url, is_active, updated_at
           FROM hero_sections WHERE is_active = true LIMIT 1"#
    )
    .fetch_optional(pool.get_ref())
    .await;

    match hero {
        Ok(Some(hero)) => HttpResponse::Ok().json(ApiResponse::success(hero)),
        Ok(None) => HttpResponse::NotFound().json(ApiError::new("Hero section not found")),
        Err(e) => {
            log::error!("Error fetching hero: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to fetch hero section"))
        }
    }
}

pub async fn update_hero(
    pool: web::Data<PgPool>,
    body: web::Json<UpdateHeroRequest>,
) -> HttpResponse {
    let hero = sqlx::query_as!(
        crate::models::HeroSection,
        r#"
        UPDATE hero_sections SET
            heading = COALESCE($1, heading),
            subheading = COALESCE($2, subheading),
            description = COALESCE($3, description),
            primary_cta_text = COALESCE($4, primary_cta_text),
            primary_cta_link = COALESCE($5, primary_cta_link),
            secondary_cta_text = COALESCE($6, secondary_cta_text),
            secondary_cta_link = COALESCE($7, secondary_cta_link),
            background_image_url = COALESCE($8, background_image_url),
            is_active = COALESCE($9, is_active),
            updated_at = NOW()
        WHERE is_active = true
        RETURNING id, heading, subheading, description, primary_cta_text, primary_cta_link,
                  secondary_cta_text, secondary_cta_link, background_image_url, is_active, updated_at
        "#,
        body.heading,
        body.subheading,
        body.description,
        body.primary_cta_text,
        body.primary_cta_link,
        body.secondary_cta_text,
        body.secondary_cta_link,
        body.background_image_url,
        body.is_active
    )
    .fetch_optional(pool.get_ref())
    .await;

    match hero {
        Ok(Some(hero)) => HttpResponse::Ok()
            .json(ApiResponse::success_message(hero, "Hero section updated")),
        Ok(None) => HttpResponse::NotFound().json(ApiError::new("Hero section not found")),
        Err(e) => {
            log::error!("Error updating hero: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to update hero section"))
        }
    }
}

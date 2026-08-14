use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    ApiError, ApiResponse, BoardMember, Claims, 
    CreateBoardMemberRequest, UpdateBoardMemberRequest,
};

fn is_super_admin(claims: &Claims) -> bool {
    claims.role == "superadmin"
}

pub async fn list_public(pool: web::Data<PgPool> ) -> HttpResponse {
    let members = sqlx::query_as!(
        BoardMember,
        r#"SELECT id, full_name, title, bio, image_url, sort_order, is_published,
         created_at, updated_at, created_by FROM board_members WHERE is_published = true
         ORDER BY sort_order ASC, created_at ASC"#
    )
    .fetch_all(pool.get_ref())
    .await;

    match members {
        Ok(m) => HttpResponse::Ok().json(ApiResponse::success(m)),
        Err(e) => {
            log::error!("Error fetching board members: {}", e);
            HttpResponse::InternalServerError()
            .json(ApiError::new("Failed to fetch board members"))
        }
    }
}

pub async fn list_admin(pool: web::Data<PgPool>) -> HttpResponse {
    let members = sqlx::query_as!(
        BoardMember,
        r#"SELECT id, full_name, title, bio, image_url, sort_order,
                  is_published, created_at, updated_at, created_by
           FROM board_members
           ORDER BY sort_order ASC, created_at ASC"#
    )
    .fetch_all(pool.get_ref())
    .await;

    match members {
        Ok(m) => HttpResponse::Ok().json(ApiResponse::success(m)),
        Err(e) => {
            log::error!("Error fetching board members: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to fetch board members"))
        }
    }
}

pub async fn create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CreateBoardMemberRequest>,
) -> HttpResponse {
    let (is_super, sub) = {
        let ext = req.extensions();
        match ext.get::<Claims>(){
            Some(c) => (is_super_admin(c), c.sub.clone()),
            None => return HttpResponse::Unauthorized().json(ApiError::new("Unauthorized")),
        }
    };

    if !is_super {
        return HttpResponse::Forbidden()
            .json(ApiError::new("Only super admin can manage board members"));
    }

    let user_id = match Uuid::parse_str(&sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().json(ApiError::new("Invalid token")),
    };

    let member = sqlx::query_as!(
        BoardMember,
        r#"INSERT INTO board_members (full_name, title, bio, image_url, sort_order, is_published,
        created_by) VALUES ( $1, $2, $3, $4, $5, $6, $7) RETURNING id, full_name, title, bio,
        image_url, sort_order, is_published, created_at, updated_at, created_by"#,
        body.full_name,
        body.title,
        body.bio,
        body.image_url,
        body.sort_order.unwrap_or(0),
        body.is_published.unwrap_or(true),
        user_id,
    )
    .fetch_one(pool.get_ref())
    .await;

    match member {
        Ok(m) => HttpResponse::Created().json(ApiResponse::success_message(
            m, "Board member created"
        )),
        Err(e) => {
            log::error!("Error creating board member: {}", e);
            HttpResponse::InternalServerError()
            .json(ApiError::new("Failed to create board member"))

    }
    }
}

pub async fn update(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateBoardMemberRequest>,
) -> HttpResponse {
    let is_super = {
        let ext = req.extensions();
        match ext.get::<Claims>() {
            Some(c) => is_super_admin(c),
            None => return HttpResponse::Unauthorized().json(ApiError::new("Unauthorized")),
        }
    };

    if !is_super {
        return HttpResponse::Forbidden().json(ApiError::new("Only super admins can manage board members"));
    }

    let id = path.into_inner();

    let member = sqlx::query_as!(
        BoardMember,
        r#"UPDATE board_members SET 
                full_name = COALESCE($1, full_name),
                title = COALESCE($2, title),
                bio = COALESCE($3, bio),
                image_url = COALESCE($4, image_url),
                sort_order = COALESCE($5::int, sort_order),
                is_published = COALESCE($6::bool, is_published),
                updated_at = NOW() WHERE id = $7 
                RETURNING id, full_name, title, bio, image_url,
                sort_order, is_published, created_at, updated_at,
                created_by"#,
                body.full_name,
                body.title,
                body.bio,
                body.image_url,
                body.sort_order,
                body.is_published,
                id,
    )
    .fetch_optional(pool.get_ref())
    .await;

    match member {
        Ok(Some(m)) => HttpResponse::Ok()
            .json(ApiResponse::success_message(m, "Board member updated")),
        Ok(None) => HttpResponse::NotFound().json(ApiError::new("Board member not found")),
        Err(e) => {
            log::error!("Error updating board member: {}", e);
            HttpResponse::InternalServerError()
            .json(ApiError::new("Failed to upload board member"))
        }
    }   
}

pub async fn delete (
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let is_super ={
        let ext = req.extensions();
        match ext.get::<Claims>(){
            Some(c) => is_super_admin(c),
            None => return HttpResponse::Unauthorized().json(ApiError::new("Unauthorized")),
        }
    };

    if !is_super {
        return HttpResponse::Forbidden()
         .json(ApiError::new("Only super admin can manage board members"));
    }

    let id = path.into_inner();

    let result = sqlx::query!(
        "DELETE FROM board_members WHERE id = $1 RETURNING id", id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(_)) => HttpResponse::Ok()
            .json(ApiResponse::success(serde_json::json!({"message":"Board member deleted"}))),
            Ok(None) => HttpResponse::NotFound().json(ApiError::new("Board member not found")),
            Err(e) => {
                log::error!("Error deleting board member: {}", e);
                HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to delete board member"))
            }
    }
}


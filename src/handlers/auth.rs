use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    ApiError, ApiResponse, Claims, LoginRequest, LoginResponse, UserPublic,
};

pub async fn login(
    pool: web::Data<PgPool>,
    jwt_secret: web::Data<String>,
    body: web::Json<LoginRequest>,
) -> HttpResponse {
    let user = sqlx::query_as!(
        crate::models::User,
        r#"
        SELECT id, email, password_hash, full_name, title, role as "role: crate::models::UserRole",
               is_active, avatar_url, permissions, created_at, updated_at, last_login
        FROM users WHERE email = $1 AND is_active = true
        "#,
        body.email.to_lowercase()
    )
    .fetch_optional(pool.get_ref())
    .await;

    match user {
        Ok(Some(user)) => {
            match verify(&body.password, &user.password_hash) {
                Ok(true) => {
                    let _ = sqlx::query!(
                        "UPDATE users SET last_login = NOW() WHERE id = $1",
                        user.id
                    )
                    .execute(pool.get_ref())
                    .await;

                    let now = chrono::Utc::now();
                    let exp = (now + chrono::Duration::hours(24)).timestamp() as usize;
                    let iat = now.timestamp() as usize;

                    let claims = Claims {
                        sub: user.id.to_string(),
                        role: user.role.to_string(),
                        exp,
                        iat,
                    };

                    match encode(
                        &Header::default(),
                        &claims,
                        &EncodingKey::from_secret(jwt_secret.as_bytes()),
                    ) {
                        Ok(token) => {
                            let user_public: UserPublic = user.into();
                            HttpResponse::Ok().json(ApiResponse::success(LoginResponse {
                                token,
                                user: user_public,
                            }))
                        }
                        Err(_) => HttpResponse::InternalServerError()
                            .json(ApiError::new("Failed to generate token")),
                    }
                }
                _ => HttpResponse::Unauthorized().json(ApiError::new("Invalid credentials")),
            }
        }
        Ok(None) => HttpResponse::Unauthorized().json(ApiError::new("Invalid credentials")),
        Err(e) => {
            log::error!("DB error during login: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Internal server error"))
        }
    }
}

pub async fn logout(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse::success("Logged out successfully"))
}

pub async fn me(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let sub = {
        let ext = req.extensions();
        match ext.get::<Claims>() {
            Some(c) => c.sub.clone(),
            None => return HttpResponse::Unauthorized().json(ApiError::new("Unauthorized")),
        }
    };

    let user_id = match Uuid::parse_str(&sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().json(ApiError::new("Invalid token")),
    };

    let user = sqlx::query_as!(
        crate::models::User,
        r#"
        SELECT id, email, password_hash, full_name, title, role as "role: crate::models::UserRole",
               is_active, avatar_url, permissions, created_at, updated_at, last_login
        FROM users WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match user {
        Ok(Some(user)) => {
            let user_public: UserPublic = user.into();
            HttpResponse::Ok().json(ApiResponse::success(user_public))
        }
        Ok(None) => HttpResponse::NotFound().json(ApiError::new("User not found")),
        Err(_) => {
            HttpResponse::InternalServerError().json(ApiError::new("Internal server error"))
        }
    }
}

pub async fn refresh_token(
    req: HttpRequest,
    jwt_secret: web::Data<String>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let sub = {
        let ext = req.extensions();
        match ext.get::<Claims>() {
            Some(c) => c.sub.clone(),
            None => return HttpResponse::Unauthorized().json(ApiError::new("Unauthorized")),
        }
    };

    let user_id = match Uuid::parse_str(&sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().json(ApiError::new("Invalid token")),
    };

    let user = sqlx::query_as!(
        crate::models::User,
        r#"
        SELECT id, email, password_hash, full_name, title, role as "role: crate::models::UserRole",
               is_active, avatar_url, permissions, created_at, updated_at, last_login
        FROM users WHERE id = $1 AND is_active = true
        "#,
        user_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match user {
        Ok(Some(user)) => {
            let now = chrono::Utc::now();
            let exp = (now + chrono::Duration::hours(24)).timestamp() as usize;
            let iat = now.timestamp() as usize;

            let new_claims = Claims {
                sub: user.id.to_string(),
                role: user.role.to_string(),
                exp,
                iat,
            };

            match encode(
                &Header::default(),
                &new_claims,
                &EncodingKey::from_secret(jwt_secret.as_bytes()),
            ) {
                Ok(token) => {
                    HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
                        "token": token
                    })))
                }
                Err(_) => HttpResponse::InternalServerError()
                    .json(ApiError::new("Failed to generate token")),
            }
        }
        Ok(None) => HttpResponse::Unauthorized().json(ApiError::new("User not found")),
        Err(_) => {
            HttpResponse::InternalServerError().json(ApiError::new("Internal server error"))
        }
    }
}

#[allow(dead_code)]
pub async fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

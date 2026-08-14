use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use bcrypt::{hash, DEFAULT_COST};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    ApiError, ApiResponse, Claims, CreateEmployeeRequest, ResetPasswordRequest,
    UpdateEmployeeRequest, UpdateRoleRequest, UserRole,
};

fn has_admin_access(claims: &Claims) -> bool {
    matches!(claims.role.as_str(), "superadmin" | "admin")
}

fn is_super_admin(claims: &Claims) -> bool {
    claims.role == "superadmin"
}

/// Reads the JWT claims out of the request extensions and immediately copies
/// out the bits we need as owned values. This avoids holding a non-`Send`
/// `Ref<Extensions>` guard across an `.await` point later in the handler.
fn caller_access(req: &HttpRequest) -> Option<(bool, bool, String)> {
    let ext = req.extensions();
    let claims = ext.get::<Claims>()?;
    Some((is_super_admin(claims), has_admin_access(claims), claims.sub.clone()))
}

pub async fn list_employees(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let Some((_is_super, has_access, _sub)) = caller_access(&req) else {
        return HttpResponse::Unauthorized().json(ApiError::new("Unauthorized"));
    };

    if !has_access {
        return HttpResponse::Forbidden().json(ApiError::new("Insufficient permissions"));
    }

    let employees = sqlx::query_as!(
        crate::models::User,
        r#"
        SELECT id, email, password_hash, full_name, title, role as "role: crate::models::UserRole",
               is_active, avatar_url, permissions, created_at, updated_at, last_login
        FROM users ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    match employees {
        Ok(employees) => {
            let public: Vec<crate::models::UserPublic> =
                employees.into_iter().map(|u| u.into()).collect();
            HttpResponse::Ok().json(ApiResponse::success(public))
        }
        Err(e) => {
            log::error!("Error fetching employees: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to fetch employees"))
        }
    }
}

pub async fn get_employee(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let Some((_is_super, has_access, _sub)) = caller_access(&req) else {
        return HttpResponse::Unauthorized().json(ApiError::new("Unauthorized"));
    };

    if !has_access {
        return HttpResponse::Forbidden().json(ApiError::new("Insufficient permissions"));
    }

    let id = path.into_inner();

    let employee = sqlx::query_as!(
        crate::models::User,
        r#"
        SELECT id, email, password_hash, full_name, title, role as "role: crate::models::UserRole",
               is_active, avatar_url, permissions, created_at, updated_at, last_login
        FROM users WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match employee {
        Ok(Some(employee)) => {
            let public: crate::models::UserPublic = employee.into();
            HttpResponse::Ok().json(ApiResponse::success(public))
        }
        Ok(None) => HttpResponse::NotFound().json(ApiError::new("Employee not found")),
        Err(e) => {
            log::error!("Error fetching employee: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to fetch employee"))
        }
    }
}

pub async fn get_public_employees(pool: web::Data<PgPool>) -> HttpResponse {
    let employees = sqlx::query_as!(
        crate::models::User,
        r#"
        SELECT id, email, password_hash, full_name, title, role as "role: crate::models::UserRole",
               is_active, avatar_url, permissions, created_at, updated_at, last_login
        FROM users WHERE is_active = true ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    match employees {
        Ok(employees) => {
            let public: Vec<crate::models::UserPublic> =
                employees.into_iter().map(|u| u.into()).collect();
            HttpResponse::Ok().json(ApiResponse::success(public))
        }
        Err(e) => {
            log::error!("Error fetching public employees: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to fetch employees"))
        }
    }
}

pub async fn create_employee(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<CreateEmployeeRequest>,
) -> HttpResponse {
    let Some((is_super, has_access, _sub)) = caller_access(&req) else {
        return HttpResponse::Unauthorized().json(ApiError::new("Unauthorized"));
    };

    if (body.role == UserRole::SuperAdmin || body.role == UserRole::Admin) && !is_super {
        return HttpResponse::Forbidden()
            .json(ApiError::new("Only super admin can create admin accounts"));
    }

    if !has_access {
        return HttpResponse::Forbidden().json(ApiError::new("Insufficient permissions"));
    }

    let password_hash = match hash(&body.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to hash password"))
        }
    };

    let permissions = body
        .permissions
        .clone()
        .unwrap_or_else(|| serde_json::json!({}));

    let employee = sqlx::query_as!(
        crate::models::User,
        r#"
        INSERT INTO users (email, password_hash, full_name, title, role, permissions)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, email, password_hash, full_name, title, role as "role: crate::models::UserRole",
                  is_active, avatar_url, permissions, created_at, updated_at, last_login
        "#,
        body.email.to_lowercase(),
        password_hash,
        body.full_name,
        body.title,
        body.role.clone() as UserRole,
        permissions
    )
    .fetch_one(pool.get_ref())
    .await;

    match employee {
        Ok(employee) => {
            let public: crate::models::UserPublic = employee.into();
            HttpResponse::Created()
                .json(ApiResponse::success_message(public, "Employee created successfully"))
        }
        Err(e) => {
            log::error!("Error creating employee: {}", e);
            if e.to_string().contains("unique") {
                HttpResponse::Conflict().json(ApiError::new("Email already exists"))
            } else {
                HttpResponse::InternalServerError()
                    .json(ApiError::new("Failed to create employee"))
            }
        }
    }
}

pub async fn update_employee(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateEmployeeRequest>,
) -> HttpResponse {
    let Some((_is_super, has_access, _sub)) = caller_access(&req) else {
        return HttpResponse::Unauthorized().json(ApiError::new("Unauthorized"));
    };

    if !has_access {
        return HttpResponse::Forbidden().json(ApiError::new("Insufficient permissions"));
    }

    let id = path.into_inner();

    let employee = sqlx::query_as!(
        crate::models::User,
        r#"
        UPDATE users SET
            email = COALESCE($1, email),
            full_name = COALESCE($2, full_name),
            title = COALESCE($3, title),
            is_active = COALESCE($4, is_active),
            avatar_url = COALESCE($5, avatar_url),
            updated_at = NOW()
        WHERE id = $6
        RETURNING id, email, password_hash, full_name, title, role as "role: crate::models::UserRole",
                  is_active, avatar_url, permissions, created_at, updated_at, last_login
        "#,
        body.email.as_deref().map(|e| e.to_lowercase()),
        body.full_name,
        body.title,
        body.is_active,
        body.avatar_url,
        id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match employee {
        Ok(Some(employee)) => {
            let public: crate::models::UserPublic = employee.into();
            HttpResponse::Ok().json(ApiResponse::success_message(public, "Employee updated"))
        }
        Ok(None) => HttpResponse::NotFound().json(ApiError::new("Employee not found")),
        Err(e) => {
            log::error!("Error updating employee: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to update employee"))
        }
    }
}

pub async fn delete_employee(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let Some((is_super, _has_access, sub)) = caller_access(&req) else {
        return HttpResponse::Unauthorized().json(ApiError::new("Unauthorized"));
    };

    if !is_super {
        return HttpResponse::Forbidden()
            .json(ApiError::new("Only super admin can delete accounts"));
    }

    let id = path.into_inner();

    if let Ok(self_id) = Uuid::parse_str(&sub) {
        if self_id == id {
            return HttpResponse::BadRequest()
                .json(ApiError::new("Cannot delete your own account"));
        }
    }

    let result = sqlx::query!("DELETE FROM users WHERE id = $1 RETURNING id", id)
        .fetch_optional(pool.get_ref())
        .await;

    match result {
        Ok(Some(_)) => HttpResponse::Ok()
            .json(ApiResponse::success(serde_json::json!({"message": "Employee deleted"}))),
        Ok(None) => HttpResponse::NotFound().json(ApiError::new("Employee not found")),
        Err(e) => {
            log::error!("Error deleting employee: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to delete employee"))
        }
    }
}

pub async fn update_role(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateRoleRequest>,
) -> HttpResponse {
    let Some((is_super, _has_access, _sub)) = caller_access(&req) else {
        return HttpResponse::Unauthorized().json(ApiError::new("Unauthorized"));
    };

    if !is_super {
        return HttpResponse::Forbidden()
            .json(ApiError::new("Only super admin can modify roles"));
    }

    let id = path.into_inner();
    let permissions = body
        .permissions
        .clone()
        .unwrap_or_else(|| serde_json::json!({}));

    let employee = sqlx::query_as!(
        crate::models::User,
        r#"
        UPDATE users SET role = $1, permissions = $2, updated_at = NOW()
        WHERE id = $3
        RETURNING id, email, password_hash, full_name, title, role as "role: crate::models::UserRole",
                  is_active, avatar_url, permissions, created_at, updated_at, last_login
        "#,
        body.role.clone() as UserRole,
        permissions,
        id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match employee {
        Ok(Some(employee)) => {
            let public: crate::models::UserPublic = employee.into();
            HttpResponse::Ok().json(ApiResponse::success_message(public, "Role updated"))
        }
        Ok(None) => HttpResponse::NotFound().json(ApiError::new("Employee not found")),
        Err(e) => {
            log::error!("Error updating role: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to update role"))
        }
    }
}

pub async fn reset_password(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<ResetPasswordRequest>,
) -> HttpResponse {
    let Some((_is_super, has_access, _sub)) = caller_access(&req) else {
        return HttpResponse::Unauthorized().json(ApiError::new("Unauthorized"));
    };

    if !has_access {
        return HttpResponse::Forbidden().json(ApiError::new("Insufficient permissions"));
    }

    let id = path.into_inner();

    let password_hash = match hash(&body.new_password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to hash password"))
        }
    };

    let result = sqlx::query!(
        "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2 RETURNING id",
        password_hash,
        id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(_)) => HttpResponse::Ok()
            .json(ApiResponse::success(serde_json::json!({"message": "Password reset successfully"}))),
        Ok(None) => HttpResponse::NotFound().json(ApiError::new("Employee not found")),
        Err(e) => {
            log::error!("Error resetting password: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to reset password"))
        }
    }
}
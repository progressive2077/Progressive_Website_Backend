use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    SuperAdmin,
    Admin,
    Editor,
    Viewer,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::SuperAdmin => write!(f, "superadmin"),
            UserRole::Admin => write!(f, "admin"),
            UserRole::Editor => write!(f, "editor"),
            UserRole::Viewer => write!(f, "viewer"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub full_name: String,
    pub title: Option<String>,
    pub role: UserRole,
    pub is_active: bool,
    pub avatar_url: Option<String>,
    pub permissions: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub title: Option<String>,
    pub role: UserRole,
    pub is_active: bool,
    pub avatar_url: Option<String>,
    pub permissions: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

impl From<User> for UserPublic {
    fn from(u: User) -> Self {
        UserPublic {
            id: u.id,
            email: u.email,
            full_name: u.full_name,
            title: u.title,
            role: u.role,
            is_active: u.is_active,
            avatar_url: u.avatar_url,
            permissions: u.permissions,
            created_at: u.created_at,
            last_login: u.last_login,
        }
    }
}

//___BoardMember____
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct BoardMember {
    pub id: Uuid,
    pub full_name: String,
    pub title: String,
    pub bio: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: i32,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}


#[derive(Debug, Deserialize)]
pub struct CreateBoardMemberRequest {
    pub full_name: String,
    pub title: String,
    pub bio: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: Option<i32>,
    pub is_published: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBoardMemberRequest {
    pub full_name: Option<String>,
    pub title: Option<String>,
    pub bio: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: Option<i32>,
    pub is_published: Option<bool>,
}


#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub short_description: Option<String>,
    pub category: String,
    pub image_url: Option<String>,
    pub is_published: bool,
    pub sort_order: i32,
    pub features: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct GalleryItem {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub image_url: String,
    pub category: Option<String>,
    pub sort_order: i32,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ContentBlock {
    pub id: Uuid,
    pub key: String,
    pub title: String,
    pub content: String,
    pub content_type: String,
    pub metadata: serde_json::Value,
    pub is_published: bool,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct HeroSection {
    pub id: Uuid,
    pub heading: String,
    pub subheading: String,
    pub description: String,
    pub primary_cta_text: String,
    pub primary_cta_link: String,
    pub secondary_cta_text: String,
    pub secondary_cta_link: String,
    pub background_image_url: Option<String>,
    pub is_active: bool,
    pub updated_at: DateTime<Utc>,
}


#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Media {
    pub id: Uuid,
    pub file_name: String,
    pub file_type: String,
    #[serde(skip_serializing)]
    pub file_data: Vec<u8>,
    pub file_url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub id: Uuid,
    pub url: String,
}

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserPublic,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmployeeRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub title: Option<String>,
    pub role: UserRole,
    pub permissions: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmployeeRequest {
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub title: Option<String>,
    pub is_active: Option<bool>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub role: UserRole,
    pub permissions: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub slug: String,
    pub description: String,
    pub short_description: Option<String>,
    pub category: String,
    pub image_url: Option<String>,
    pub is_published: Option<bool>,
    pub sort_order: Option<i32>,
    pub features: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub category: Option<String>,
    pub image_url: Option<String>,
    pub is_published: Option<bool>,
    pub sort_order: Option<i32>,
    pub features: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGalleryItemRequest {
    pub title: String,
    pub description: Option<String>,
    pub image_url: String,
    pub category: Option<String>,
    pub sort_order: Option<i32>,
    pub is_published: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGalleryItemRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub category: Option<String>,
    pub sort_order: Option<i32>,
    pub is_published: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContentRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub is_published: Option<bool>,
}

#[derive(serde::Deserialize)]
pub struct UpdateHeroRequest {
    pub heading: Option<String>,
    pub subheading: Option<String>,
    pub description: Option<String>,
    pub primary_cta_text: Option<String>,
    pub primary_cta_link: Option<String>,
    pub secondary_cta_text: Option<String>,
    pub secondary_cta_link: Option<String>,
    pub background_image_url: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            message: None,
            error: None,
        }
    }

    pub fn success_message(data: T, message: &str) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            message: Some(message.to_string()),
            error: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub success: bool,
    pub error: String,
    pub message: Option<String>,
}

impl ApiError {
    pub fn new(error: &str) -> Self {
        ApiError {
            success: false,
            error: error.to_string(),
            message: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_products: i64,
    pub published_products: i64,
    pub total_gallery_items: i64,
    pub total_employees: i64,
    pub active_employees: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

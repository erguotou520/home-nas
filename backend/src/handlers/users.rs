use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::middleware::auth::Claims;
use crate::services::user_service;

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub role: String,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub role: Option<String>,
    pub password: Option<String>,
}

pub async fn list_users(
    db: web::Data<DatabaseConnection>,
    claims: Claims,
) -> HttpResponse {
    if !claims.is_admin() {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Admin access required"
        }));
    }
    
    match user_service::list_users(&db).await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn update_user(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    claims: Claims,
    body: web::Json<UpdateUserRequest>,
) -> HttpResponse {
    if !claims.is_admin() {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Admin access required"
        }));
    }
    
    let user_id = path.into_inner();
    
    match user_service::update_user(&db, user_id, body.role.as_deref(), body.password.as_deref()).await {
        Ok(user) => HttpResponse::Ok().json(UserResponse {
            id: user.id,
            username: user.username,
            role: format!("{:?}", user.role).to_lowercase(),
            created_at: user.created_at.to_rfc3339(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn delete_user(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    claims: Claims,
) -> HttpResponse {
    if !claims.is_admin() {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Admin access required"
        }));
    }
    
    let user_id = path.into_inner();
    
    // Prevent self-deletion
    if user_id == claims.sub {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Cannot delete yourself"
        }));
    }
    
    match user_service::delete_user(&db, user_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

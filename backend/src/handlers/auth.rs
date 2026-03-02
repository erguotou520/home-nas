use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::middleware::auth::Claims;
use crate::services::auth_service;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Serialize)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub role: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub role: Option<String>,
}

pub async fn login(
    db: web::Data<DatabaseConnection>,
    jwt_secret: web::Data<String>,
    body: web::Json<LoginRequest>,
) -> HttpResponse {
    match auth_service::login(&db, &body.username, &body.password, &jwt_secret).await {
        Ok((token, user)) => HttpResponse::Ok().json(LoginResponse {
            token,
            user: UserInfo {
                id: user.id,
                username: user.username,
                role: format!("{:?}", user.role).to_lowercase(),
            },
        }),
        Err(e) => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn register(
    db: web::Data<DatabaseConnection>,
    claims: Claims,
    body: web::Json<RegisterRequest>,
) -> HttpResponse {
    // Only admins can register new users
    if !claims.is_admin() {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only admins can create users"
        }));
    }

    let role = body.role.as_deref().unwrap_or("user");
    match auth_service::register(&db, &body.username, &body.password, role).await {
        Ok(user) => HttpResponse::Created().json(UserInfo {
            id: user.id,
            username: user.username,
            role: format!("{:?}", user.role).to_lowercase(),
        }),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn me(claims: Claims) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "id": claims.sub,
        "username": claims.username,
        "role": claims.role,
    }))
}

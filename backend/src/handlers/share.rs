use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::middleware::auth::Claims;
use crate::services::share_service;

#[derive(Deserialize)]
pub struct CreateShareRequest {
    pub file_path: String,
    pub app_type: String,
    pub expires_in_hours: Option<i64>,
    pub burn_after_read: Option<bool>,
    pub max_views: Option<i32>,
}

#[derive(Serialize)]
pub struct ShareResponse {
    pub id: i32,
    pub token: String,
    pub url: String,
    pub expires_at: Option<String>,
    pub burn_after_read: bool,
}

pub async fn create_share(
    db: web::Data<DatabaseConnection>,
    claims: Claims,
    web_url: web::Data<String>,
    body: web::Json<CreateShareRequest>,
) -> HttpResponse {
    match share_service::create_share(
        &db,
        claims.sub,
        &body.file_path,
        &body.app_type,
        body.expires_in_hours,
        body.burn_after_read.unwrap_or(false),
        body.max_views,
    ).await {
        Ok(share) => {
            let url = format!("{}/s/{}", web_url.as_str(), share.token);
            HttpResponse::Created().json(ShareResponse {
                id: share.id,
                token: share.token,
                url,
                expires_at: share.expires_at.map(|d| d.to_rfc3339()),
                burn_after_read: share.burn_after_read,
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn list_shares(
    db: web::Data<DatabaseConnection>,
    claims: Claims,
) -> HttpResponse {
    match share_service::list_user_shares(&db, claims.sub).await {
        Ok(shares) => HttpResponse::Ok().json(shares),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn delete_share(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    claims: Claims,
) -> HttpResponse {
    let share_id = path.into_inner();
    
    match share_service::delete_share(&db, share_id, claims.sub).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn access_share(
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
) -> HttpResponse {
    let token = path.into_inner();
    
    match share_service::access_share(&db, &token).await {
        Ok(file_info) => HttpResponse::Ok().json(file_info),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

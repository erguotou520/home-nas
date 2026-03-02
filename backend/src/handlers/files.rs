use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::Config;
use crate::middleware::auth::Claims;
use crate::services::file_service;

#[derive(Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<i64>,
    pub mime_type: Option<String>,
    pub thumbnail: Option<String>,
    pub metadata: Option<MediaMetadata>,
}

#[derive(Serialize)]
pub struct MediaMetadata {
    pub title: Option<String>,
    pub poster: Option<String>,
    pub fanart: Option<String>,
    pub year: Option<String>,
    pub plot: Option<String>,
}

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Serialize)]
pub struct ListResponse {
    pub entries: Vec<FileEntry>,
    pub total: usize,
    pub path: String,
    pub app: String,
}

pub async fn list_files(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    query: web::Query<ListQuery>,
    _claims: Claims,
) -> HttpResponse {
    let (app, rel_path) = path.into_inner();
    
    // Find the base path for this app
    let base_path = match get_app_base_path(&config, &app) {
        Some(p) => p,
        None => return HttpResponse::NotFound().json(serde_json::json!({
            "error": "App not found"
        })),
    };
    
    let full_path = if rel_path.is_empty() || rel_path == "/" {
        base_path.clone()
    } else {
        format!("{}/{}", base_path, rel_path.trim_start_matches('/'))
    };
    
    match file_service::list_directory(&full_path, &app).await {
        Ok(entries) => HttpResponse::Ok().json(ListResponse {
            total: entries.len(),
            entries,
            path: rel_path,
            app,
        }),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn list_app_root(
    config: web::Data<Config>,
    path: web::Path<String>,
    query: web::Query<ListQuery>,
    claims: Claims,
) -> HttpResponse {
    let app = path.into_inner();
    list_files(
        config,
        web::Path::from((app, String::new())),
        query,
        claims,
    ).await
}

#[derive(Deserialize)]
pub struct FileOperation {
    pub operation: String, // "copy" or "move"
    pub destination: String,
}

pub async fn operate_file(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    body: web::Json<FileOperation>,
    _claims: Claims,
) -> HttpResponse {
    let (app, rel_path) = path.into_inner();
    
    let base_path = match get_app_base_path(&config, &app) {
        Some(p) => p,
        None => return HttpResponse::NotFound().json(serde_json::json!({
            "error": "App not found"
        })),
    };
    
    let src_path = format!("{}/{}", base_path, rel_path.trim_start_matches('/'));
    let dest_path = format!("{}/{}", base_path, body.destination.trim_start_matches('/'));
    
    let result = match body.operation.as_str() {
        "copy" => file_service::copy_file(&src_path, &dest_path).await,
        "move" => file_service::move_file(&src_path, &dest_path).await,
        _ => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid operation"
        })),
    };
    
    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn delete_file(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    _claims: Claims,
) -> HttpResponse {
    let (app, rel_path) = path.into_inner();
    
    let base_path = match get_app_base_path(&config, &app) {
        Some(p) => p,
        None => return HttpResponse::NotFound().json(serde_json::json!({
            "error": "App not found"
        })),
    };
    
    let full_path = format!("{}/{}", base_path, rel_path.trim_start_matches('/'));
    
    match file_service::delete_file(&full_path).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

fn get_app_base_path(config: &Config, app: &str) -> Option<String> {
    let paths = match app {
        "medias" => &config.apps.medias,
        "documents" => &config.apps.documents,
        "videos" => &config.apps.videos,
        "music" => &config.apps.music,
        _ => return None,
    };
    
    // For now, return the first path's base directory
    paths.first().and_then(|p| {
        p.split(':').last().map(|s| s.to_string())
    })
}

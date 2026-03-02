use actix_files::NamedFile;
use actix_web::{web, HttpRequest, HttpResponse};
use std::path::PathBuf;

use crate::config::Config;
use crate::middleware::auth::Claims;
use crate::services::{media_service, thumbnail_service};

pub async fn stream_media(
    config: web::Data<Config>,
    req: HttpRequest,
    path: web::Path<String>,
    _claims: Claims,
) -> actix_web::Result<HttpResponse> {
    let file_path = path.into_inner();
    
    // Validate and resolve path
    let full_path = PathBuf::from(&file_path);
    if !full_path.exists() {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "File not found"
        })));
    }
    
    // Use actix-files for streaming with range support
    let file = NamedFile::open(&full_path)?;
    Ok(file.into_response(&req))
}

pub async fn get_thumbnail(
    path: web::Path<String>,
    _claims: Claims,
) -> HttpResponse {
    let file_path = path.into_inner();
    
    match thumbnail_service::get_or_create_thumbnail(&file_path).await {
        Ok(thumb_path) => {
            match NamedFile::open(&thumb_path) {
                Ok(file) => {
                    HttpResponse::Ok()
                        .content_type("image/jpeg")
                        .body(std::fs::read(&thumb_path).unwrap_or_default())
                }
                Err(_) => HttpResponse::NotFound().finish(),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn get_lyrics(
    path: web::Path<String>,
    _claims: Claims,
) -> HttpResponse {
    let file_path = path.into_inner();
    
    match media_service::get_lyrics(&file_path).await {
        Ok(lyrics) => HttpResponse::Ok().json(serde_json::json!({
            "lyrics": lyrics
        })),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn get_media_info(
    path: web::Path<String>,
    _claims: Claims,
) -> HttpResponse {
    let file_path = path.into_inner();
    
    match media_service::get_media_info(&file_path).await {
        Ok(info) => HttpResponse::Ok().json(info),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

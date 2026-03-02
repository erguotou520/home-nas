mod config;
mod handlers;
mod middleware;
mod models;
mod services;
mod utils;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, middleware as actix_middleware};
use sea_orm::{Database, DatabaseConnection};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "../config.yaml".into());
    let config = config::Config::load(&config_path)
        .expect("Failed to load config");
    
    tracing::info!("Loaded configuration from {}", config_path);
    
    // Connect to database
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| config.database_url());
    
    let db: DatabaseConnection = Database::connect(&db_url)
        .await
        .expect("Failed to connect to database");
    
    tracing::info!("Connected to database");
    
    // Extract values for app data
    let jwt_secret = config.global.jwt_secret.clone();
    let web_url = config.global.web_url.clone();
    
    // Start server
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into());
    tracing::info!("Starting server on {}", bind_addr);
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        App::new()
            .wrap(cors)
            .wrap(actix_middleware::Logger::default())
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(jwt_secret.clone()))
            .app_data(web::Data::new(web_url.clone()))
            .configure(configure_routes)
    })
    .bind(&bind_addr)?
    .run()
    .await
}

fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // Auth routes (public)
        .route("/api/auth/login", web::post().to(handlers::auth::login))
        
        // Auth routes (protected, admin only for register)
        .route("/api/auth/register", web::post().to(handlers::auth::register))
        .route("/api/auth/me", web::get().to(handlers::auth::me))
        
        // File routes
        .route("/api/files/{app}", web::get().to(handlers::files::list_app_root))
        .route("/api/files/{app}/{path:.*}", web::get().to(handlers::files::list_files))
        .route("/api/files/{app}/{path:.*}", web::patch().to(handlers::files::operate_file))
        .route("/api/files/{app}/{path:.*}", web::delete().to(handlers::files::delete_file))
        
        // Media routes
        .route("/api/media/stream/{path:.*}", web::get().to(handlers::media::stream_media))
        .route("/api/media/thumbnail/{path:.*}", web::get().to(handlers::media::get_thumbnail))
        .route("/api/media/lyrics/{path:.*}", web::get().to(handlers::media::get_lyrics))
        .route("/api/media/info/{path:.*}", web::get().to(handlers::media::get_media_info))
        
        // Share routes
        .route("/api/shares", web::post().to(handlers::share::create_share))
        .route("/api/shares", web::get().to(handlers::share::list_shares))
        .route("/api/shares/{id}", web::delete().to(handlers::share::delete_share))
        .route("/s/{token}", web::get().to(handlers::share::access_share))
        
        // User management (admin only)
        .route("/api/users", web::get().to(handlers::users::list_users))
        .route("/api/users/{id}", web::patch().to(handlers::users::update_user))
        .route("/api/users/{id}", web::delete().to(handlers::users::delete_user));
}

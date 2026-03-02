mod config;

use std::{path::PathBuf, sync::Arc};

use axum::{
    extract::{FromRequestParts, Path, Query, State},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, PgPool};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    config: config::Config,
    jwt_secret: Arc<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    sub: i32,
    username: String,
    role: String,
    exp: usize,
}

impl Claims {
    fn is_admin(&self) -> bool {
        self.role == "admin"
    }
}

impl<S> FromRequestParts<S> for Claims
where
    Arc<String>: axum::extract::FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let secret = Arc::<String>::from_ref(state);
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error":"Missing or invalid authorization header"})),
            ))?;

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map(|d| d.claims)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error":"Invalid token"})),
            )
        })
    }
}

impl axum::extract::FromRef<AppState> for Arc<String> {
    fn from_ref(state: &AppState) -> Self {
        state.jwt_secret.clone()
    }
}

#[derive(Serialize, FromRow)]
struct UserRow {
    id: i32,
    username: String,
    password_hash: String,
    role: String,
    created_at: DateTime<Utc>,
}

#[derive(Serialize, FromRow)]
struct ShareRow {
    id: i32,
    user_id: i32,
    file_path: String,
    app_type: String,
    token: String,
    expires_at: Option<DateTime<Utc>>,
    burn_after_read: bool,
    max_views: Option<i32>,
    view_count: i32,
    created_at: DateTime<Utc>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "../config.yaml".into());
    let config = config::Config::load(&config_path)?;
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| config.database_url());
    let pool = PgPool::connect(&db_url).await?;
    init_schema(&pool).await?;

    let state = AppState {
        pool,
        jwt_secret: Arc::new(config.global.jwt_secret.clone()),
        config,
    };

    let app = Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/auth/register", post(register))
        .route("/api/auth/me", get(me))
        .route("/api/users", get(list_users))
        .route("/api/users/:id", patch(update_user).delete(delete_user))
        .route("/api/shares", post(create_share).get(list_shares))
        .route("/api/shares/:id", delete(delete_share))
        .route("/s/:token", get(access_share))
        .route("/api/files/:app", get(list_app_root))
        .route(
            "/api/files/:app/*path",
            get(list_files).patch(operate_file).delete(delete_file),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("listening on {bind_addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn init_schema(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            username VARCHAR(64) UNIQUE NOT NULL,
            password_hash VARCHAR(128) NOT NULL,
            role VARCHAR(16) NOT NULL DEFAULT 'user',
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
        )"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS shares (
            id SERIAL PRIMARY KEY,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            file_path TEXT NOT NULL,
            app_type VARCHAR(32) NOT NULL,
            token VARCHAR(32) UNIQUE NOT NULL,
            expires_at TIMESTAMPTZ NULL,
            burn_after_read BOOLEAN NOT NULL DEFAULT FALSE,
            max_views INTEGER NULL,
            view_count INTEGER NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL
        )"#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}
#[derive(Serialize)]
struct UserInfo {
    id: i32,
    username: String,
    role: String,
}
#[derive(Serialize)]
struct LoginResponse {
    token: String,
    user: UserInfo,
}

async fn login(State(state): State<AppState>, Json(body): Json<LoginRequest>) -> impl IntoResponse {
    let user = match sqlx::query_as::<_, UserRow>(
        "SELECT id, username, password_hash, role, created_at FROM users WHERE username = $1",
    )
    .bind(&body.username)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(user)) => user,
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error":"Invalid credentials"})),
            )
                .into_response()
        }
    };

    if !bcrypt::verify(&body.password, &user.password_hash).unwrap_or(false) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error":"Invalid credentials"})),
        )
            .into_response();
    }

    let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;
    let claims = Claims {
        sub: user.id,
        username: user.username.clone(),
        role: user.role.clone(),
        exp,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .unwrap();

    (
        StatusCode::OK,
        Json(json!(LoginResponse {
            token,
            user: UserInfo {
                id: user.id,
                username: user.username,
                role: user.role
            }
        })),
    )
        .into_response()
}

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
    role: Option<String>,
}
async fn register(
    State(state): State<AppState>,
    claims: Claims,
    Json(body): Json<RegisterRequest>,
) -> impl IntoResponse {
    if !claims.is_admin() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error":"Only admins can create users"})),
        )
            .into_response();
    }
    let hash = match bcrypt::hash(&body.password, bcrypt::DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error":"Invalid password"})),
            )
                .into_response()
        }
    };
    let role = body.role.unwrap_or_else(|| "user".to_string());
    let now = Utc::now();

    match sqlx::query_as::<_, UserRow>("INSERT INTO users (username, password_hash, role, created_at, updated_at) VALUES ($1,$2,$3,$4,$4) RETURNING id, username, password_hash, role, created_at")
        .bind(&body.username).bind(&hash).bind(&role).bind(now).fetch_one(&state.pool).await {
            Ok(user) => (StatusCode::CREATED, Json(json!(UserInfo { id: user.id, username: user.username, role: user.role }))).into_response(),
            Err(_) => (StatusCode::BAD_REQUEST, Json(json!({"error":"Username already exists"}))).into_response(),
        }
}

async fn me(claims: Claims) -> impl IntoResponse {
    (StatusCode::OK, Json(json!(claims)))
}

async fn list_users(State(state): State<AppState>, claims: Claims) -> impl IntoResponse {
    if !claims.is_admin() {
        return (StatusCode::FORBIDDEN, Json(json!({"error":"Admin only"}))).into_response();
    }
    let rows: Vec<UserRow> = match sqlx::query_as(
        "SELECT id, username, password_hash, role, created_at FROM users ORDER BY id",
    )
    .fetch_all(&state.pool)
    .await
    {
        Ok(r) => r,
        Err(_) => vec![],
    };
    let users: Vec<_> = rows.into_iter().map(|u| json!({"id":u.id,"username":u.username,"role":u.role,"created_at":u.created_at.to_rfc3339()})).collect();
    (StatusCode::OK, Json(json!(users))).into_response()
}

#[derive(Deserialize)]
struct UpdateUserRequest {
    role: Option<String>,
    password: Option<String>,
}
async fn update_user(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<i32>,
    Json(body): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    if !claims.is_admin() {
        return (StatusCode::FORBIDDEN, Json(json!({"error":"Admin only"}))).into_response();
    }
    if let Some(role) = body.role {
        let _ = sqlx::query("UPDATE users SET role = $1, updated_at = $2 WHERE id = $3")
            .bind(role)
            .bind(Utc::now())
            .bind(id)
            .execute(&state.pool)
            .await;
    }
    if let Some(password) = body.password {
        if let Ok(hash) = bcrypt::hash(password, bcrypt::DEFAULT_COST) {
            let _ =
                sqlx::query("UPDATE users SET password_hash = $1, updated_at = $2 WHERE id = $3")
                    .bind(hash)
                    .bind(Utc::now())
                    .bind(id)
                    .execute(&state.pool)
                    .await;
        }
    }
    (StatusCode::OK, Json(json!({"success": true}))).into_response()
}

async fn delete_user(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    if !claims.is_admin() {
        return (StatusCode::FORBIDDEN, Json(json!({"error":"Admin only"}))).into_response();
    }
    let _ = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await;
    (StatusCode::OK, Json(json!({"success": true}))).into_response()
}

#[derive(Deserialize)]
struct CreateShareRequest {
    file_path: String,
    app_type: String,
    expires_in_hours: Option<i64>,
    burn_after_read: Option<bool>,
    max_views: Option<i32>,
}

async fn create_share(
    State(state): State<AppState>,
    claims: Claims,
    Json(body): Json<CreateShareRequest>,
) -> impl IntoResponse {
    let token = Uuid::new_v4().to_string().replace('-', "")[..16].to_string();
    let now = Utc::now();
    let expires_at = body.expires_in_hours.map(|h| now + Duration::hours(h));
    match sqlx::query_as::<_, ShareRow>("INSERT INTO shares (user_id,file_path,app_type,token,expires_at,burn_after_read,max_views,view_count,created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,0,$8) RETURNING id,user_id,file_path,app_type,token,expires_at,burn_after_read,max_views,view_count,created_at")
        .bind(claims.sub).bind(&body.file_path).bind(&body.app_type).bind(&token).bind(expires_at).bind(body.burn_after_read.unwrap_or(false)).bind(body.max_views).bind(now).fetch_one(&state.pool).await {
            Ok(share) => (StatusCode::CREATED, Json(json!(share))).into_response(),
            Err(_) => (StatusCode::BAD_REQUEST, Json(json!({"error":"Failed to create share"}))).into_response(),
        }
}

async fn list_shares(State(state): State<AppState>, claims: Claims) -> impl IntoResponse {
    let shares: Vec<ShareRow> = sqlx::query_as("SELECT id,user_id,file_path,app_type,token,expires_at,burn_after_read,max_views,view_count,created_at FROM shares WHERE user_id = $1 ORDER BY id DESC")
        .bind(claims.sub).fetch_all(&state.pool).await.unwrap_or_default();
    (StatusCode::OK, Json(json!(shares))).into_response()
}

async fn delete_share(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let _ = sqlx::query("DELETE FROM shares WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(claims.sub)
        .execute(&state.pool)
        .await;
    (StatusCode::OK, Json(json!({"success": true}))).into_response()
}

async fn access_share(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    let share = sqlx::query_as::<_, ShareRow>("SELECT id,user_id,file_path,app_type,token,expires_at,burn_after_read,max_views,view_count,created_at FROM shares WHERE token = $1")
        .bind(&token).fetch_optional(&state.pool).await.unwrap_or(None);
    let Some(share) = share else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error":"Share not found"})),
        )
            .into_response();
    };

    if share.expires_at.is_some_and(|t| t < Utc::now())
        || share.max_views.is_some_and(|m| share.view_count >= m)
    {
        return (StatusCode::GONE, Json(json!({"error":"Share expired"}))).into_response();
    }

    let _ = sqlx::query("UPDATE shares SET view_count = view_count + 1 WHERE id = $1")
        .bind(share.id)
        .execute(&state.pool)
        .await;
    (
        StatusCode::OK,
        Json(json!({"file_path": share.file_path, "app_type": share.app_type, "is_valid": true})),
    )
        .into_response()
}

#[derive(Deserialize)]
struct ListQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn list_app_root(
    State(state): State<AppState>,
    Path(app): Path<String>,
    query: Query<ListQuery>,
    claims: Claims,
) -> impl IntoResponse {
    list_files(State(state), Path((app, String::new())), query, claims).await
}

async fn list_files(
    State(state): State<AppState>,
    Path((app, path)): Path<(String, String)>,
    Query(_query): Query<ListQuery>,
    _claims: Claims,
) -> impl IntoResponse {
    let Some(base) = get_app_base_path(&state.config, &app) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error":"App not found"})),
        )
            .into_response();
    };
    let full = if path.is_empty() {
        PathBuf::from(base)
    } else {
        PathBuf::from(base).join(path.trim_start_matches('/'))
    };
    let mut entries = vec![];
    if let Ok(mut dir) = tokio::fs::read_dir(&full).await {
        while let Ok(Some(item)) = dir.next_entry().await {
            let meta = item.metadata().await.ok();
            entries.push(json!({
                "name": item.file_name().to_string_lossy().to_string(),
                "path": item.path().to_string_lossy().to_string(),
                "is_dir": meta.as_ref().is_some_and(|m| m.is_dir()),
                "size": meta.as_ref().map(|m| m.len())
            }));
        }
    }
    (
        StatusCode::OK,
        Json(json!({"entries": entries, "total": entries.len(), "path": path, "app": app})),
    )
        .into_response()
}

#[derive(Deserialize)]
struct FileOperation {
    operation: String,
    destination: String,
}

async fn operate_file(
    State(state): State<AppState>,
    Path((app, path)): Path<(String, String)>,
    _claims: Claims,
    Json(body): Json<FileOperation>,
) -> impl IntoResponse {
    let Some(base) = get_app_base_path(&state.config, &app) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error":"App not found"})),
        )
            .into_response();
    };
    let src = PathBuf::from(&base).join(path.trim_start_matches('/'));
    let dst = PathBuf::from(base).join(body.destination.trim_start_matches('/'));
    let result = match body.operation.as_str() {
        "copy" => tokio::fs::copy(src, dst).await.map(|_| ()),
        "move" => tokio::fs::rename(src, dst).await,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error":"Invalid operation"})),
            )
                .into_response()
        }
    };
    match result {
        Ok(_) => (StatusCode::OK, Json(json!({"success":true}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn delete_file(
    State(state): State<AppState>,
    Path((app, path)): Path<(String, String)>,
    _claims: Claims,
) -> impl IntoResponse {
    let Some(base) = get_app_base_path(&state.config, &app) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error":"App not found"})),
        )
            .into_response();
    };
    let target = PathBuf::from(base).join(path.trim_start_matches('/'));
    let result = match tokio::fs::metadata(&target).await {
        Ok(m) if m.is_dir() => tokio::fs::remove_dir_all(target).await,
        _ => tokio::fs::remove_file(target).await,
    };
    match result {
        Ok(_) => (StatusCode::OK, Json(json!({"success":true}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

fn get_app_base_path(config: &config::Config, app: &str) -> Option<String> {
    let paths = match app {
        "medias" => &config.apps.medias,
        "documents" => &config.apps.documents,
        "videos" => &config.apps.videos,
        "music" => &config.apps.music,
        _ => return None,
    };

    paths
        .first()
        .and_then(|p| p.split(':').next_back().map(str::to_string))
}

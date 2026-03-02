mod config;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::{
    body::{Body, Bytes},
    extract::{FromRef, FromRequestParts, Path as AxumPath, Query, State},
    http::{header, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use mime_guess::MimeGuess;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, PgPool};
use tokio::fs;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    config: config::Config,
    jwt_secret: Arc<String>,
}

impl FromRef<AppState> for Arc<String> {
    fn from_ref(state: &AppState) -> Self {
        state.jwt_secret.clone()
    }
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
    Arc<String>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let secret = Arc::<String>::from_ref(state);
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or_else(|| ApiError::unauthorized("Missing or invalid authorization header"))?;

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map(|d| d.claims)
        .map_err(|_| ApiError::unauthorized("Invalid token"))
    }
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    fn unauthorized(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, msg)
    }

    fn forbidden(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, msg)
    }

    fn bad_request(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, msg)
    }

    fn not_found(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, msg)
    }

    fn internal(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, msg)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({"error": self.message}))).into_response()
    }
}

type ApiResult<T> = Result<T, ApiError>;

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
    sqlx::migrate!("./migrations").run(&pool).await?;

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
        .route("/api/files/:app/upload/*path", post(upload_file))
        .route("/api/media/stream/*path", get(stream_media))
        .route("/api/media/thumbnail/*path", get(media_thumbnail))
        .route("/api/media/lyrics/*path", get(media_lyrics))
        .route("/api/media/info/*path", get(media_info))
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

async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let user = sqlx::query_as::<_, UserRow>(
        "SELECT id, username, password_hash, role, created_at FROM users WHERE username = $1",
    )
    .bind(&body.username)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?
    .ok_or_else(|| ApiError::unauthorized("Invalid credentials"))?;

    let verified = bcrypt::verify(&body.password, &user.password_hash)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    if !verified {
        return Err(ApiError::unauthorized("Invalid credentials"));
    }

    let claims = Claims {
        sub: user.id,
        username: user.username.clone(),
        role: user.role.clone(),
        exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(LoginResponse {
        token,
        user: UserInfo {
            id: user.id,
            username: user.username,
            role: user.role,
        },
    }))
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
) -> ApiResult<(StatusCode, Json<UserInfo>)> {
    if !claims.is_admin() {
        return Err(ApiError::forbidden("Only admins can create users"));
    }

    let hash = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let role = body.role.unwrap_or_else(|| "user".to_string());
    let now = Utc::now();

    let user = sqlx::query_as::<_, UserRow>("INSERT INTO users (username, password_hash, role, created_at, updated_at) VALUES ($1,$2,$3,$4,$4) RETURNING id, username, password_hash, role, created_at")
        .bind(&body.username)
        .bind(hash)
        .bind(role)
        .bind(now)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| ApiError::bad_request("Username already exists"))?;

    Ok((
        StatusCode::CREATED,
        Json(UserInfo {
            id: user.id,
            username: user.username,
            role: user.role,
        }),
    ))
}

async fn me(claims: Claims) -> Json<UserInfo> {
    Json(UserInfo {
        id: claims.sub,
        username: claims.username,
        role: claims.role,
    })
}

async fn list_users(
    State(state): State<AppState>,
    claims: Claims,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    if !claims.is_admin() {
        return Err(ApiError::forbidden("Admin only"));
    }

    let rows: Vec<UserRow> = sqlx::query_as(
        "SELECT id, username, password_hash, role, created_at FROM users ORDER BY id",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?;

    let users = rows
        .into_iter()
        .map(|u| {
            json!({"id":u.id,"username":u.username,"role":u.role,"created_at":u.created_at.to_rfc3339()})
        })
        .collect();

    Ok(Json(users))
}

#[derive(Deserialize)]
struct UpdateUserRequest {
    role: Option<String>,
    password: Option<String>,
}

async fn update_user(
    State(state): State<AppState>,
    claims: Claims,
    AxumPath(id): AxumPath<i32>,
    Json(body): Json<UpdateUserRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if !claims.is_admin() {
        return Err(ApiError::forbidden("Admin only"));
    }

    if let Some(role) = body.role {
        sqlx::query("UPDATE users SET role = $1, updated_at = $2 WHERE id = $3")
            .bind(role)
            .bind(Utc::now())
            .bind(id)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?;
    }

    if let Some(password) = body.password {
        let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = $2 WHERE id = $3")
            .bind(hash)
            .bind(Utc::now())
            .bind(id)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?;
    }

    Ok(Json(json!({"success": true})))
}

async fn delete_user(
    State(state): State<AppState>,
    claims: Claims,
    AxumPath(id): AxumPath<i32>,
) -> ApiResult<Json<serde_json::Value>> {
    if !claims.is_admin() {
        return Err(ApiError::forbidden("Admin only"));
    }

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(json!({"success": true})))
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
) -> ApiResult<(StatusCode, Json<ShareRow>)> {
    let token = Uuid::new_v4().to_string().replace('-', "")[..16].to_string();
    let now = Utc::now();
    let expires_at = body.expires_in_hours.map(|h| now + Duration::hours(h));

    let share = sqlx::query_as::<_, ShareRow>("INSERT INTO shares (user_id,file_path,app_type,token,expires_at,burn_after_read,max_views,view_count,created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,0,$8) RETURNING id,user_id,file_path,app_type,token,expires_at,burn_after_read,max_views,view_count,created_at")
        .bind(claims.sub)
        .bind(body.file_path)
        .bind(body.app_type)
        .bind(token)
        .bind(expires_at)
        .bind(body.burn_after_read.unwrap_or(false))
        .bind(body.max_views)
        .bind(now)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(share)))
}

async fn list_shares(
    State(state): State<AppState>,
    claims: Claims,
) -> ApiResult<Json<Vec<ShareRow>>> {
    let shares = sqlx::query_as("SELECT id,user_id,file_path,app_type,token,expires_at,burn_after_read,max_views,view_count,created_at FROM shares WHERE user_id = $1 ORDER BY id DESC")
        .bind(claims.sub)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(shares))
}

async fn delete_share(
    State(state): State<AppState>,
    claims: Claims,
    AxumPath(id): AxumPath<i32>,
) -> ApiResult<Json<serde_json::Value>> {
    sqlx::query("DELETE FROM shares WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(claims.sub)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(json!({"success": true})))
}

async fn access_share(
    State(state): State<AppState>,
    AxumPath(token): AxumPath<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let share = sqlx::query_as::<_, ShareRow>("SELECT id,user_id,file_path,app_type,token,expires_at,burn_after_read,max_views,view_count,created_at FROM shares WHERE token = $1")
        .bind(&token)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .ok_or_else(|| ApiError::not_found("Share not found"))?;

    if share.expires_at.is_some_and(|t| t < Utc::now())
        || share.max_views.is_some_and(|m| share.view_count >= m)
    {
        return Err(ApiError::new(StatusCode::GONE, "Share expired"));
    }

    sqlx::query("UPDATE shares SET view_count = view_count + 1 WHERE id = $1")
        .bind(share.id)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    if share.burn_after_read {
        sqlx::query("DELETE FROM shares WHERE id = $1")
            .bind(share.id)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?;
    }

    Ok(Json(
        json!({"file_path": share.file_path, "app_type": share.app_type, "is_valid": true}),
    ))
}

#[derive(Deserialize)]
struct ListQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn list_app_root(
    State(state): State<AppState>,
    AxumPath(app): AxumPath<String>,
    query: Query<ListQuery>,
    claims: Claims,
) -> ApiResult<Json<serde_json::Value>> {
    list_files(State(state), AxumPath((app, String::new())), query, claims).await
}

async fn list_files(
    State(state): State<AppState>,
    AxumPath((app, path)): AxumPath<(String, String)>,
    Query(query): Query<ListQuery>,
    _claims: Claims,
) -> ApiResult<Json<serde_json::Value>> {
    let base = get_app_base_path(&state.config, &app)
        .ok_or_else(|| ApiError::not_found("App not found"))?;
    let full_path = resolve_within_base(&base, &path)?;

    let mut entries = vec![];
    let mut dir = fs::read_dir(&full_path)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    while let Some(item) = dir
        .next_entry()
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    {
        let metadata = item.metadata().await.ok();
        let item_path = item.path();
        let rel_item_path = path_join_for_client(&path, &item.file_name().to_string_lossy());
        let mime = if metadata.as_ref().is_some_and(|m| m.is_file()) {
            MimeGuess::from_path(&item_path)
                .first_raw()
                .map(str::to_string)
        } else {
            None
        };

        entries.push(json!({
            "name": item.file_name().to_string_lossy().to_string(),
            "path": rel_item_path,
            "is_dir": metadata.as_ref().is_some_and(|m| m.is_dir()),
            "size": metadata.as_ref().map(|m| m.len()),
            "mime_type": mime,
            "thumbnail": metadata.as_ref().and_then(|m| if m.is_file() { Some(format!("/api/media/thumbnail/{}", urlencoding::encode(&path_join_for_client(&path, &item.file_name().to_string_lossy())))) } else { None })
        }));
    }

    entries.sort_by(|a, b| {
        a.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .cmp(b.get("name").and_then(|v| v.as_str()).unwrap_or_default())
    });

    let total = entries.len();
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(total as u32).max(1) as usize;
    let start = ((page - 1) as usize) * limit;
    let paginated = entries
        .into_iter()
        .skip(start)
        .take(limit)
        .collect::<Vec<_>>();

    Ok(Json(
        json!({"entries": paginated, "total": total, "path": path, "app": app}),
    ))
}

#[derive(Deserialize)]
struct FileOperation {
    operation: String,
    destination: String,
}

async fn operate_file(
    State(state): State<AppState>,
    AxumPath((app, path)): AxumPath<(String, String)>,
    _claims: Claims,
    Json(body): Json<FileOperation>,
) -> ApiResult<Json<serde_json::Value>> {
    let base = get_app_base_path(&state.config, &app)
        .ok_or_else(|| ApiError::not_found("App not found"))?;
    let src = resolve_within_base(&base, &path)?;
    let dest = resolve_within_base(&base, &body.destination)?;

    match body.operation.as_str() {
        "copy" => {
            fs::copy(src, dest)
                .await
                .map_err(|e| ApiError::internal(e.to_string()))?;
        }
        "move" => {
            fs::rename(src, dest)
                .await
                .map_err(|e| ApiError::internal(e.to_string()))?;
        }
        _ => return Err(ApiError::bad_request("Invalid operation")),
    }

    Ok(Json(json!({"success": true})))
}

async fn delete_file(
    State(state): State<AppState>,
    AxumPath((app, path)): AxumPath<(String, String)>,
    _claims: Claims,
) -> ApiResult<Json<serde_json::Value>> {
    let base = get_app_base_path(&state.config, &app)
        .ok_or_else(|| ApiError::not_found("App not found"))?;
    let target = resolve_within_base(&base, &path)?;

    let metadata = fs::metadata(&target)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    if metadata.is_dir() {
        fs::remove_dir_all(target)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?;
    } else {
        fs::remove_file(target)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?;
    }

    Ok(Json(json!({"success": true})))
}

#[derive(Deserialize)]
struct UploadQuery {
    filename: Option<String>,
    overwrite: Option<bool>,
}

async fn upload_file(
    State(state): State<AppState>,
    AxumPath((app, path)): AxumPath<(String, String)>,
    Query(query): Query<UploadQuery>,
    _claims: Claims,
    body: Bytes,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    if body.is_empty() {
        return Err(ApiError::bad_request("Empty upload body"));
    }

    let base = get_app_base_path(&state.config, &app)
        .ok_or_else(|| ApiError::not_found("App not found"))?;
    let dir = resolve_within_base(&base, &path)?;

    let metadata = fs::metadata(&dir)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    if !metadata.is_dir() {
        return Err(ApiError::bad_request("Upload target must be a directory"));
    }

    let filename = query
        .filename
        .as_deref()
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| ApiError::bad_request("Missing filename query parameter"))?;

    validate_filename(filename)?;

    let target = dir.join(filename);
    if fs::try_exists(&target)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        && !query.overwrite.unwrap_or(false)
    {
        return Err(ApiError::bad_request(
            "Target file already exists; set overwrite=true",
        ));
    }

    fs::write(&target, body)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "path": path_join_for_client(&path, filename)
        })),
    ))
}

async fn stream_media(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
    _claims: Claims,
) -> ApiResult<Response> {
    let full = resolve_media_path(&state.config, &path)?;
    let bytes = fs::read(full)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok((
        [(header::CONTENT_TYPE, "application/octet-stream")],
        Body::from(bytes),
    )
        .into_response())
}

async fn media_thumbnail(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
    _claims: Claims,
) -> ApiResult<Response> {
    let full = resolve_media_path(&state.config, &path)?;
    let metadata = fs::metadata(&full)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    if metadata.is_dir() {
        return Err(ApiError::bad_request("Cannot get thumbnail for directory"));
    }

    let mime = MimeGuess::from_path(&full)
        .first_raw()
        .unwrap_or("application/octet-stream");
    let bytes = fs::read(full)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(([(header::CONTENT_TYPE, mime)], Body::from(bytes)).into_response())
}

async fn media_lyrics(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
    _claims: Claims,
) -> ApiResult<Json<serde_json::Value>> {
    let media_path = resolve_media_path(&state.config, &path)?;
    let mut lrc_path = media_path.clone();
    lrc_path.set_extension("lrc");

    if !lrc_path.exists() {
        return Ok(Json(json!({"lyrics": []})));
    }

    let content = fs::read_to_string(lrc_path)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    let lyrics = parse_lrc(&content);
    Ok(Json(json!({"lyrics": lyrics})))
}

async fn media_info(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
    _claims: Claims,
) -> ApiResult<Json<serde_json::Value>> {
    let full = resolve_media_path(&state.config, &path)?;
    let title = full
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();

    let mime = MimeGuess::from_path(&full)
        .first_raw()
        .unwrap_or("application/octet-stream");

    Ok(Json(json!({
        "title": title,
        "artist": null,
        "album": null,
        "year": null,
        "duration_secs": null,
        "cover_art": format!("/api/media/thumbnail/{}", urlencoding::encode(&full.to_string_lossy())),
        "mime_type": mime
    })))
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

fn resolve_media_path(config: &config::Config, path: &str) -> ApiResult<PathBuf> {
    if path.contains("..") {
        return Err(ApiError::bad_request("Path traversal is not allowed"));
    }

    let decoded = urlencoding::decode(path)
        .map_err(|_| ApiError::bad_request("Invalid encoded path"))?
        .to_string();

    let candidate = PathBuf::from(&decoded);
    if candidate.is_absolute() {
        if path_in_allowed_roots(config, &candidate) {
            return Ok(candidate);
        }
        return Err(ApiError::forbidden("Path is outside configured roots"));
    }

    for root in all_app_roots(config) {
        let full = PathBuf::from(&root).join(decoded.trim_start_matches('/'));
        if full.exists() {
            return Ok(full);
        }
    }

    Err(ApiError::not_found("Media file not found"))
}

fn path_in_allowed_roots(config: &config::Config, path: &Path) -> bool {
    all_app_roots(config)
        .iter()
        .map(PathBuf::from)
        .any(|root| path.starts_with(root))
}

fn all_app_roots(config: &config::Config) -> Vec<String> {
    [
        &config.apps.medias,
        &config.apps.videos,
        &config.apps.music,
        &config.apps.documents,
    ]
    .into_iter()
    .flat_map(|paths| paths.iter())
    .filter_map(|p| p.split(':').next_back().map(str::to_string))
    .collect()
}

fn parse_lrc(content: &str) -> Vec<serde_json::Value> {
    content
        .lines()
        .filter_map(|line| {
            let start = line.find('[')?;
            let end = line.find(']')?;
            let ts = &line[start + 1..end];
            let text = line[end + 1..].trim();
            let time_ms = parse_timestamp_to_ms(ts)?;
            Some(json!({"time_ms": time_ms, "text": text}))
        })
        .collect()
}

fn parse_timestamp_to_ms(ts: &str) -> Option<u64> {
    let (min, rest) = ts.split_once(':')?;
    let (sec, frac) = rest.split_once('.').unwrap_or((rest, "0"));
    let minutes: u64 = min.parse().ok()?;
    let seconds: u64 = sec.parse().ok()?;
    let millis: u64 = match frac.len() {
        0 => 0,
        1 => frac.parse::<u64>().ok()? * 100,
        2 => frac.parse::<u64>().ok()? * 10,
        _ => frac[..3].parse::<u64>().ok()?,
    };
    Some(minutes * 60_000 + seconds * 1_000 + millis)
}

fn validate_filename(filename: &str) -> ApiResult<()> {
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(ApiError::bad_request("Invalid filename"));
    }
    Ok(())
}

fn resolve_within_base(base: &str, rel: &str) -> ApiResult<PathBuf> {
    if rel.contains("..") {
        return Err(ApiError::bad_request("Path traversal is not allowed"));
    }

    let base_path = Path::new(base);
    let rel_path = PathBuf::from(rel);
    if rel_path.is_absolute() {
        if rel_path.starts_with(base_path) {
            return Ok(rel_path);
        }
        return Err(ApiError::forbidden("Path is outside app root"));
    }

    Ok(base_path.join(rel.trim_start_matches('/')))
}

fn path_join_for_client(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_string()
    } else {
        format!("{}/{}", parent.trim_end_matches('/'), name)
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_lrc, parse_timestamp_to_ms, path_join_for_client};

    #[test]
    fn test_parse_timestamp_to_ms() {
        assert_eq!(parse_timestamp_to_ms("01:02.34"), Some(62_340));
        assert_eq!(parse_timestamp_to_ms("00:10"), Some(10_000));
        assert_eq!(parse_timestamp_to_ms("bad"), None);
    }

    #[test]
    fn test_parse_lrc() {
        let content = "[00:01.00]hello
[00:02.50]world";
        let parsed = parse_lrc(content);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["time_ms"], 1000);
        assert_eq!(parsed[0]["text"], "hello");
    }

    #[test]
    fn test_path_join_for_client() {
        assert_eq!(path_join_for_client("", "a.mp3"), "a.mp3");
        assert_eq!(path_join_for_client("music", "a.mp3"), "music/a.mp3");
    }
}

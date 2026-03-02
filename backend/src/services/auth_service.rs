use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, ColumnTrait, QueryFilter};
use anyhow::{Result, anyhow};

use crate::models::user::{self, ActiveModel, Entity as User, UserRole};
use crate::middleware::auth;

pub async fn login(
    db: &DatabaseConnection,
    username: &str,
    password: &str,
    jwt_secret: &str,
) -> Result<(String, user::Model)> {
    // Find user by username
    let user = User::find()
        .filter(user::Column::Username.eq(username))
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Invalid credentials"))?;
    
    // Verify password
    if !bcrypt::verify(password, &user.password_hash)? {
        return Err(anyhow!("Invalid credentials"));
    }
    
    // Generate token
    let role = format!("{:?}", user.role).to_lowercase();
    let token = auth::create_token(user.id, &user.username, &role, jwt_secret)?;
    
    Ok((token, user))
}

pub async fn register(
    db: &DatabaseConnection,
    username: &str,
    password: &str,
    role: &str,
) -> Result<user::Model> {
    // Check if username exists
    let existing = User::find()
        .filter(user::Column::Username.eq(username))
        .one(db)
        .await?;
    
    if existing.is_some() {
        return Err(anyhow!("Username already exists"));
    }
    
    // Hash password
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
    
    // Parse role
    let user_role = match role {
        "admin" => UserRole::Admin,
        _ => UserRole::User,
    };
    
    // Create user
    let now = chrono::Utc::now();
    let new_user = ActiveModel {
        username: Set(username.to_string()),
        password_hash: Set(password_hash),
        role: Set(user_role),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    
    let user = new_user.insert(db).await?;
    Ok(user)
}

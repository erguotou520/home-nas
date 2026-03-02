use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, ColumnTrait, QueryFilter};
use anyhow::{Result, anyhow};
use serde::Serialize;

use crate::models::user::{self, ActiveModel, Entity as User, UserRole};

#[derive(Serialize)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub role: String,
    pub created_at: String,
}

pub async fn list_users(db: &DatabaseConnection) -> Result<Vec<UserInfo>> {
    let users = User::find().all(db).await?;
    
    Ok(users.into_iter().map(|u| UserInfo {
        id: u.id,
        username: u.username,
        role: format!("{:?}", u.role).to_lowercase(),
        created_at: u.created_at.to_rfc3339(),
    }).collect())
}

pub async fn update_user(
    db: &DatabaseConnection,
    user_id: i32,
    role: Option<&str>,
    password: Option<&str>,
) -> Result<user::Model> {
    let user = User::find_by_id(user_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("User not found"))?;
    
    let mut active: ActiveModel = user.into();
    
    if let Some(role_str) = role {
        let new_role = match role_str {
            "admin" => UserRole::Admin,
            _ => UserRole::User,
        };
        active.role = Set(new_role);
    }
    
    if let Some(pwd) = password {
        let hash = bcrypt::hash(pwd, bcrypt::DEFAULT_COST)?;
        active.password_hash = Set(hash);
    }
    
    active.updated_at = Set(chrono::Utc::now());
    
    let updated = active.update(db).await?;
    Ok(updated)
}

pub async fn delete_user(db: &DatabaseConnection, user_id: i32) -> Result<()> {
    User::delete_by_id(user_id).exec(db).await?;
    Ok(())
}

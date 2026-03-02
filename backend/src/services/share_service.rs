use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, ColumnTrait, QueryFilter};
use anyhow::{Result, anyhow};
use uuid::Uuid;
use chrono::{Utc, Duration};
use serde::Serialize;

use crate::models::share::{self, ActiveModel, Entity as Share};

#[derive(Serialize)]
pub struct ShareInfo {
    pub id: i32,
    pub file_path: String,
    pub app_type: String,
    pub token: String,
    pub expires_at: Option<String>,
    pub burn_after_read: bool,
    pub max_views: Option<i32>,
    pub view_count: i32,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct SharedFileInfo {
    pub file_path: String,
    pub app_type: String,
    pub is_valid: bool,
}

pub async fn create_share(
    db: &DatabaseConnection,
    user_id: i32,
    file_path: &str,
    app_type: &str,
    expires_in_hours: Option<i64>,
    burn_after_read: bool,
    max_views: Option<i32>,
) -> Result<share::Model> {
    let token = Uuid::new_v4().to_string().replace("-", "")[..16].to_string();
    let now = Utc::now();
    
    let expires_at = expires_in_hours.map(|hours| now + Duration::hours(hours));
    
    let new_share = ActiveModel {
        user_id: Set(user_id),
        file_path: Set(file_path.to_string()),
        app_type: Set(app_type.to_string()),
        token: Set(token),
        expires_at: Set(expires_at),
        burn_after_read: Set(burn_after_read),
        max_views: Set(max_views),
        view_count: Set(0),
        created_at: Set(now),
        ..Default::default()
    };
    
    let share = new_share.insert(db).await?;
    Ok(share)
}

pub async fn list_user_shares(db: &DatabaseConnection, user_id: i32) -> Result<Vec<ShareInfo>> {
    let shares = Share::find()
        .filter(share::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    
    Ok(shares.into_iter().map(|s| ShareInfo {
        id: s.id,
        file_path: s.file_path,
        app_type: s.app_type,
        token: s.token,
        expires_at: s.expires_at.map(|d| d.to_rfc3339()),
        burn_after_read: s.burn_after_read,
        max_views: s.max_views,
        view_count: s.view_count,
        created_at: s.created_at.to_rfc3339(),
    }).collect())
}

pub async fn delete_share(db: &DatabaseConnection, share_id: i32, user_id: i32) -> Result<()> {
    let share = Share::find_by_id(share_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Share not found"))?;
    
    // Only owner can delete
    if share.user_id != user_id {
        return Err(anyhow!("Not authorized"));
    }
    
    Share::delete_by_id(share_id).exec(db).await?;
    Ok(())
}

pub async fn access_share(db: &DatabaseConnection, token: &str) -> Result<SharedFileInfo> {
    let share = Share::find()
        .filter(share::Column::Token.eq(token))
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Share not found"))?;
    
    if !share.is_valid() {
        return Err(anyhow!("Share has expired or reached view limit"));
    }
    
    // Increment view count
    let mut active: ActiveModel = share.clone().into();
    active.view_count = Set(share.view_count + 1);
    active.update(db).await?;
    
    Ok(SharedFileInfo {
        file_path: share.file_path,
        app_type: share.app_type,
        is_valid: true,
    })
}

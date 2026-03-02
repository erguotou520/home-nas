use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "shares")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub file_path: String,
    pub app_type: String,
    #[sea_orm(unique)]
    pub token: String,
    pub expires_at: Option<DateTimeUtc>,
    pub burn_after_read: bool,
    pub max_views: Option<i32>,
    pub view_count: i32,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn is_valid(&self) -> bool {
        // Check expiration
        if let Some(expires_at) = self.expires_at {
            if expires_at < chrono::Utc::now() {
                return false;
            }
        }
        
        // Check max views
        if let Some(max) = self.max_views {
            if self.view_count >= max {
                return false;
            }
        }
        
        // Check burn after read
        if self.burn_after_read && self.view_count > 0 {
            return false;
        }
        
        true
    }
}

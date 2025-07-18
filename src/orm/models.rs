use sea_orm::entity::prelude::*;
use chrono::{DateTime, Utc};

// model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "packages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub hash: Option<String>,
    pub signature: Option<String>,
    pub encrypted_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

// active model
#[derive(Debug, Clone, Copy, EnumIter, DeriveRelation)] pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}

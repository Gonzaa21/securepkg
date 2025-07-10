use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, DatabaseConnection, ActiveModelTrait};
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

// insert_pkg function to use in publish cmd
pub async fn insert_package(
    conn: &DatabaseConnection,
    name: String,
    version: String,
    author: Option<String>,
) -> Result<(), sea_orm::DbErr> {
    let new_package = ActiveModel {
        id: ActiveValue::NotSet,
        name: ActiveValue::Set(name),
        version: ActiveValue::Set(version),
        author: ActiveValue::Set(author),
        hash: ActiveValue::Set(None),
        signature: ActiveValue::Set(None),
        encrypted_path: ActiveValue::Set(None),
        created_at: ActiveValue::Set(Utc::now()),
    };

    new_package.insert(conn).await?;
    Ok(())
}

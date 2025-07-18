use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, IntoActiveModel};
use chrono::{DateTime, Utc};
use base64::engine::general_purpose;
use base64::Engine;

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
pub async fn insert_package(conn: &DatabaseConnection, name: String, version: String, author: Option<String>, hash: Option<String>, encrypted_path: Option<String>) -> Result<(), sea_orm::DbErr> {
    let new_package = ActiveModel {
        id: ActiveValue::NotSet,
        name: ActiveValue::Set(name),
        version: ActiveValue::Set(version),
        author: ActiveValue::Set(author),
        hash: ActiveValue::Set(hash),
        signature: ActiveValue::Set(None),
        encrypted_path: ActiveValue::Set(encrypted_path),
        created_at: ActiveValue::Set(Utc::now()),
    };

    new_package.insert(conn).await?;
    Ok(())
}

// find_pkg function
pub async fn find_pkg(conn: &DatabaseConnection, name: &str, version: &str) -> Result<Option<Model>, sea_orm::DbErr> {
    Entity::find() // filters using WHERE cond
        .filter(Column::Name.eq(name))
        .filter(Column::Version.eq(version))
        .one(conn) // execute query
        .await
}

// signature function
pub async fn update_signature(db: &DatabaseConnection, name: &str, version: &str, signature: Vec<u8>) -> Result<(), DbErr> {
    let result: Option<Model> = find_pkg(db, name, version).await?;
    
    match result {
        Some(pkg) => {
            let mut active_pkg = pkg.into_active_model();
            let sig_str = general_purpose::STANDARD.encode(&signature); // convert Vec<u8> to String
            active_pkg.signature = Set(Some(sig_str)); // asign sign to ActiveModel field
            active_pkg.update(db).await?; // save changes in db
        }
        None => return Err(DbErr::Custom("❌ Package not found".into())),
    }
    Ok(())
}

pub async fn list_pkg(conn: &DatabaseConnection) -> Result<Vec<Model>, DbErr> {
    let packages = Entity::find().all(conn).await?;
    Ok(packages)
}
use sea_orm::{Database, DatabaseConnection};
use crate::storage;

pub async fn connectdb() -> Result<DatabaseConnection, sea_orm::DbErr> {
    let sqlite_db = storage::get_db_path();
    let db_url = format!("sqlite://{}", sqlite_db.to_string_lossy());
    let conn = Database::connect(&db_url).await?;
    Ok(conn)
}
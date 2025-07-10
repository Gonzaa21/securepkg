use sea_query::SqliteQueryBuilder;
use sea_orm::{Database, DatabaseConnection};
use sea_orm::{DbBackend, Statement, ConnectionTrait};
use sea_query::{Table, ColumnDef, Expr, Iden};
use crate::storage;
pub mod models;

// connect to database
pub async fn connectdb() -> Result<DatabaseConnection, sea_orm::DbErr> {
    let sqlite_db = storage::get_db_path();
    let db_url = format!("sqlite://{}", sqlite_db.to_string_lossy());
    let conn = Database::connect(&db_url).await?;
    Ok(conn)
}

// pkgs columns
#[derive(Iden)]
enum Packages {
    Table,
    Id,
    Name,
    Version,
    Author,
    Hash,
    Signature,
    EncryptedPath,
    CreatedAt,
}

pub async fn create_table(conn: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    // create table
    let table = Table::create()
        .table(Packages::Table)
        .if_not_exists()
        .col(ColumnDef::new(Packages::Id).integer().not_null().auto_increment().primary_key())
        .col(ColumnDef::new(Packages::Name).string().not_null())
        .col(ColumnDef::new(Packages::Version).string().not_null())
        .col(ColumnDef::new(Packages::Author).string())
        .col(ColumnDef::new(Packages::Hash).string())
        .col(ColumnDef::new(Packages::Signature).string())
        .col(ColumnDef::new(Packages::EncryptedPath).string())
        .col(ColumnDef::new(Packages::CreatedAt).date_time().default(Expr::current_timestamp()))
        .to_owned();

    let sql = table.to_string(SqliteQueryBuilder); // convert statement to SQL
    conn.execute(Statement::from_string(DbBackend::Sqlite, sql)).await?; // execute query
    Ok(())
}
// connect to database
#[macro_export]
macro_rules! connect_db {
    () => {{
        match $crate::orm::connectdb().await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("❌ Error to connect DB: {e}");
                return;
            }
        }
    }}
}
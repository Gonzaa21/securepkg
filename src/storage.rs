use std::fs::File;

pub fn init_local_repo() -> std::io::Result<()> {
    let home_path = dirs::home_dir().expect("Could not get HOME directory");
    let folder_path = home_path.join(".securepkg");

    if folder_path.exists() { // verifying if folder exist
        println!("📁 Already existing folder: {}", folder_path.display());
    } else {
        std::fs::create_dir_all(&folder_path)?;
        println!("📁 Folder created: {}", folder_path.display());
    }

    let db_path = home_path.join("db.sqlite");
    if db_path.exists() { // verifying if SQLite database exist
        println!("🗄️ Already existing database: {}", db_path.display());
    } else {
        File::create(&db_path)?;
        println!("🗄️ Database created: {}", db_path.display());
    }
    Ok(())
}

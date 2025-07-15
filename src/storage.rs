use std::fs::File;
use std::path::PathBuf;
use rand::Rng;
use rsa::pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey};
use rsa::rand_core::OsRng;
use tokio::fs;

pub async fn init_local_repo() -> std::io::Result<()> {
    let home_path = dirs::home_dir().expect("Could not get HOME directory");
    let folder_path = home_path.join(".securepkg");
    let key_dir = folder_path.join("keys");
    let key_path = key_dir.join("secret.key");
    let pkg_dir = folder_path.join("packages");

    if folder_path.exists() { // verifying if folder exist
        println!("📁 Already existing folder: {}", folder_path.display());
    } else {
        std::fs::create_dir_all(&folder_path)?;
        println!("📁 Folder created: {}", folder_path.display());
    }

    if key_dir.exists() { // verifying if key folder exist
        println!("Already existing key dir: {}", key_dir.display());
    } else {
        std::fs::create_dir_all(&key_dir)?;
        println!("key dir generated: {}", key_dir.display());
    }

    if !key_path.exists() { // verifying if key file exist
        let mut key = [0u8; 32]; // array limited with 32bytes
        rand::rng().fill(&mut key); // generate random key
        std::fs::write(&key_path, key)?; // save in key_dir
        println!("key generated: {}", key_path.display());
    } else {
        println!("Already existing key: {}", key_path.display());
    }

    if !pkg_dir.exists() { // verifying if pkg dir exist
        std::fs::create_dir_all(&pkg_dir)?;
        println!("📦 Packages dir created: {}", pkg_dir.display());
    } else {
        println!("📦 Already existing packages dir: {}", pkg_dir.display());
    }

    let db_path = folder_path.join("db.sqlite");
    if db_path.exists() { // verifying if SQLite database exist
        println!("🗄️ Already existing database: {}", db_path.display());
    } else {
        File::create(&db_path)?;
        println!("🗄️ Database created: {}", db_path.display());
    }

    generate_keypair().await?;
    Ok(())
}

async fn generate_keypair() -> std::io::Result<()> {
    // paths
    let key_dir = dirs::home_dir()
        .expect("Could not get HOME directory")
        .join(".securepkg")
        .join("keys");
    let private_key_path = key_dir.join("private.pem");
    let public_key_path = key_dir.join("public.pem");

    // verify if paths already exists
    if private_key_path.exists() && public_key_path.exists() {
        println!("🔑 RSA keypair already exists.");
        return Ok(());
    }

    // generate keys
    let mut rng = OsRng;
    let bits = 2048;

    let private_key = rsa::RsaPrivateKey::new(&mut rng, bits)
        .expect("Failed to generate private key");

    let public_key = rsa::RsaPublicKey::from(&private_key);

    // convert keys to PEM
    let priv_key_pem = &private_key.to_pkcs1_pem(Default::default()).unwrap();
    let pub_key_pem = &public_key.to_pkcs1_pem(Default::default()).unwrap();

    // write keys
    fs::write(private_key_path, priv_key_pem).await?;
    fs::write(public_key_path, pub_key_pem).await?;

    Ok(())
}

// get funcions
pub fn get_securepkg_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not get HOME")
        .join(".securepkg")
}

pub fn get_db_path() -> PathBuf {
    get_securepkg_dir().join("db.sqlite")
}

pub fn get_key_path() -> PathBuf {
    dirs::home_dir().unwrap()
    .join(".securepkg")
    .join("keys")
    .join("secret.key")
}

pub fn get_pkg_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".securepkg").join("packages")
}
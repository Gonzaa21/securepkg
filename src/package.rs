use std::{fs::{self, File}, path::{Path, PathBuf}};
use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce, aead::{Aead, OsRng, rand_core::RngCore}};
use zip::{write::FileOptions, CompressionMethod};
use walkdir::WalkDir;
use rsa::{pkcs1::DecodeRsaPrivateKey,pkcs1::DecodeRsaPublicKey, RsaPrivateKey, Pkcs1v15Sign, RsaPublicKey, traits::SignatureScheme};
use sha2::{Digest, Sha256};
use crate::{orm, storage};
use sea_orm::DatabaseConnection;

// to compress dirs in .zip
pub fn zip_dir(src_dir: &Path, dst_file: &Path) -> zip::result::ZipResult<()> {
    let file = File::create(dst_file)?; // Create file
    let mut zip = zip::ZipWriter::new(file); // zip writer

    // file options
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // travel dirs and subdirs in src_dir
    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.strip_prefix(src_dir).unwrap(); // keep path structure and use a relative way

        if path.is_file() { // if is file:
            zip.start_file(name.to_string_lossy(), options)?; // create file
            let mut f = File::open(path)?;
            std::io::copy(&mut f, &mut zip)?; // copy its content in zip archive
        } else if name.as_os_str().len() != 0 { // if is folder:
            zip.add_directory(name.to_string_lossy(), options)?; // add a dir to zip
        }
        zip.finish()?; // close zip
    }
    Ok(())
}

// to encrypt zip archive in pkg
pub fn encrypt_zip(input: &Path, output: &Path, key: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let read_zip = fs::read(input)?; // read input path (zip)
    let read_key = fs::read(key)?; // read key path

    let key = Key::from_slice(&read_key); // convert array
    let cipher = ChaCha20Poly1305::new(key); // create encrypted value

    let mut nonce = [0u8; 12]; // nonce array limited 12bytes
    OsRng.fill_bytes(&mut nonce);
    let nonce = Nonce::from_slice(&nonce); // convert array to nonce

    // encrypt key and convert read_zip to &[u8]
    let cipher_text = cipher.encrypt(nonce, read_zip.as_ref()).map_err(|e| format!("Error to code: {:?}", e))?;

    let mut content = Vec::with_capacity(nonce.len() + cipher_text.iter().len());
    content.extend_from_slice(nonce); // add elements in nonce
    content.extend_from_slice(&cipher_text); // wait a &[u8]

    std::fs::write(output, &content)?; // write content in pkg path
    
    Ok(())
}

// sign .pkg with priv key
pub fn sign_pkg(pkg_path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let pkg_data = fs::read(pkg_path)?; // read content .pkg

    // read private key path
    let priv_key_path = dirs::home_dir()
        .expect("Could not get HOME directory")
        .join(".securepkg")
        .join("keys")
        .join("private.pem");

    let pem = fs::read_to_string(priv_key_path)?;
    let private_key = RsaPrivateKey::from_pkcs1_pem(&pem)?;

    // hash pkg content and sign
    let digest = Sha256::digest(&pkg_data);
    let signature = private_key.sign(Pkcs1v15Sign::new::<Sha256>(), &digest)?;

    Ok(signature)
}

// export pkg
pub async fn export_pkg(name: &str, version: &str, conn: &DatabaseConnection, repo: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    // find conn, name, version
    let pkg = match orm::models::find_pkg(conn, name, version).await {
        Ok(Some(pkg)) => pkg,
        Ok(None) => return Err("Package not found in database".into()),
        Err(e) => return Err(format!("DB error: {:?}", e).into()),
    };

    // validate if package has encrypted path
    let encrypted_path = match pkg.encrypted_path {
        Some(path) => PathBuf::from(path),
        None => return Err("Package has no encrypted path.".into()),
    };

    // determine destination dir
    let export_dir = match repo {
        Some(path) => PathBuf::from(path),
        None => storage::get_securepkg_dir().join("exports")
    };

    // create .securepkg/exports folder
    if !export_dir.exists() {
        fs::create_dir_all(&export_dir)?;
    } else {
        println!("📁 Already existing Exports folder: {}", export_dir.display());
    }

    // add filename and copy to export_path
    let export_path = export_dir.join(encrypted_path.file_name().unwrap());
    fs::copy(&encrypted_path, &export_path)?;
    println!("📤 Package exported to {:?}", export_path);

    // copy .sig if exists
    let sig_src = encrypted_path.with_extension("sig");
    let sig_dst = export_dir.join(sig_src.file_name().unwrap());
    
    if sig_src.exists() {
        fs::copy(&sig_src, &sig_dst)?;
        println!("🖊️ Signature file exported to {:?}", sig_dst);
    } else {
        println!("⚠️ No signature file found to export");
    }

    Ok(())
}

// install pkg: validate signature, decrypt pkg to zip and then zip to file
pub async fn install_pkg(name: &str, version: &str, from_file: Option<&Path>) -> Result<(), Box<dyn std::error::Error>> {
    // search and validate if package exists
    let pkg_path = if let Some(path) = from_file {
        PathBuf::from(path)
    } else {
        storage::get_securepkg_dir().join("exports").join(format!("{}-{}.pkg", name, version))
    };
    
    if pkg_path.exists() {
        println!("📦 Found exported package at {}", pkg_path.display());

    } else {
        return Err("Package not found in exports folder.".into());
    }

    // validate sign
    let sign_path = pkg_path.with_extension("sig");
    let sign = fs::read(&sign_path).map_err(|e| format!("Signature file not found: {}", e))?;

    let pub_key_path = storage::get_securepkg_dir().join("keys").join("public.pem");
    let pub_pem = fs::read_to_string(&pub_key_path)?;
    let public_key = RsaPublicKey::from_pkcs1_pem(&pub_pem)?;

    let pkg_data = fs::read(&pkg_path)?;
    let digest = Sha256::digest(&pkg_data);
    let verifier = Pkcs1v15Sign::new::<Sha256>();
    verifier.verify(&public_key, &digest, &sign).map_err(|_| "Invalid signature")?;

    println!("🧾 Signature verified successfully");

    // DECRYPT PKG TO ZIP
    // read secret key
    let key_path = storage::get_key_path();
    let key_bytes = fs::read(&key_path)?;
    
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key_bytes));

    // devide nonce and cipher_text
    let (nonce_bytes, ciphertext) = pkg_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    // decrypt pkg
    let decrypted_data = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {:?}", e))?;


    // EXTRACT ZIP TO FOLDER
    // save temporally in temp.zip
    let temp_zip = storage::get_securepkg_dir().join("temp.zip");
    fs::write(&temp_zip, &decrypted_data)?;

    // create .securepkg/install folder
    let install_path = storage::get_securepkg_dir()
    .join("installed")
    .join(format!("{}-{}", name, version));
    fs::create_dir_all(&install_path)?;
    
    let zip_file = File::open(&temp_zip)?; // open zip file
    let mut file = zip::ZipArchive::new(zip_file)?; // read zip file
    file.extract(&install_path)?; // extract file to install dir
    println!("📁 Package extracted to: {}", install_path.display());

    fs::remove_file(&temp_zip)?; // remove temporal file

    Ok(())
}
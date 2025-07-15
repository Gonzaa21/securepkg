use clap::{Parser, Subcommand};
use crate::{orm::{self, models::find_pkg}, package::{encrypt_zip, sign_pkg}, storage};
use std::{fs, path::PathBuf};
use crate::package::zip_dir;
use sha2::{Sha256, Digest};
use hex;

// CLI struct
#[derive(Parser)]
#[command(name = "securepkg")]
#[command(about = "Encrypted package manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

// Commands
#[derive(Subcommand)]
pub enum Commands {
    /// Start package environment
    Init,
    Package {
        #[command(subcommand)]
        subcommand: PackageSubcommand,
    }
}

// Subcommands
#[derive(Subcommand)]
pub enum PackageSubcommand {
    Build {
        path: PathBuf,
        name: String,
        version: String,
        author: Option<String>,
    },
    Publish {
        name: String,
        version: String
    }
}

pub async fn run() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            println!("Starting...");
            if let Err(e) = storage::init_local_repo().await {
                eprintln!("Set up error: {e}");
            } else {
                println!("✅ Local repository initialized ~/.securepkg");
            }
        },
        Commands::Package { subcommand } => {
            match subcommand {
                PackageSubcommand::Build { path, name, version, author } => {
                    println!("🚧 package build:");
                    println!("Path: {path:?}, Name: {name}, Version: {version}, Author: {:?}", author);

                    let filename = format!("{}-{}.zip", name, version);
                    let output_path = PathBuf::from(filename);

                    match zip_dir(&path, &output_path) {
                        Ok(_) => println!("✅ Package created at {:?}", output_path),
                        Err(e) => eprintln!("❌ Error creating package: {:?}", e),
                    }
                    
                    let input = PathBuf::from(format!("{}-{}.zip", name, version));
                    let output = storage::get_pkg_dir().join(format!("{}-{}.pkg", name, version));
                    let key = storage::get_key_path();

                    match encrypt_zip(&input, &output, &key) {
                        Ok(_) => println!("🔐 archive encrypted correctly {:?}", output),
                        Err(e) => eprintln!("❌ Error encrypting file: {:?}", e),
                    }

                    // connect and save pkg into DB
                    let data_pkg = fs::read(&output);
                    let mut hasher = Sha256::new(); // create hash
                    hasher.update(data_pkg.unwrap()); // update hash using data_pkg
                    let hash = hasher.finalize(); // return result
                    let hash_hex = hex::encode(&hash); // convert to hex string
                    
                    let conn = match orm::connectdb().await {
                        Ok(conn) => {
                            println!("🔗 DB connected correctly");
                            conn
                        },
                        Err(e) => {
                            eprintln!("❌ Error to connect DB: '{e}'");
                            return;
                        }
                    };

                    orm::create_table(&conn).await.expect("❌ Error creating table");
                    match orm::insert_package(&conn, name, version, author, Some(hash_hex), Some(output.to_string_lossy().to_string())).await {
                        Ok(_) => println!("📦 Package inserted into database"),
                        Err(e) => eprintln!("❌ Error inserting into database: {:?}", e),
                    }
                },
                PackageSubcommand::Publish { name, version } => {
                    // connect to db
                    let conn = match orm::connectdb().await {
                        Ok(conn) => conn,
                        Err(e) => {
                            eprintln!("❌ Error to connect DB: {e}");
                            return;
                        }
                    };

                    // verify if package exists
                    let pkg = match find_pkg(&conn, &name, &version).await {
                        Ok(Some(pkg)) => {
                            println!("📦 Package found: {:?}", pkg);
                            pkg
                        },
                        Ok(None) => {
                            eprintln!("❌ Package not found in database");
                            return;
                        },
                        Err(e) => {
                            eprintln!("❌ DB error: {:?}", e);
                            return;
                        }
                    };

                    // sign
                    let pkg_path = PathBuf::from(pkg.encrypted_path.unwrap());
                    let signature = match sign_pkg(&pkg_path) {
                        Ok(sig) => {
                            println!("🖊️ Package successfully signed");
                            sig
                        }
                        Err(e) => {
                            eprintln!("❌ Signature error: {:?}", e);
                            return;
                        }
                    };

                    let sig_path = pkg_path.with_extension("sig"); // convert .pkg to .sig
                    // save in .sig file
                    match fs::write(&sig_path, &signature) {
                        Ok(_) => println!("💾 Signature saved to: {:?}", sig_path),
                        Err(e) => {
                            eprintln!("❌ Error writing signature file: {:?}", e);
                            return;
                        }
                    };

                    match orm::models::update_signature(&conn, &name, &version, signature).await {
                        Ok(_) => println!("🗄️ Signature updated in database"),
                        Err(e) => eprintln!("❌ Error saving signature in DB: {:?}", e),
                    }

                }
            }
        }
    }
}

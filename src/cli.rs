use clap::{Parser, Subcommand};
use crate::{orm, storage};

#[derive(Parser)]
#[command(name = "securepkg")]
#[command(about = "Encrypted package manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start package environment
    Init,
}

pub async fn run() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            println!("Starting...");
            if let Err(e) = storage::init_local_repo() {
                eprintln!("Set up error: {e}");
            } else {
                println!("✅ Local repository initialized ~/.securepkg");
            }

            match orm::connectdb().await {
                Ok(conn) => {
                    println!("🔗 DB connected correctly");
                    orm::create_table(&conn).await.expect("❌ Error creating table");
                },
                Err(e) => eprintln!("❌ Error to connect DB: '{e}'")
            }
        }
    }
}

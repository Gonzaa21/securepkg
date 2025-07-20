mod cli;
mod storage;
mod orm;
mod package;
#[macro_use]
mod dsl;

#[tokio::main]
async fn main() {
    cli::run().await;
}

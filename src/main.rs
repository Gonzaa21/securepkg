mod cli;
mod storage;
mod orm;
mod package;

#[tokio::main]
async fn main() {
    cli::run().await;
}

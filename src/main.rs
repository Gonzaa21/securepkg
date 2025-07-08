mod cli;
mod storage;
mod orm;

#[tokio::main]
async fn main() {
    cli::run().await;
}

use sea_orm_migration::prelude::*;
use dotenv::dotenv;

#[async_std::main]
async fn main() {
    dotenv().ok(); 
    cli::run_cli(migration::Migrator).await;
}

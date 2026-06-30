
use std::env;

use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn connect_db() -> PgPool {
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in env!");

    println!("Connecting to: {}", database_url);
    
    PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to databse!")
}
mod controllers;
mod error;
mod models;

use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Database URL - in production, you'd use an environment variable
    let database_url = "sqlite://feedme.db";

    // Create database if it doesn't exist
    if !sqlx::Sqlite::database_exists(database_url).await? {
        println!("Creating database {}", database_url);
        sqlx::Sqlite::create_database(database_url).await?;
    }

    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Run migrations
    println!("Running migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;

    println!("Database setup complete!");

    Ok(())
}

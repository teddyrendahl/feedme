use rstest::*;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

/// Test fixture that creates an in-memory SQLite database with migrations applied
///
/// This fixture can be imported and used across all model tests to ensure
/// consistency in test database setup.
#[fixture]
pub async fn test_db() -> SqlitePool {
    // Create an in-memory SQLite database
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

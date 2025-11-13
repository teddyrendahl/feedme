use sqlx::prelude::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct IngredientRecord {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::test_fixtures::test_db;
    use rstest::*;
    use sqlx::SqlitePool;

    #[rstest]
    #[tokio::test]
    async fn test_ingredient_model_compatibility(#[future] test_db: SqlitePool) {
        let pool = test_db.await;

        // Insert a test ingredient
        sqlx::query("INSERT INTO ingredients (name) VALUES (?)")
            .bind("Test Ingredient")
            .execute(&pool)
            .await
            .expect("Failed to insert ingredient");

        // Query and map to IngredientRecord struct
        let ingredient = sqlx::query_as::<_, IngredientRecord>(
            "SELECT id, name, created_at FROM ingredients WHERE name = ?",
        )
        .bind("Test Ingredient")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch ingredient");

        // Verify the model fields match
        assert_eq!(ingredient.name, "Test Ingredient");
        assert!(ingredient.id > 0);
        assert!(!ingredient.created_at.is_empty());
    }
}

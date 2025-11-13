use sqlx::prelude::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct RecipeRecord {
    pub id: i64,
    pub name: String,
    pub instructions: Option<String>,
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
    async fn test_recipe_model_compatibility_with_instructions(#[future] test_db: SqlitePool) {
        let pool = test_db.await;

        // Insert a test recipe with instructions
        sqlx::query("INSERT INTO recipes (name, instructions) VALUES (?, ?)")
            .bind("Test Recipe")
            .bind("Cook it well")
            .execute(&pool)
            .await
            .expect("Failed to insert recipe");

        // Query and map to RecipeRecord struct
        let recipe = sqlx::query_as::<_, RecipeRecord>(
            "SELECT id, name, instructions, created_at FROM recipes WHERE name = ?",
        )
        .bind("Test Recipe")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch recipe");

        // Verify the model fields match
        assert_eq!(recipe.name, "Test Recipe");
        assert_eq!(recipe.instructions, Some("Cook it well".to_string()));
        assert!(recipe.id > 0);
        assert!(!recipe.created_at.is_empty());
    }

    #[rstest]
    #[tokio::test]
    async fn test_recipe_model_compatibility_null_instructions(#[future] test_db: SqlitePool) {
        let pool = test_db.await;

        // Insert a test recipe without instructions
        sqlx::query("INSERT INTO recipes (name) VALUES (?)")
            .bind("Simple Recipe")
            .execute(&pool)
            .await
            .expect("Failed to insert recipe");

        // Query and map to RecipeRecord struct
        let recipe = sqlx::query_as::<_, RecipeRecord>(
            "SELECT id, name, instructions, created_at FROM recipes WHERE name = ?",
        )
        .bind("Simple Recipe")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch recipe");

        // Verify the model handles NULL instructions
        assert_eq!(recipe.name, "Simple Recipe");
        assert_eq!(recipe.instructions, None);
        assert!(recipe.id > 0);
        assert!(!recipe.created_at.is_empty());
    }
}

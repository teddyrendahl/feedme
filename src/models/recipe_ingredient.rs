use sqlx::prelude::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct RecipeIngredientRecord {
    pub id: i64,
    pub recipe_id: i64,
    pub ingredient_id: i64,
    pub quantity_unit: String,
    pub notes: Option<String>,
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
    async fn test_recipe_ingredient_model_compatibility_with_notes(#[future] test_db: SqlitePool) {
        let pool = test_db.await;

        // First, insert a recipe and an ingredient
        let recipe_id = sqlx::query("INSERT INTO recipes (name) VALUES (?)")
            .bind("Test Recipe")
            .execute(&pool)
            .await
            .expect("Failed to insert recipe")
            .last_insert_rowid();

        let ingredient_id = sqlx::query("INSERT INTO ingredients (name) VALUES (?)")
            .bind("Test Ingredient")
            .execute(&pool)
            .await
            .expect("Failed to insert ingredient")
            .last_insert_rowid();

        // Insert a recipe_ingredient with notes
        sqlx::query(
            "INSERT INTO recipe_ingredients (recipe_id, ingredient_id, quantity_unit, notes) VALUES (?, ?, ?, ?)"
        )
        .bind(recipe_id)
        .bind(ingredient_id)
        .bind("2 cups")
        .bind("diced")
        .execute(&pool)
        .await
        .expect("Failed to insert recipe_ingredient");

        // Query and map to RecipeIngredientRecord struct
        let recipe_ingredient = sqlx::query_as::<_, RecipeIngredientRecord>(
            "SELECT id, recipe_id, ingredient_id, quantity_unit, notes, created_at FROM recipe_ingredients WHERE recipe_id = ?"
        )
        .bind(recipe_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch recipe_ingredient");

        // Verify the model fields match
        assert_eq!(recipe_ingredient.recipe_id, recipe_id);
        assert_eq!(recipe_ingredient.ingredient_id, ingredient_id);
        assert_eq!(recipe_ingredient.quantity_unit, "2 cups");
        assert_eq!(recipe_ingredient.notes, Some("diced".to_string()));
        assert!(recipe_ingredient.id > 0);
        assert!(!recipe_ingredient.created_at.is_empty());
    }

    #[rstest]
    #[tokio::test]
    async fn test_recipe_ingredient_model_compatibility_null_notes(#[future] test_db: SqlitePool) {
        let pool = test_db.await;

        // First, insert a recipe and an ingredient
        let recipe_id = sqlx::query("INSERT INTO recipes (name) VALUES (?)")
            .bind("Test Recipe 2")
            .execute(&pool)
            .await
            .expect("Failed to insert recipe")
            .last_insert_rowid();

        let ingredient_id = sqlx::query("INSERT INTO ingredients (name) VALUES (?)")
            .bind("Test Ingredient 2")
            .execute(&pool)
            .await
            .expect("Failed to insert ingredient")
            .last_insert_rowid();

        // Insert a recipe_ingredient without notes
        sqlx::query(
            "INSERT INTO recipe_ingredients (recipe_id, ingredient_id, quantity_unit) VALUES (?, ?, ?)"
        )
        .bind(recipe_id)
        .bind(ingredient_id)
        .bind("1 pinch")
        .execute(&pool)
        .await
        .expect("Failed to insert recipe_ingredient");

        // Query and map to RecipeIngredientRecord struct
        let recipe_ingredient = sqlx::query_as::<_, RecipeIngredientRecord>(
            "SELECT id, recipe_id, ingredient_id, quantity_unit, notes, created_at FROM recipe_ingredients WHERE recipe_id = ?"
        )
        .bind(recipe_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch recipe_ingredient");

        // Verify the model handles NULL notes
        assert_eq!(recipe_ingredient.recipe_id, recipe_id);
        assert_eq!(recipe_ingredient.ingredient_id, ingredient_id);
        assert_eq!(recipe_ingredient.quantity_unit, "1 pinch");
        assert_eq!(recipe_ingredient.notes, None);
        assert!(recipe_ingredient.id > 0);
        assert!(!recipe_ingredient.created_at.is_empty());
    }
}

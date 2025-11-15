use sqlx::SqlitePool;

use crate::error::Result;
use crate::models::IngredientRecord;

/// Create a new ingredient
/// Returns the ingredient ID
/// Note: This will fail if an ingredient with the same name already exists (UNIQUE constraint)
pub async fn create_ingredient(pool: &SqlitePool, name: &str) -> Result<i64> {
    let ingredient_id = sqlx::query("INSERT INTO ingredients (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await?
        .last_insert_rowid();

    Ok(ingredient_id)
}

/// Get all ingredients from the database
/// Returns a list of all ingredients ordered by name
pub async fn get_all_ingredients(pool: &SqlitePool) -> Result<Vec<IngredientRecord>> {
    let ingredients = sqlx::query_as::<_, IngredientRecord>(
        "SELECT id, name, created_at FROM ingredients ORDER BY name",
    )
    .fetch_all(pool)
    .await?;

    Ok(ingredients)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::test_fixtures::test_db;
    use rstest::*;

    #[rstest]
    #[tokio::test]
    async fn test_create_ingredient(#[future] test_db: SqlitePool) {
        let pool = test_db.await;

        let ingredient_id = create_ingredient(&pool, "tomato")
            .await
            .expect("Failed to create ingredient");

        assert!(ingredient_id > 0);

        // Verify it was created
        let name: String = sqlx::query_scalar("SELECT name FROM ingredients WHERE id = ?")
            .bind(ingredient_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch ingredient");

        assert_eq!(name, "tomato");
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_ingredient_duplicate_name_fails(#[future] test_db: SqlitePool) {
        let pool = test_db.await;

        // Create first ingredient
        create_ingredient(&pool, "flour")
            .await
            .expect("Failed to create first ingredient");

        // Try to create duplicate
        let result = create_ingredient(&pool, "flour").await;

        assert!(result.is_err(), "Should fail with duplicate name");
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_multiple_ingredients(#[future] test_db: SqlitePool) {
        let pool = test_db.await;

        let id1 = create_ingredient(&pool, "salt")
            .await
            .expect("Failed to create salt");

        let id2 = create_ingredient(&pool, "pepper")
            .await
            .expect("Failed to create pepper");

        let id3 = create_ingredient(&pool, "sugar")
            .await
            .expect("Failed to create sugar");

        // All IDs should be unique
        assert!(id1 > 0);
        assert!(id2 > 0);
        assert!(id3 > 0);
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);

        // Count total ingredients
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ingredients")
            .fetch_one(&pool)
            .await
            .expect("Failed to count ingredients");

        assert_eq!(count, 3);
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_all_ingredients_empty(#[future] test_db: SqlitePool) {
        let pool = test_db.await;

        let ingredients = get_all_ingredients(&pool)
            .await
            .expect("Failed to get ingredients");

        assert_eq!(ingredients.len(), 0);
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_all_ingredients(#[future] test_db: SqlitePool) {
        let pool = test_db.await;

        // Create some ingredients
        create_ingredient(&pool, "flour")
            .await
            .expect("Failed to create flour");

        create_ingredient(&pool, "sugar")
            .await
            .expect("Failed to create sugar");

        create_ingredient(&pool, "butter")
            .await
            .expect("Failed to create butter");

        // Get all ingredients
        let ingredients = get_all_ingredients(&pool)
            .await
            .expect("Failed to get ingredients");

        assert_eq!(ingredients.len(), 3);

        // Verify they're ordered by name
        assert_eq!(ingredients[0].name, "butter");
        assert_eq!(ingredients[1].name, "flour");
        assert_eq!(ingredients[2].name, "sugar");

        // Verify all have IDs and created_at
        for ingredient in &ingredients {
            assert!(ingredient.id > 0);
            assert!(!ingredient.created_at.is_empty());
        }
    }
}

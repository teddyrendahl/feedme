use sqlx::{Row, SqlitePool};

use crate::error::Result;
use crate::models::RecipeRecord;
use crate::models::api::{Recipe, RecipeIngredient};

/// Fetch a recipe by ID with all its ingredients
pub async fn get_recipe(pool: &SqlitePool, recipe_id: i64) -> Result<Recipe> {
    // Fetch the recipe
    let recipe = sqlx::query_as::<_, RecipeRecord>(
        "SELECT id, name, instructions, created_at FROM recipes WHERE id = ?",
    )
    .bind(recipe_id)
    .fetch_optional(pool)
    .await?
    .ok_or(crate::error::FeedMeError::RecipeNotFound(recipe_id))?;

    // Fetch all recipe_ingredients for this recipe with ingredient names
    // Using a JOIN to get ingredient names in a single query
    let ingredients = sqlx::query(
        r#"
        SELECT
            i.name as ingredient_name,
            ri.quantity_unit,
            ri.notes
        FROM recipe_ingredients ri
        JOIN ingredients i ON ri.ingredient_id = i.id
        WHERE ri.recipe_id = ?
        ORDER BY ri.id
        "#,
    )
    .bind(recipe_id)
    .fetch_all(pool)
    .await?;

    // Map to RecipeIngredient structs
    let recipe_ingredients: Vec<RecipeIngredient> = ingredients
        .iter()
        .map(|row| RecipeIngredient {
            ingredient_name: row.get("ingredient_name"),
            quantity_unit: row.get("quantity_unit"),
            notes: row.get("notes"),
        })
        .collect();

    Ok(Recipe {
        id: recipe.id,
        name: recipe.name,
        instructions: recipe.instructions,
        created_at: recipe.created_at,
        ingredients: recipe_ingredients,
    })
}

/// Create a new recipe with ingredients
/// Takes a Recipe struct (ignoring id and created_at) and creates ingredients if they don't exist
pub async fn create_recipe(pool: &SqlitePool, recipe: &Recipe) -> Result<i64> {
    // Start a transaction
    let mut tx = pool.begin().await?;

    // Insert the recipe
    let recipe_id = sqlx::query("INSERT INTO recipes (name, instructions) VALUES (?, ?)")
        .bind(&recipe.name)
        .bind(&recipe.instructions)
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();

    // For each ingredient, find or create it, then add to recipe_ingredients
    for ingredient in &recipe.ingredients {
        // Try to find existing ingredient
        let ingredient_id: Option<i64> =
            sqlx::query_scalar("SELECT id FROM ingredients WHERE name = ?")
                .bind(&ingredient.ingredient_name)
                .fetch_optional(&mut *tx)
                .await?;

        // If ingredient doesn't exist, create it
        let ingredient_id = match ingredient_id {
            Some(id) => id,
            None => sqlx::query("INSERT INTO ingredients (name) VALUES (?)")
                .bind(&ingredient.ingredient_name)
                .execute(&mut *tx)
                .await?
                .last_insert_rowid(),
        };

        // Insert recipe_ingredient
        sqlx::query(
            "INSERT INTO recipe_ingredients (recipe_id, ingredient_id, quantity_unit, notes) VALUES (?, ?, ?, ?)"
        )
        .bind(recipe_id)
        .bind(ingredient_id)
        .bind(&ingredient.quantity_unit)
        .bind(&ingredient.notes)
        .execute(&mut *tx)
        .await?;
    }

    // Commit the transaction
    tx.commit().await?;

    Ok(recipe_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::test_fixtures::test_db;
    use rstest::*;

    #[rstest]
    #[tokio::test]
    async fn test_get_recipe(#[future] test_db: SqlitePool) {
        let pool = test_db.await;

        // Insert a recipe
        let recipe_id = sqlx::query("INSERT INTO recipes (name, instructions) VALUES (?, ?)")
            .bind("Pancakes")
            .bind("Mix and cook on griddle")
            .execute(&pool)
            .await
            .expect("Failed to insert recipe")
            .last_insert_rowid();

        // Insert ingredients
        let flour_id = sqlx::query("INSERT INTO ingredients (name) VALUES (?)")
            .bind("flour")
            .execute(&pool)
            .await
            .expect("Failed to insert flour")
            .last_insert_rowid();

        let milk_id = sqlx::query("INSERT INTO ingredients (name) VALUES (?)")
            .bind("milk")
            .execute(&pool)
            .await
            .expect("Failed to insert milk")
            .last_insert_rowid();

        // Insert recipe_ingredients
        sqlx::query(
            "INSERT INTO recipe_ingredients (recipe_id, ingredient_id, quantity_unit, notes) VALUES (?, ?, ?, ?)",
        )
        .bind(recipe_id)
        .bind(flour_id)
        .bind("2 cups")
        .bind("all-purpose")
        .execute(&pool)
        .await
        .expect("Failed to insert recipe_ingredient");

        sqlx::query(
            "INSERT INTO recipe_ingredients (recipe_id, ingredient_id, quantity_unit) VALUES (?, ?, ?)",
        )
        .bind(recipe_id)
        .bind(milk_id)
        .bind("1 cup")
        .execute(&pool)
        .await
        .expect("Failed to insert recipe_ingredient");

        // Fetch the recipe
        let recipe = get_recipe(&pool, recipe_id)
            .await
            .expect("Failed to fetch recipe");

        // Verify the recipe
        assert_eq!(recipe.id, recipe_id);
        assert_eq!(recipe.name, "Pancakes");
        assert_eq!(
            recipe.instructions,
            Some("Mix and cook on griddle".to_string())
        );

        // Verify ingredients
        assert_eq!(recipe.ingredients.len(), 2);

        let flour_ingredient = &recipe.ingredients[0];
        assert_eq!(flour_ingredient.ingredient_name, "flour");
        assert_eq!(flour_ingredient.quantity_unit, "2 cups");
        assert_eq!(flour_ingredient.notes, Some("all-purpose".to_string()));

        let milk_ingredient = &recipe.ingredients[1];
        assert_eq!(milk_ingredient.ingredient_name, "milk");
        assert_eq!(milk_ingredient.quantity_unit, "1 cup");
        assert_eq!(milk_ingredient.notes, None);
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_recipe_not_found(#[future] test_db: SqlitePool) {
        let pool = test_db.await;

        // Try to fetch a non-existent recipe
        let result = get_recipe(&pool, 999).await;

        assert!(result.is_err());

        // Verify it's the correct error type
        match result {
            Err(crate::error::FeedMeError::RecipeNotFound(id)) => {
                assert_eq!(id, 999);
            }
            _ => panic!("Expected RecipeNotFound error"),
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_recipe_no_ingredients(#[future] test_db: SqlitePool) {
        let pool = test_db.await;

        // Insert a recipe without ingredients
        let recipe_id = sqlx::query("INSERT INTO recipes (name) VALUES (?)")
            .bind("Empty Recipe")
            .execute(&pool)
            .await
            .expect("Failed to insert recipe")
            .last_insert_rowid();

        // Fetch the recipe
        let recipe = get_recipe(&pool, recipe_id)
            .await
            .expect("Failed to fetch recipe");

        // Verify the recipe has no ingredients
        assert_eq!(recipe.name, "Empty Recipe");
        assert_eq!(recipe.ingredients.len(), 0);
        assert_eq!(recipe.instructions, None);
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_recipe(#[future] test_db: SqlitePool) {
        let pool = test_db.await;

        // Create a recipe
        let new_recipe = Recipe {
            id: 0, // Will be ignored
            name: "Pasta Carbonara".to_string(),
            instructions: Some("Cook pasta, fry bacon, mix with eggs".to_string()),
            created_at: String::new(), // Will be ignored
            ingredients: vec![
                RecipeIngredient {
                    ingredient_name: "pasta".to_string(),
                    quantity_unit: "500g".to_string(),
                    notes: Some("spaghetti".to_string()),
                },
                RecipeIngredient {
                    ingredient_name: "bacon".to_string(),
                    quantity_unit: "200g".to_string(),
                    notes: None,
                },
                RecipeIngredient {
                    ingredient_name: "eggs".to_string(),
                    quantity_unit: "3 whole".to_string(),
                    notes: None,
                },
            ],
        };

        let recipe_id = create_recipe(&pool, &new_recipe)
            .await
            .expect("Failed to create recipe");

        // Verify the recipe was created
        assert!(recipe_id > 0);

        // Fetch the recipe back and verify
        let fetched_recipe = get_recipe(&pool, recipe_id)
            .await
            .expect("Failed to fetch created recipe");

        assert_eq!(fetched_recipe.name, "Pasta Carbonara");
        assert_eq!(
            fetched_recipe.instructions,
            Some("Cook pasta, fry bacon, mix with eggs".to_string())
        );
        assert_eq!(fetched_recipe.ingredients.len(), 3);

        // Verify ingredients
        assert_eq!(fetched_recipe.ingredients[0].ingredient_name, "pasta");
        assert_eq!(fetched_recipe.ingredients[0].quantity_unit, "500g");
        assert_eq!(
            fetched_recipe.ingredients[0].notes,
            Some("spaghetti".to_string())
        );

        assert_eq!(fetched_recipe.ingredients[1].ingredient_name, "bacon");
        assert_eq!(fetched_recipe.ingredients[2].ingredient_name, "eggs");
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_recipe_reuses_existing_ingredients(#[future] test_db: SqlitePool) {
        let pool = test_db.await;

        // Create first recipe with flour
        let recipe1 = Recipe {
            id: 0,
            name: "Pancakes".to_string(),
            instructions: None,
            created_at: String::new(),
            ingredients: vec![RecipeIngredient {
                ingredient_name: "flour".to_string(),
                quantity_unit: "2 cups".to_string(),
                notes: None,
            }],
        };

        create_recipe(&pool, &recipe1)
            .await
            .expect("Failed to create first recipe");

        // Count how many times "flour" exists in ingredients table
        let flour_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM ingredients WHERE name = ?")
                .bind("flour")
                .fetch_one(&pool)
                .await
                .expect("Failed to count flour");

        assert_eq!(flour_count, 1);

        // Create second recipe also with flour
        let recipe2 = Recipe {
            id: 0,
            name: "Bread".to_string(),
            instructions: None,
            created_at: String::new(),
            ingredients: vec![RecipeIngredient {
                ingredient_name: "flour".to_string(),
                quantity_unit: "3 cups".to_string(),
                notes: None,
            }],
        };

        create_recipe(&pool, &recipe2)
            .await
            .expect("Failed to create second recipe");

        // Flour should still only exist once in ingredients table
        let flour_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM ingredients WHERE name = ?")
                .bind("flour")
                .fetch_one(&pool)
                .await
                .expect("Failed to count flour");

        assert_eq!(
            flour_count, 1,
            "Flour ingredient should be reused, not duplicated"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_create_recipe_empty_ingredients(#[future] test_db: SqlitePool) {
        let pool = test_db.await;

        // Create a recipe with no ingredients
        let recipe = Recipe {
            id: 0,
            name: "Simple Recipe".to_string(),
            instructions: Some("Just do it".to_string()),
            created_at: String::new(),
            ingredients: vec![],
        };

        let recipe_id = create_recipe(&pool, &recipe)
            .await
            .expect("Failed to create recipe");

        // Fetch it back
        let fetched = get_recipe(&pool, recipe_id)
            .await
            .expect("Failed to fetch recipe");

        assert_eq!(fetched.name, "Simple Recipe");
        assert_eq!(fetched.ingredients.len(), 0);
    }
}

use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

use crate::error::Result;
use crate::models::RecipeRecord;
use crate::models::api::{Recipe, RecipeIngredient, ShoppingListItem};

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

    // Fetch all recipe_ingredients for this recipe with ingredient details
    // Using a JOIN to get ingredient data in a single query
    let ingredients = sqlx::query(
        r#"
        SELECT
            i.id as ingredient_id,
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
            ingredient_id: row.get("ingredient_id"),
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
/// Takes a Recipe struct (ignoring id and created_at) and links it to existing ingredients by ID
/// Ingredients must already exist in the database before creating the recipe
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

    // Insert recipe_ingredients using the provided ingredient IDs
    for ingredient in &recipe.ingredients {
        sqlx::query(
            "INSERT INTO recipe_ingredients (recipe_id, ingredient_id, quantity_unit, notes) VALUES (?, ?, ?, ?)"
        )
        .bind(recipe_id)
        .bind(ingredient.ingredient_id)
        .bind(&ingredient.quantity_unit)
        .bind(&ingredient.notes)
        .execute(&mut *tx)
        .await?;
    }

    // Commit the transaction
    tx.commit().await?;

    Ok(recipe_id)
}

/// Generate a shopping list from multiple recipes
/// Combines ingredients with the same name, concatenating their quantities
pub async fn generate_shopping_list(
    pool: &SqlitePool,
    recipe_ids: &[i64],
) -> Result<Vec<ShoppingListItem>> {
    if recipe_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Build the IN clause with placeholders
    let placeholders = recipe_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        r#"
        SELECT
            i.name as ingredient_name,
            ri.quantity_unit
        FROM recipe_ingredients ri
        JOIN ingredients i ON ri.ingredient_id = i.id
        WHERE ri.recipe_id IN ({})
        ORDER BY i.name, ri.id
        "#,
        placeholders
    );

    // Build the query and bind all recipe_ids
    let mut query_builder = sqlx::query(&query);
    for recipe_id in recipe_ids {
        query_builder = query_builder.bind(recipe_id);
    }

    let rows = query_builder.fetch_all(pool).await?;

    // Group by ingredient name and combine quantities
    let mut ingredient_map: HashMap<String, Vec<String>> = HashMap::new();

    for row in rows {
        let ingredient_name: String = row.get("ingredient_name");
        let quantity_unit: String = row.get("quantity_unit");

        ingredient_map
            .entry(ingredient_name)
            .or_insert_with(Vec::new)
            .push(quantity_unit);
    }

    // Convert to ShoppingListItem, combining quantities with " + "
    let mut shopping_list: Vec<ShoppingListItem> = ingredient_map
        .into_iter()
        .map(|(ingredient_name, quantities)| ShoppingListItem {
            ingredient_name,
            combined_quantity: quantities.join(" + "),
        })
        .collect();

    // Sort by ingredient name for consistent output
    shopping_list.sort_by(|a, b| a.ingredient_name.cmp(&b.ingredient_name));

    Ok(shopping_list)
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

        // First, create ingredients in the database
        let pasta_id = sqlx::query("INSERT INTO ingredients (name) VALUES (?)")
            .bind("pasta")
            .execute(&pool)
            .await
            .expect("Failed to insert pasta")
            .last_insert_rowid();

        let bacon_id = sqlx::query("INSERT INTO ingredients (name) VALUES (?)")
            .bind("bacon")
            .execute(&pool)
            .await
            .expect("Failed to insert bacon")
            .last_insert_rowid();

        let eggs_id = sqlx::query("INSERT INTO ingredients (name) VALUES (?)")
            .bind("eggs")
            .execute(&pool)
            .await
            .expect("Failed to insert eggs")
            .last_insert_rowid();

        // Create a recipe
        let new_recipe = Recipe {
            id: 0, // Will be ignored
            name: "Pasta Carbonara".to_string(),
            instructions: Some("Cook pasta, fry bacon, mix with eggs".to_string()),
            created_at: String::new(), // Will be ignored
            ingredients: vec![
                RecipeIngredient {
                    ingredient_id: pasta_id,
                    ingredient_name: "pasta".to_string(),
                    quantity_unit: "500g".to_string(),
                    notes: Some("spaghetti".to_string()),
                },
                RecipeIngredient {
                    ingredient_id: bacon_id,
                    ingredient_name: "bacon".to_string(),
                    quantity_unit: "200g".to_string(),
                    notes: None,
                },
                RecipeIngredient {
                    ingredient_id: eggs_id,
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

        // Create ingredient first
        let flour_id = sqlx::query("INSERT INTO ingredients (name) VALUES (?)")
            .bind("flour")
            .execute(&pool)
            .await
            .expect("Failed to insert flour")
            .last_insert_rowid();

        // Create first recipe with flour
        let recipe1 = Recipe {
            id: 0,
            name: "Pancakes".to_string(),
            instructions: None,
            created_at: String::new(),
            ingredients: vec![RecipeIngredient {
                ingredient_id: flour_id,
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

        // Create second recipe also with flour (reusing the same ingredient_id)
        let recipe2 = Recipe {
            id: 0,
            name: "Bread".to_string(),
            instructions: None,
            created_at: String::new(),
            ingredients: vec![RecipeIngredient {
                ingredient_id: flour_id,
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

    #[rstest]
    #[tokio::test]
    async fn test_generate_shopping_list_empty(#[future] test_db: SqlitePool) {
        let pool = test_db.await;

        // Generate shopping list with no recipes
        let shopping_list = generate_shopping_list(&pool, &[])
            .await
            .expect("Failed to generate shopping list");

        assert_eq!(shopping_list.len(), 0);
    }

    #[rstest]
    #[tokio::test]
    async fn test_generate_shopping_list_single_recipe(#[future] test_db: SqlitePool) {
        let pool = test_db.await;

        // Create ingredients first
        let pasta_id = sqlx::query("INSERT INTO ingredients (name) VALUES (?)")
            .bind("pasta")
            .execute(&pool)
            .await
            .expect("Failed to insert pasta")
            .last_insert_rowid();

        let sauce_id = sqlx::query("INSERT INTO ingredients (name) VALUES (?)")
            .bind("tomato sauce")
            .execute(&pool)
            .await
            .expect("Failed to insert tomato sauce")
            .last_insert_rowid();

        // Create a recipe
        let recipe = Recipe {
            id: 0,
            name: "Pasta".to_string(),
            instructions: None,
            created_at: String::new(),
            ingredients: vec![
                RecipeIngredient {
                    ingredient_id: pasta_id,
                    ingredient_name: "pasta".to_string(),
                    quantity_unit: "500g".to_string(),
                    notes: None,
                },
                RecipeIngredient {
                    ingredient_id: sauce_id,
                    ingredient_name: "tomato sauce".to_string(),
                    quantity_unit: "1 jar".to_string(),
                    notes: None,
                },
            ],
        };

        let recipe_id = create_recipe(&pool, &recipe)
            .await
            .expect("Failed to create recipe");

        // Generate shopping list
        let shopping_list = generate_shopping_list(&pool, &[recipe_id])
            .await
            .expect("Failed to generate shopping list");

        assert_eq!(shopping_list.len(), 2);

        // Check pasta
        let pasta = shopping_list
            .iter()
            .find(|item| item.ingredient_name == "pasta")
            .expect("Pasta not found");
        assert_eq!(pasta.combined_quantity, "500g");

        // Check tomato sauce
        let sauce = shopping_list
            .iter()
            .find(|item| item.ingredient_name == "tomato sauce")
            .expect("Tomato sauce not found");
        assert_eq!(sauce.combined_quantity, "1 jar");
    }

    #[rstest]
    #[tokio::test]
    async fn test_generate_shopping_list_multiple_recipes_with_shared_ingredients(
        #[future] test_db: SqlitePool,
    ) {
        let pool = test_db.await;

        // Create all ingredients first
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

        let eggs_id = sqlx::query("INSERT INTO ingredients (name) VALUES (?)")
            .bind("eggs")
            .execute(&pool)
            .await
            .expect("Failed to insert eggs")
            .last_insert_rowid();

        let sugar_id = sqlx::query("INSERT INTO ingredients (name) VALUES (?)")
            .bind("sugar")
            .execute(&pool)
            .await
            .expect("Failed to insert sugar")
            .last_insert_rowid();

        let butter_id = sqlx::query("INSERT INTO ingredients (name) VALUES (?)")
            .bind("butter")
            .execute(&pool)
            .await
            .expect("Failed to insert butter")
            .last_insert_rowid();

        // Create first recipe
        let recipe1 = Recipe {
            id: 0,
            name: "Pancakes".to_string(),
            instructions: None,
            created_at: String::new(),
            ingredients: vec![
                RecipeIngredient {
                    ingredient_id: flour_id,
                    ingredient_name: "flour".to_string(),
                    quantity_unit: "2 cups".to_string(),
                    notes: None,
                },
                RecipeIngredient {
                    ingredient_id: milk_id,
                    ingredient_name: "milk".to_string(),
                    quantity_unit: "1 cup".to_string(),
                    notes: None,
                },
                RecipeIngredient {
                    ingredient_id: eggs_id,
                    ingredient_name: "eggs".to_string(),
                    quantity_unit: "2 whole".to_string(),
                    notes: None,
                },
            ],
        };

        let recipe1_id = create_recipe(&pool, &recipe1)
            .await
            .expect("Failed to create recipe 1");

        // Create second recipe with some shared ingredients
        let recipe2 = Recipe {
            id: 0,
            name: "Cookies".to_string(),
            instructions: None,
            created_at: String::new(),
            ingredients: vec![
                RecipeIngredient {
                    ingredient_id: flour_id,
                    ingredient_name: "flour".to_string(),
                    quantity_unit: "3 cups".to_string(),
                    notes: None,
                },
                RecipeIngredient {
                    ingredient_id: sugar_id,
                    ingredient_name: "sugar".to_string(),
                    quantity_unit: "1 cup".to_string(),
                    notes: None,
                },
                RecipeIngredient {
                    ingredient_id: butter_id,
                    ingredient_name: "butter".to_string(),
                    quantity_unit: "1 stick".to_string(),
                    notes: None,
                },
            ],
        };

        let recipe2_id = create_recipe(&pool, &recipe2)
            .await
            .expect("Failed to create recipe 2");

        // Generate shopping list for both recipes
        let shopping_list = generate_shopping_list(&pool, &[recipe1_id, recipe2_id])
            .await
            .expect("Failed to generate shopping list");

        // Should have 5 unique ingredients: flour, milk, eggs, sugar, butter
        assert_eq!(shopping_list.len(), 5);

        // Check flour (should be combined)
        let flour = shopping_list
            .iter()
            .find(|item| item.ingredient_name == "flour")
            .expect("Flour not found");
        assert_eq!(flour.combined_quantity, "2 cups + 3 cups");

        // Check milk (only in pancakes)
        let milk = shopping_list
            .iter()
            .find(|item| item.ingredient_name == "milk")
            .expect("Milk not found");
        assert_eq!(milk.combined_quantity, "1 cup");

        // Check sugar (only in cookies)
        let sugar = shopping_list
            .iter()
            .find(|item| item.ingredient_name == "sugar")
            .expect("Sugar not found");
        assert_eq!(sugar.combined_quantity, "1 cup");
    }
}

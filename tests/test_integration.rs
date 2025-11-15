use feedme::controllers::{create_ingredient, create_recipe, get_recipe};
use feedme::models::api::{Recipe, RecipeIngredient};
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::test]
async fn test_create_and_get_recipe_roundtrip() {
    // Create an in-memory database with migrations
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

    // Create ingredients first
    let flour_id = create_ingredient(&pool, "flour")
        .await
        .expect("Failed to create flour");

    let sugar_id = create_ingredient(&pool, "sugar")
        .await
        .expect("Failed to create sugar");

    let chocolate_id = create_ingredient(&pool, "chocolate chips")
        .await
        .expect("Failed to create chocolate chips");

    let butter_id = create_ingredient(&pool, "butter")
        .await
        .expect("Failed to create butter");

    let eggs_id = create_ingredient(&pool, "eggs")
        .await
        .expect("Failed to create eggs");

    // Create a new recipe
    let new_recipe = Recipe {
        id: 0, // Will be ignored
        name: "Chocolate Chip Cookies".to_string(),
        instructions: Some(
            "Mix dry ingredients, add wet ingredients, bake at 350°F for 12 minutes".to_string(),
        ),
        created_at: String::new(), // Will be ignored
        ingredients: vec![
            RecipeIngredient {
                ingredient_id: flour_id,
                ingredient_name: "flour".to_string(),
                quantity_unit: "2 cups".to_string(),
                notes: Some("all-purpose".to_string()),
            },
            RecipeIngredient {
                ingredient_id: sugar_id,
                ingredient_name: "sugar".to_string(),
                quantity_unit: "1 cup".to_string(),
                notes: None,
            },
            RecipeIngredient {
                ingredient_id: chocolate_id,
                ingredient_name: "chocolate chips".to_string(),
                quantity_unit: "2 cups".to_string(),
                notes: Some("semi-sweet".to_string()),
            },
            RecipeIngredient {
                ingredient_id: butter_id,
                ingredient_name: "butter".to_string(),
                quantity_unit: "1 cup".to_string(),
                notes: Some("softened".to_string()),
            },
            RecipeIngredient {
                ingredient_id: eggs_id,
                ingredient_name: "eggs".to_string(),
                quantity_unit: "2 whole".to_string(),
                notes: None,
            },
        ],
    };

    // Create the recipe
    let recipe_id = create_recipe(&pool, &new_recipe)
        .await
        .expect("Failed to create recipe");

    assert!(recipe_id > 0, "Recipe ID should be positive");

    // Fetch the recipe back
    let fetched_recipe = get_recipe(&pool, recipe_id)
        .await
        .expect("Failed to fetch recipe");

    // Verify all fields match
    assert_eq!(fetched_recipe.id, recipe_id);
    assert_eq!(fetched_recipe.name, "Chocolate Chip Cookies");
    assert_eq!(
        fetched_recipe.instructions,
        Some("Mix dry ingredients, add wet ingredients, bake at 350°F for 12 minutes".to_string())
    );
    assert!(
        !fetched_recipe.created_at.is_empty(),
        "created_at should be set"
    );

    // Verify ingredients
    assert_eq!(fetched_recipe.ingredients.len(), 5);

    // Check flour
    let flour = &fetched_recipe.ingredients[0];
    assert_eq!(flour.ingredient_name, "flour");
    assert_eq!(flour.quantity_unit, "2 cups");
    assert_eq!(flour.notes, Some("all-purpose".to_string()));

    // Check sugar
    let sugar = &fetched_recipe.ingredients[1];
    assert_eq!(sugar.ingredient_name, "sugar");
    assert_eq!(sugar.quantity_unit, "1 cup");
    assert_eq!(sugar.notes, None);

    // Check chocolate chips
    let chocolate = &fetched_recipe.ingredients[2];
    assert_eq!(chocolate.ingredient_name, "chocolate chips");
    assert_eq!(chocolate.quantity_unit, "2 cups");
    assert_eq!(chocolate.notes, Some("semi-sweet".to_string()));

    // Check butter
    let butter = &fetched_recipe.ingredients[3];
    assert_eq!(butter.ingredient_name, "butter");
    assert_eq!(butter.quantity_unit, "1 cup");
    assert_eq!(butter.notes, Some("softened".to_string()));

    // Check eggs
    let eggs = &fetched_recipe.ingredients[4];
    assert_eq!(eggs.ingredient_name, "eggs");
    assert_eq!(eggs.quantity_unit, "2 whole");
    assert_eq!(eggs.notes, None);
}

#[tokio::test]
async fn test_create_multiple_recipes_with_shared_ingredients() {
    // Create an in-memory database with migrations
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

    // Create all ingredients first
    let flour_id = create_ingredient(&pool, "flour")
        .await
        .expect("Failed to create flour");

    let eggs_id = create_ingredient(&pool, "eggs")
        .await
        .expect("Failed to create eggs");

    let milk_id = create_ingredient(&pool, "milk")
        .await
        .expect("Failed to create milk");

    let butter_id = create_ingredient(&pool, "butter")
        .await
        .expect("Failed to create butter");

    // Create first recipe with flour and eggs
    let recipe1 = Recipe {
        id: 0,
        name: "Pancakes".to_string(),
        instructions: Some("Mix and cook on griddle".to_string()),
        created_at: String::new(),
        ingredients: vec![
            RecipeIngredient {
                ingredient_id: flour_id,
                ingredient_name: "flour".to_string(),
                quantity_unit: "2 cups".to_string(),
                notes: None,
            },
            RecipeIngredient {
                ingredient_id: eggs_id,
                ingredient_name: "eggs".to_string(),
                quantity_unit: "2 whole".to_string(),
                notes: None,
            },
            RecipeIngredient {
                ingredient_id: milk_id,
                ingredient_name: "milk".to_string(),
                quantity_unit: "1 cup".to_string(),
                notes: None,
            },
        ],
    };

    let recipe1_id = create_recipe(&pool, &recipe1)
        .await
        .expect("Failed to create first recipe");

    // Create second recipe also with flour and eggs
    let recipe2 = Recipe {
        id: 0,
        name: "Waffles".to_string(),
        instructions: Some("Mix and cook in waffle iron".to_string()),
        created_at: String::new(),
        ingredients: vec![
            RecipeIngredient {
                ingredient_id: flour_id,
                ingredient_name: "flour".to_string(),
                quantity_unit: "2.5 cups".to_string(),
                notes: None,
            },
            RecipeIngredient {
                ingredient_id: eggs_id,
                ingredient_name: "eggs".to_string(),
                quantity_unit: "3 whole".to_string(),
                notes: None,
            },
            RecipeIngredient {
                ingredient_id: butter_id,
                ingredient_name: "butter".to_string(),
                quantity_unit: "0.5 cup".to_string(),
                notes: Some("melted".to_string()),
            },
        ],
    };

    let recipe2_id = create_recipe(&pool, &recipe2)
        .await
        .expect("Failed to create second recipe");

    // Verify both recipes exist and have correct ingredients
    let fetched_recipe1 = get_recipe(&pool, recipe1_id)
        .await
        .expect("Failed to fetch first recipe");

    let fetched_recipe2 = get_recipe(&pool, recipe2_id)
        .await
        .expect("Failed to fetch second recipe");

    assert_eq!(fetched_recipe1.name, "Pancakes");
    assert_eq!(fetched_recipe1.ingredients.len(), 3);

    assert_eq!(fetched_recipe2.name, "Waffles");
    assert_eq!(fetched_recipe2.ingredients.len(), 3);

    // Verify that flour and eggs are the same ingredient records (reused)
    // Check by counting total ingredients in database
    let total_ingredients: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ingredients")
        .fetch_one(&pool)
        .await
        .expect("Failed to count ingredients");

    // Should be 4 unique ingredients: flour, eggs, milk, butter
    assert_eq!(
        total_ingredients, 4,
        "Should have 4 unique ingredients (flour, eggs, milk, butter)"
    );
}

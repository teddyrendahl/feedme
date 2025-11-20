use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use feedme::{
    controllers::{create_ingredient, create_recipe, get_all_ingredients},
    models::api::{Recipe, RecipeIngredient},
    tui::app::{AppAction, IngredientStatus, RecipeApp},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Database setup
    let database_url = "sqlite://feedme.db";

    // Create database if it doesn't exist
    if !sqlx::Sqlite::database_exists(database_url).await? {
        sqlx::Sqlite::create_database(database_url).await?;
    }

    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load ingredients as name -> id mapping
    let mut app = RecipeApp::new(
        get_all_ingredients(&pool)
            .await?
            .into_iter()
            .map(|i| (i.name, i.id))
            .collect(),
    );

    // Main loop
    let action = loop {
        // Draw UI
        terminal.draw(|f| app.render(f))?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            match app.handle_key(key.code) {
                AppAction::Continue => {}
                action @ (AppAction::SaveAndExit | AppAction::CancelAndExit) => {
                    break action;
                }
            }
        }
    };

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    // Save recipe if user finished (not cancelled)
    if matches!(action, AppAction::SaveAndExit) {
        let context = app.into_context();

        if !context.name.is_empty() {
            println!("Saving recipe: {}", context.name);

            // Create new ingredients and collect all IDs
            let mut recipe_ingredients = Vec::new();

            for (name, info) in context.ingredients {
                let ingredient_id = match info.status {
                    IngredientStatus::New => {
                        // Create new ingredient
                        create_ingredient(&pool, &name).await?
                    }
                    IngredientStatus::Existing(id) => id,
                };

                recipe_ingredients.push(RecipeIngredient {
                    ingredient_id,
                    ingredient_name: name,
                    quantity_unit: info.quantity_unit,
                    notes: if info.notes.is_empty() {
                        None
                    } else {
                        Some(info.notes)
                    },
                });
            }

            // Create recipe
            let recipe = Recipe {
                id: 0, // Ignored
                name: context.name,
                instructions: if context.instructions.is_empty() {
                    None
                } else {
                    Some(context.instructions.join("\n"))
                },
                ingredients: recipe_ingredients,
                created_at: String::new(), // Ignored
            };

            let recipe_id = create_recipe(&pool, &recipe).await?;
            println!("Recipe saved with ID: {}", recipe_id);
        } else {
            println!("No recipe name provided, not saving.");
        }
    } else {
        println!("Recipe entry cancelled.");
    }

    Ok(())
}

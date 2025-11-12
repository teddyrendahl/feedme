use thiserror::Error;

#[derive(Error, Debug)]
pub enum FeedMeError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Recipe not found with id: {0}")]
    RecipeNotFound(i64),

    #[error("Ingredient not found with id: {0}")]
    IngredientNotFound(i64),
}

pub type Result<T> = std::result::Result<T, FeedMeError>;

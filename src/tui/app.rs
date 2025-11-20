use std::collections::HashMap;

use crossterm::event::KeyCode;
use indexmap::IndexMap;
use ratatui::Frame;

use super::ingredient_states::RecipeName;

pub enum AppAction {
    Continue,      // Keep running
    SaveAndExit,   // Finished - save recipe
    CancelAndExit, // Esc pressed - don't save
}

#[derive(Copy, Clone)]
pub enum IngredientStatus {
    Existing(i64), // Has database ID
    New,           // Needs to be created
}

pub struct IngredientInfo {
    pub status: IngredientStatus,
    pub quantity_unit: String,
    pub notes: String,
}

pub struct RecipeApp {
    state: Box<dyn RecipeState>,
    context: RecipeContext,
}

pub struct RecipeContext {
    pub name: String,
    pub ingredients: IndexMap<String, IngredientInfo>,
    pub possible_ingredients: HashMap<String, i64>, // name -> id
    pub instructions: Vec<String>,
    pub finished: bool, // Set to true when ready to save
}

impl RecipeContext {
    pub fn new(possible_ingredients: HashMap<String, i64>) -> Self {
        Self {
            name: String::new(),
            ingredients: IndexMap::new(),
            // TODO: Separate prep from instructions?
            instructions: Vec::new(),
            possible_ingredients,
            finished: false,
        }
    }
}

pub(crate) trait RecipeState {
    fn render(&self, context: &RecipeContext, frame: &mut Frame);
    fn handle_key(
        &mut self,
        key: KeyCode,
        context: &mut RecipeContext,
    ) -> Option<Box<dyn RecipeState>>;
}

impl RecipeApp {
    pub fn new(possible_ingredients: HashMap<String, i64>) -> Self {
        Self {
            state: Box::new(RecipeName::new()),
            context: RecipeContext::new(possible_ingredients),
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        self.state.render(&self.context, frame);
    }

    pub fn handle_key(&mut self, key: KeyCode) -> AppAction {
        // global exit behavior
        if key == KeyCode::Esc {
            return AppAction::CancelAndExit;
        }

        // otherwise let the state handle it
        if let Some(next_state) = self.state.handle_key(key, &mut self.context) {
            self.state = next_state
        }

        // Check if recipe is finished
        if self.context.finished {
            AppAction::SaveAndExit
        } else {
            AppAction::Continue
        }
    }

    /// Consume the app and return the recipe context
    pub fn into_context(self) -> RecipeContext {
        self.context
    }
}

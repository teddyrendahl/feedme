use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::app::{IngredientInfo, IngredientStatus, RecipeContext, RecipeState};

pub(crate) struct RecipeName {
    current_input: String,
}

impl RecipeName {
    pub fn new() -> Self {
        Self {
            current_input: String::new(),
        }
    }
}

impl RecipeState for RecipeName {
    fn render(&self, _context: &RecipeContext, frame: &mut Frame) {
        let block = Paragraph::new(self.current_input.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Recipe Name (Enter to Continue)"),
        );
        frame.render_widget(block, frame.area());
    }

    fn handle_key(
        &mut self,
        key: KeyCode,
        context: &mut RecipeContext,
    ) -> Option<Box<dyn RecipeState>> {
        match key {
            KeyCode::Char(c) => {
                self.current_input.push(c);
                None
            }
            KeyCode::Backspace => {
                self.current_input.pop();
                None
            }
            KeyCode::Enter => {
                context.name = self.current_input.clone();
                self.current_input.clear();
                Some(Box::new(IngredientList::new()))
            }
            _ => None,
        }
    }
}

pub(crate) struct IngredientList {
    current_input: String,
    error_message: Option<String>,
}

impl IngredientList {
    pub fn new() -> Self {
        Self {
            current_input: String::new(),
            error_message: None,
        }
    }
}

impl RecipeState for IngredientList {
    fn render(&self, context: &RecipeContext, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(frame.area());

        let ingredient_lines: Vec<Line> = context
            .ingredients
            .iter()
            .map(|(name, info)| {
                let base_text = if info.quantity_unit.is_empty() {
                    name.to_string()
                } else {
                    format!("{} {}", info.quantity_unit, name)
                };

                if info.notes.is_empty() {
                    Line::from(base_text)
                } else {
                    Line::from(vec![
                        Span::raw(base_text),
                        Span::raw(" "),
                        Span::styled(
                            format!("({})", info.notes),
                            Style::default().add_modifier(Modifier::ITALIC),
                        ),
                    ])
                }
            })
            .collect();

        let ingredient_list = Paragraph::new(ingredient_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Ingredients for {}", context.name)),
        );
        frame.render_widget(ingredient_list, chunks[0]);

        let title = if let Some(error) = &self.error_message {
            format!("Enter ingredients for {} - ERROR: {}", context.name, error)
        } else {
            format!(
                "Enter ingredients {} (Enter on empty to continue)",
                context.name
            )
        };

        let input = Paragraph::new(self.current_input.as_str())
            .block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(input, chunks[1]);
    }

    fn handle_key(
        &mut self,
        key: KeyCode,
        context: &mut RecipeContext,
    ) -> Option<Box<dyn RecipeState>> {
        match key {
            KeyCode::Char(c) => {
                self.current_input.push(c);
                self.error_message = None; // Clear error when user types
                None
            }
            KeyCode::Backspace => {
                self.current_input.pop();
                self.error_message = None; // Clear error when user types
                None
            }
            KeyCode::Enter => {
                let ingredient_name = self.current_input.clone();

                // Onto instructions state
                if ingredient_name.is_empty() {
                    Some(Box::new(Instructions::new()))
                // Check if already in this recipe
                } else if context.ingredients.contains_key(&ingredient_name) {
                    self.error_message = Some(format!("'{}' already added", ingredient_name));
                    self.current_input.clear();
                    None
                // If the ingredient exists in the db, then move on
                } else if let Some(&ingredient_id) =
                    context.possible_ingredients.get(&ingredient_name)
                {
                    self.current_input.clear();
                    self.error_message = None;
                    Some(Box::new(IngredientQuantity::new(
                        ingredient_name,
                        IngredientStatus::Existing(ingredient_id),
                    )))
                // Otherwise, force them to confirm
                } else {
                    Some(Box::new(ConfirmIngredient::new(ingredient_name)))
                }
            }
            _ => None,
        }
    }
}

pub(crate) struct ConfirmIngredient {
    ingredient: String,
}

impl ConfirmIngredient {
    pub fn new(ingredient: String) -> Self {
        Self { ingredient }
    }
}

impl RecipeState for ConfirmIngredient {
    fn render(&self, _context: &RecipeContext, frame: &mut Frame) {
        let message = format!("Add new ingredient '{}'?\n\n(Y)es / (N)", self.ingredient);

        let block = Paragraph::new(message).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Confirm New Ingredient"),
        );
        frame.render_widget(block, frame.area());
    }

    fn handle_key(
        &mut self,
        key: KeyCode,
        _context: &mut RecipeContext,
    ) -> Option<Box<dyn RecipeState>> {
        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                // Go to quantity entry for new ingredient
                Some(Box::new(IngredientQuantity::new(
                    self.ingredient.clone(),
                    IngredientStatus::New,
                )))
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                // Reject - just go back to ingredient list
                Some(Box::new(IngredientList::new()))
            }
            _ => None, // Ignore other keys
        }
    }
}

pub(crate) struct IngredientQuantity {
    current_input: String,
    ingredient: String,
    status: IngredientStatus,
}

impl IngredientQuantity {
    pub fn new(ingredient: String, status: IngredientStatus) -> Self {
        Self {
            ingredient,
            current_input: String::new(),
            status,
        }
    }
}

impl RecipeState for IngredientQuantity {
    fn render(&self, _context: &RecipeContext, frame: &mut Frame) {
        let input = Paragraph::new(self.current_input.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Quantity for {}", self.ingredient)),
        );

        frame.render_widget(input, frame.area());
    }

    fn handle_key(
        &mut self,
        key: KeyCode,
        _context: &mut RecipeContext,
    ) -> Option<Box<dyn RecipeState>> {
        match key {
            KeyCode::Char(c) => {
                self.current_input.push(c);
                None
            }
            KeyCode::Backspace => {
                self.current_input.pop();
                None
            }
            KeyCode::Enter => {
                // Move to notes entry
                Some(Box::new(IngredientNotes::new(
                    self.ingredient.clone(),
                    self.status,
                    self.current_input.clone(),
                )))
            }
            _ => None,
        }
    }
}

pub(crate) struct IngredientNotes {
    current_input: String,
    ingredient: String,
    status: IngredientStatus,
    quantity_unit: String,
}

impl IngredientNotes {
    pub fn new(ingredient: String, status: IngredientStatus, quantity_unit: String) -> Self {
        Self {
            ingredient,
            current_input: String::new(),
            status,
            quantity_unit,
        }
    }
}

impl RecipeState for IngredientNotes {
    fn render(&self, _context: &RecipeContext, frame: &mut Frame) {
        let input = Paragraph::new(self.current_input.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Notes for {} (Enter to skip)", self.ingredient)),
        );

        frame.render_widget(input, frame.area());
    }

    fn handle_key(
        &mut self,
        key: KeyCode,
        context: &mut RecipeContext,
    ) -> Option<Box<dyn RecipeState>> {
        match key {
            KeyCode::Char(c) => {
                self.current_input.push(c);
                None
            }
            KeyCode::Backspace => {
                self.current_input.pop();
                None
            }
            KeyCode::Enter => {
                // Add ingredient to recipe with all info
                context.ingredients.insert(
                    self.ingredient.clone(),
                    IngredientInfo {
                        status: self.status,
                        quantity_unit: self.quantity_unit.clone(),
                        notes: self.current_input.clone(),
                    },
                );
                Some(Box::new(IngredientList::new()))
            }
            _ => None,
        }
    }
}

struct Instructions {
    current_input: String,
}

impl Instructions {
    pub fn new() -> Self {
        Self {
            current_input: String::new(),
        }
    }
}

impl RecipeState for Instructions {
    fn render(&self, context: &RecipeContext, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30), // Ingredients
                Constraint::Min(1),         // Instructions
                Constraint::Length(3),      // Input
            ])
            .split(frame.area());

        // Render ingredients
        let ingredient_lines: Vec<Line> = context
            .ingredients
            .iter()
            .map(|(name, info)| {
                let base_text = if info.quantity_unit.is_empty() {
                    name.to_string()
                } else {
                    format!("{} {}", info.quantity_unit, name)
                };

                if info.notes.is_empty() {
                    Line::from(base_text)
                } else {
                    Line::from(vec![
                        Span::raw(base_text),
                        Span::raw(" "),
                        Span::styled(
                            format!("({})", info.notes),
                            Style::default().add_modifier(Modifier::ITALIC),
                        ),
                    ])
                }
            })
            .collect();

        let ingredient_list = Paragraph::new(ingredient_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Ingredients for {}", context.name)),
        );
        frame.render_widget(ingredient_list, chunks[0]);

        // Render numbered instructions
        let instructions_text: String = context
            .instructions
            .iter()
            .enumerate()
            .map(|(i, step)| format!("{}. {}", i + 1, step))
            .collect::<Vec<_>>()
            .join("\n");

        let instruction_list = Paragraph::new(instructions_text)
            .block(Block::default().borders(Borders::ALL).title("Instructions"));
        frame.render_widget(instruction_list, chunks[1]);

        // Render input
        let step_num = context.instructions.len() + 1;
        let title = format!("Enter step {} (Enter on empty to finish)", step_num);

        let input = Paragraph::new(self.current_input.as_str())
            .block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(input, chunks[2]);
    }
    fn handle_key(
        &mut self,
        key: KeyCode,
        context: &mut RecipeContext,
    ) -> Option<Box<dyn RecipeState>> {
        match key {
            KeyCode::Char(c) => {
                self.current_input.push(c);
                None
            }
            KeyCode::Backspace => {
                self.current_input.pop();
                None
            }
            KeyCode::Enter => {
                let instruction = self.current_input.clone();

                if instruction.is_empty() {
                    // Finished with instructions - signal to save
                    context.finished = true;
                    None
                } else {
                    context.instructions.push(instruction);
                    self.current_input.clear();
                    None
                }
            }
            _ => None,
        }
    }
}

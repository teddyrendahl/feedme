/// Complete recipe with all ingredients for API responses
#[derive(Debug, Clone)]
pub struct Recipe {
    pub id: i64,
    pub name: String,
    pub instructions: Option<String>,
    pub ingredients: Vec<RecipeIngredient>,
    pub created_at: String,
}

/// A single ingredient within a recipe
#[derive(Debug, Clone)]
pub struct RecipeIngredient {
    pub ingredient_id: i64,
    pub ingredient_name: String,
    pub quantity_unit: String,
    pub notes: Option<String>,
}

impl Recipe {
    /// Format the recipe as a human-readable string
    pub fn to_string(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("Recipe: {}\n", self.name));
        output.push_str(&format!("ID: {}\n", self.id));
        output.push_str(&format!("Created: {}\n", self.created_at));
        output.push_str("\nIngredients:\n");

        for ingredient in &self.ingredients {
            output.push_str(&format!(
                "  - {} {}",
                ingredient.quantity_unit, ingredient.ingredient_name
            ));

            if let Some(notes) = &ingredient.notes {
                output.push_str(&format!(" ({})", notes));
            }

            output.push('\n');
        }

        if let Some(instructions) = &self.instructions {
            output.push_str(&format!("\nInstructions:\n{}\n", instructions));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_to_string_with_all_fields() {
        let recipe = Recipe {
            id: 1,
            name: "Chocolate Chip Cookies".to_string(),
            instructions: Some("Mix and bake at 350°F for 12 minutes".to_string()),
            created_at: "2024-01-15 10:30:00".to_string(),
            ingredients: vec![
                RecipeIngredient {
                    ingredient_id: 1,
                    ingredient_name: "flour".to_string(),
                    quantity_unit: "2 cups".to_string(),
                    notes: Some("all-purpose".to_string()),
                },
                RecipeIngredient {
                    ingredient_id: 2,
                    ingredient_name: "sugar".to_string(),
                    quantity_unit: "1 cup".to_string(),
                    notes: None,
                },
            ],
        };

        let output = recipe.to_string();

        assert!(output.contains("Recipe: Chocolate Chip Cookies"));
        assert!(output.contains("ID: 1"));
        assert!(output.contains("2 cups flour (all-purpose)"));
        assert!(output.contains("1 cup sugar"));
        assert!(output.contains("Mix and bake at 350°F for 12 minutes"));
    }

    #[test]
    fn test_recipe_to_string_without_instructions() {
        let recipe = Recipe {
            id: 2,
            name: "Simple Salad".to_string(),
            instructions: None,
            created_at: "2024-01-15 11:00:00".to_string(),
            ingredients: vec![RecipeIngredient {
                ingredient_id: 1,
                ingredient_name: "lettuce".to_string(),
                quantity_unit: "1 head".to_string(),
                notes: None,
            }],
        };

        let output = recipe.to_string();

        assert!(output.contains("Recipe: Simple Salad"));
        assert!(output.contains("1 head lettuce"));
        assert!(!output.contains("Instructions:"));
    }
}

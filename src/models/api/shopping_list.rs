#[derive(Debug, Clone)]
pub struct ShoppingListItem {
    pub ingredient_name: String,
    pub combined_quantity: String,
}

impl ShoppingListItem {
    pub fn to_string(&self) -> String {
        format!("{}: {}", self.ingredient_name, self.combined_quantity)
    }
}

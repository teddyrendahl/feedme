# FeedMe - Recipe Manager Architecture

## Overview
FeedMe is a recipe management system with pantry tracking, grocery list generation, and LLM-powered recipe recommendations. Built with Rust, SQLite, and MCP for AI integrations.

## Current Architecture (Local)

```
┌─────────────────┐
│  Claude Desktop │
│   or Claude Code│
└────────┬────────┘
         │
    ┌────▼────────────┐
    │  MCP Server     │  (TypeScript/Node)
    │  (feedme-mcp)   │
    └────────┬────────┘
             │ Direct SQL queries
    ┌────────▼────────┐
    │  SQLite DB      │  (feedme.db)
    │  Local file     │
    └─────────────────┘
```

### Components

**SQLite Database** (`feedme.db`)
- Local file-based database
- Migrations in `migrations/` folder
- Tables: `ingredients`, `recipes`, `recipe_ingredients`

**MCP Server** (To be built)
- Provides Claude with tools to query and manage recipes
- Direct SQLite access for fast local queries
- Tools include:
  - `search_recipes` - Find recipes by name, ingredients
  - `get_recipe` - Get full recipe details with ingredients
  - `list_pantry` - View current pantry inventory
  - `add_to_pantry` - Add/update pantry items
  - `find_makeable_recipes` - Find recipes based on pantry contents
  - `suggest_substitutes` - LLM-powered ingredient substitutions

## Database Schema

### Tables

**ingredients**
- `id` - Unique identifier
- `name` - Human-readable ingredient name (unique)
- `created_at` - Timestamp

**recipes**
- `id` - Unique identifier
- `name` - Recipe name
- `instructions` - Cooking instructions (optional)
- `created_at` - Timestamp

**recipe_ingredients**
- `id` - Unique identifier
- `recipe_id` - Foreign key to recipes
- `ingredient_id` - Foreign key to ingredients
- `quantity_unit` - Combined string (e.g., "2 cups", "1 pinch")
- `notes` - Optional notes (e.g., "diced", "fresh")
- `created_at` - Timestamp

## MVC Architecture (For Web UI)

### Models
Database structs using SQLx with `FromRow` derive:
- `Ingredient` - src/models/ingredient.rs
- `Recipe` - src/models/recipe.rs
- `RecipeIngredient` - src/models/recipe_ingredient.rs

### Controllers (Future)
Route handlers in Axum that contain business logic:
- Query database using models
- Transform data into view structs
- Return JSON responses or HTML templates

Location: `src/handlers/`

### Views (Future)
Two options being considered:

**Option 1: JSON API + HTMX**
- Response DTOs (serialize to JSON with serde)
- HTML templates (Askama/Tera)
- HTMX for dynamic interactions without heavy JavaScript
- Simple CSS framework (Pico.css)

**Option 2: Pure JSON API**
- Separate frontend (React/Vue/vanilla JS)
- Backend only returns JSON

## Design Decisions

### Quantity + Unit Storage
Currently storing as combined string (`quantity_unit` field) for flexibility:
- "2.5 cups"
- "1 pinch"
- "3 whole"
- "1 sprig"

This allows for non-standard measurements without complex unit conversion logic. Can be parsed later if grocery list aggregation needs it.

### Ingredients as Unique Entities
Each ingredient has a unique ID, allowing multiple recipes to reference the same ingredient even if named slightly differently. This enables:
- Pantry management across recipes
- Grocery list generation
- Recipe recommendations based on available ingredients

## Future Enhancements

### Web Hosting (Axum + Fly.io)

**Architecture Evolution:**
```
┌─────────────────┐         ┌─────────────────┐
│  Claude Desktop │         │   Web Browser   │
└────────┬────────┘         └────────┬────────┘
         │                           │
    ┌────▼────────────┐         ┌────▼────────────┐
    │  MCP Server     │         │   Web Frontend  │
    │  (local)        │         │   (HTML/HTMX)   │
    └────────┬────────┘         └────────┬────────┘
             │                           │
             │   HTTP requests           │
             └────────┬──────────────────┘
                      │
             ┌────────▼────────┐
             │  Axum Backend   │  (Fly.io)
             │  (feedme-api)   │
             └────────┬────────┘
                      │
             ┌────────▼────────┐
             │  PostgreSQL or  │  (Fly.io managed)
             │  SQLite Volume  │
             └─────────────────┘
```

**Migration Steps:**
1. Build Axum web server with MVC structure
2. Implement REST API endpoints
3. Add authentication/authorization
4. Deploy to Fly.io with managed database
5. Update MCP server to use HTTP requests instead of direct DB access
6. Add web UI (HTML templates + HTMX or SPA)

**Benefits:**
- Cross-device sync (shared remote database)
- Access from anywhere (web, mobile)
- Multiple clients can use same API
- Scalable architecture

### Additional Features

**Pantry Management**
- Track ingredient inventory
- Expiration date tracking
- Low-stock alerts

**Grocery Lists**
- Generate from selected recipes
- Aggregate quantities
- Integration with grocery delivery APIs

**Recipe Features**
- Rating system
- Search and filtering
- Tags/categories
- Meal planning calendar
- Nutritional information

**LLM Integrations**
- Recipe recommendations based on pantry
- Ingredient substitution suggestions
- Recipe scaling
- Dietary restriction filtering
- Parse recipes from websites/images

**Unit Conversions (Future)**
Create `units` table with:
- Standard units (metric, imperial)
- Conversion factors
- Unit types (volume, weight, count, custom)
- Enable smart grocery list aggregation

## Development Guidelines

### Testing
- Prefer unit tests over bespoke scripts
- Avoid changing directories to run commands
- Prefer running individual tests instead of complete test suites

### Database Migrations
- All schema changes go in `migrations/` folder
- Numbered sequentially (001, 002, 003...)
- Run automatically on app startup via `sqlx::migrate!()`

### Dependencies
- `sqlx` - Database access with compile-time query checking
- `tokio` - Async runtime
- Future: `axum` for web server, `serde` for JSON serialization

# OpenGP TUI - Quick Start

## Running the Application

### Option 1: Using the Run Script (Recommended)

```bash
./run-dev.sh
```

This automatically sets up development environment variables.

### Option 2: Using Environment Variables

```bash
# Set required variables
export ENCRYPTION_KEY=0000000000000000000000000000000000000000000000000000000000000000
export DATABASE_URL=sqlite:opengp.db
export LOG_LEVEL=info

# Run
cargo run
```

### Option 3: Create a .env file (with cargo-dotenv)

```bash
# Copy example env file
cp .env.example .env

# Edit .env with your values (optional)
# The defaults work for development

# Install cargo-watch with dotenv support
cargo install cargo-watch

# Run with .env loading
cargo watch -x run
```

## Keyboard Controls

Once the TUI is running:

- **`1`** - Jump to Patients screen
- **`2`** - Jump to Appointments screen
- **`3`** - Jump to Clinical screen
- **`4`** - Jump to Billing screen
- **`Tab`** - Next screen
- **`Shift+Tab`** - Previous screen
- **`q`** or **`Ctrl+C`** - Quit application

## Current Implementation Status

✅ **Implemented:**
- Event loop with render/update cycle
- Navigation between screens (Patients, Appointments, Clinical, Billing)
- Keyboard shortcuts and tab-based UI
- Component architecture (trait-based)
- Action dispatcher system
- Terminal management (enter/exit)

⏳ **Not Yet Implemented:**
- Individual component implementations (currently showing placeholders)
- Patient management features
- Appointment scheduling
- Clinical records
- Billing functionality

## Development Notes

### Architecture

The TUI follows a layered, component-based architecture:

```
UI Layer (Ratatui)
    ↓
Application Layer (App - event loop, routing)
    ↓
Components (trait-based, one per screen)
    ↓
Domain Services (business logic)
    ↓
Repository Layer (database access)
```

### Event Flow

```
User Input (Key Press)
    ↓
Event (crossterm::Event)
    ↓
App::handle_global_events() → Action
    ↓
Component::handle_events() → Action
    ↓
Action sent via channel
    ↓
App::update() processes action
    ↓
Component::update() (if component-specific)
    ↓
State changes
    ↓
Re-render on next loop
```

### Adding New Components

To implement a screen component:

1. Create struct in `src/components/{screen}/`
2. Implement `Component` trait
3. Instantiate in `App::init_components()`
4. Component automatically integrates with event loop

Example:
```rust
// src/components/patient/list.rs
pub struct PatientListComponent {
    // ... state
}

#[async_trait]
impl Component for PatientListComponent {
    async fn init(&mut self) -> Result<()> { ... }
    fn handle_key_events(&mut self, key: KeyEvent) -> Action { ... }
    async fn update(&mut self, action: Action) -> Result<Option<Action>> { ... }
    fn render(&mut self, frame: &mut Frame, area: Rect) { ... }
}
```

## Troubleshooting

### "Missing encryption key" error

**Solution:** Set the `ENCRYPTION_KEY` environment variable:
```bash
export ENCRYPTION_KEY=0000000000000000000000000000000000000000000000000000000000000000
```

Or use `./run-dev.sh` which sets it automatically.

### Database errors

**Solution:** Check that SQLite is installed and the database path is writable:
```bash
# For in-memory database (testing)
export DATABASE_URL=sqlite::memory:

# For persistent database
export DATABASE_URL=sqlite:opengp.db
```

### Terminal display issues

If the TUI doesn't render correctly:
- Ensure your terminal supports ANSI colors
- Try resizing the terminal window
- Check `TERM` environment variable is set (e.g., `xterm-256color`)

## Next Steps

See `ARCHITECTURE.md` for detailed architecture documentation.

To implement features:
1. Review `REQUIREMENTS.md` for feature requirements
2. Check `AGENTS.md` for coding guidelines
3. Follow existing patterns in `src/components/mod.rs`
4. Add tests in `tests/` directory

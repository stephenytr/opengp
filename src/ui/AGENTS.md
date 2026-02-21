# UI Layer

**Path:** `src/ui/`

## OVERVIEW

Terminal UI using Ratatui + ratatui-interact. Keyboard and mouse interface.

## SUBDIRS

| Directory | Purpose |
|-----------|---------|
| `components/` | Reusable TUI widgets |
| `services/` | UI-level services |
| `app.rs` | Main TUI application |
| `theme.rs` | Color scheme |
| `keybinds.rs` | Keyboard shortcuts |

## COMPONENTS

```
components/
├── mod.rs
├── patient/
│   ├── form.rs     # Patient form
│   ├── list.rs     # Patient list
│   ├── state.rs    # Component state
│   └── mod.rs
├── tabs.rs         # Tab navigation
├── help.rs         # Help overlay
└── status_bar.rs  # Bottom status
```

## CONVENTIONS

- Use `ratatui` for rendering
- Use `tui-realm` for component state management
- Keyboard-driven (no mouse)
- Help overlay via `?` key

## KEYBINDINGS

See `keybinds.rs` for complete list. Common:
- `?` - Toggle help
- `Tab` / `Shift+Tab` - Navigate
- `Enter` - Select
- `Esc` - Cancel/Back

## TEST CONVENTIONS

- **Integration tests**: `tests/*_test.rs` - end-to-end with in-memory DB
- **Embedded tests**: `#[cfg(test)]` modules in components
- **Test helpers**: `tests/helpers/assertions.rs` - custom field-by-field comparisons
- **Test pool**: `src/infrastructure/database/test_utils.rs` - `create_test_pool()`

## VIEW MODELS

- **Location**: `src/ui/view_models/`
- **Pattern**: MVVM with `tui-realm` state
- **Services**: `src/ui/services/` - patient_service, appointment_service

## SEE ALSO

- Parent: `/AGENTS.md`
- Domain: `/src/domain/`

# Component Lifecycle Standardization Report (Task 18)

## Completion Date
March 30, 2026

## Task Summary
Standardized component construction patterns across all UI components to ensure:
1. Explicit `new(theme: Theme, ...)` constructor pattern
2. Removal of redundant `Default` impls where constructor requires Theme
3. Consistent Theme dependency injection at construction, not render-time

## Components Audited

### 1. **PatientState** (`patient/state.rs`)
- **Constructor**: `pub fn new() -> Self` 
- **Theme Dependency**: ❌ NO Theme required (stateless container)
- **Default impl**: ✅ Appropriate (no Theme needed)
- **Status**: ✅ Already compliant

### 2. **AppointmentState** (`appointment/state.rs`)
- **Constructor**: `pub fn new(theme: Theme, config: CalendarConfig) -> Self`
- **Theme Dependency**: ✅ Takes Theme at construction
- **Default impl**: ❌ None (correct - Theme required)
- **Status**: ✅ Already compliant

### 3. **ClinicalState** (`clinical/state.rs`)
- **Constructor**: `pub fn new(theme: Theme) -> Self` + `pub fn with_theme(theme: Theme) -> Self`
- **Theme Dependency**: ✅ Takes Theme at construction
- **Default impl**: ❌ None (correct - Theme required)
- **Status**: ✅ Already compliant

### 4. **HelpOverlay** (`help.rs`)
- **Constructor**: `pub fn new(theme: Theme) -> Self`
- **Theme Dependency**: ✅ Takes Theme at construction
- **Default impl**: ❌ REMOVED (was redundant)
- **Change Made**: Removed `impl Default for HelpOverlay`
- **Status**: ✅ Fixed

### 5. **TabBar** (`tabs.rs`)
- **Constructor**: `pub fn new(theme: Theme) -> Self`
- **Theme Dependency**: ✅ Takes Theme at construction
- **Default impl**: ❌ REMOVED (was redundant)
- **Change Made**: Removed `impl Default for TabBar`
- **Status**: ✅ Fixed

### 6. **StatusBar** (`status_bar.rs`)
- **Constructor**: `pub fn new(theme: Theme) -> Self`
- **Theme Dependency**: ✅ Takes Theme at construction
- **Default impl**: ❌ REMOVED (derive Default was removed)
- **Change Made**: Removed `#[derive(Debug, Clone, Default)]`, kept `#[derive(Debug, Clone)]`
- **Status**: ✅ Fixed

## Changes Applied

### Files Modified
1. **crates/opengp-ui/src/ui/components/help.rs**
   - Removed: `impl Default for HelpOverlay { fn default() -> Self { ... } }`

2. **crates/opengp-ui/src/ui/components/tabs.rs**
   - Removed: `impl Default for TabBar { fn default() -> Self { ... } }`

3. **crates/opengp-ui/src/ui/components/status_bar.rs**
   - Changed: `#[derive(Debug, Clone, Default)]` → `#[derive(Debug, Clone)]`

### Why These Changes?
- **HelpOverlay** requires Theme for rendering (colors, styles)
- **TabBar** requires Theme for rendering (colors, styles)
- **StatusBar** requires Theme for rendering (colors, styles)

Default impls that create Theme::dark() bypassed explicit dependency injection, making it easy to accidentally create components with wrong theme context.

## Verification
✅ All tests pass (493 passed; 0 failed)
✅ No compiler errors (check completed successfully)
✅ No uses of removed Default impls found in non-test code
✅ Component tests in all affected files still pass

## Standardized Pattern

All Theme-dependent components now follow this pattern:

```rust
pub struct ComponentName {
    theme: Theme,
    // ... other fields
}

impl ComponentName {
    /// Explicit constructor requiring Theme
    pub fn new(theme: Theme, /* other required params */) -> Self {
        Self {
            theme,
            // ...
        }
    }
    
    // No Default impl - forces explicit Theme provision
}
```

## Acceptance Criteria
✅ All components use consistent constructor pattern
✅ No Default impl on components that require Theme
✅ All tests pass with no regressions
✅ Build succeeds without warnings related to this change

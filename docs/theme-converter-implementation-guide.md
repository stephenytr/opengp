# OpenGP Theme Converter (Beginner Implementation Guide)

This guide walks you step-by-step through building a **fully functional Alacritty -> OpenGP theme converter**.

It is written for beginners who can run Cargo commands and edit Rust files, even if you have not built a converter before.

---

## 1) What you are building

You are building a CLI tool that:

1. Reads an Alacritty theme file (`.toml`, `.yml`, or `.yaml`)
2. Maps Alacritty colors into OpenGP semantic theme tokens
3. Writes a valid OpenGP theme TOML file
4. Warns when values are missing/ambiguous
5. Checks basic accessibility contrast

### Why this design?

- OpenGP keeps a stable internal format (semantic TOML)
- Users can import popular community themes easily
- You can improve import logic over time without changing OpenGP runtime theme loading

---

## 2) Project structure (scaffolded)

The converter lives in:

```text
crates/opengp-theme-converter/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── error.rs
│   ├── output.rs
│   ├── parse/
│   │   ├── mod.rs
│   │   ├── toml_parser.rs
│   │   └── yaml_parser.rs
│   ├── mapping/
│   │   ├── mod.rs
│   │   ├── color.rs
│   │   └── mapper.rs
│   └── validation/
│       ├── mod.rs
│       ├── contrast.rs
│       └── fallbacks.rs
└── tests/
    └── golden/
        └── mod.rs
```

---

## 3) High-level architecture

Pipeline:

```text
Input file -> Parse -> Normalize colors -> Map semantic tokens -> Apply fallbacks -> Validate -> Emit TOML
```

Use these responsibilities:

- `parse/*`: read Alacritty source files into Rust structs
- `mapping/color.rs`: parse hex colors, luminance helpers, color conversion
- `mapping/mapper.rs`: mapping rules from Alacritty to OpenGP tokens
- `validation/fallbacks.rs`: how missing tokens are derived
- `validation/contrast.rs`: contrast checks and warning generation
- `output.rs`: deterministic OpenGP TOML output
- `error.rs`: typed errors and warnings

---

## 4) Step-by-step implementation plan

Follow these steps in order.

### Step 1: Define shared models

In `src/lib.rs` or new `src/model.rs` (your choice), define:

- `AlacrittyTheme`
- `AlacrittyColors`
- `Ansi8`
- `OpenGPTheme`
- `OpenGPPalette`

Keep model structs serializable:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

Tip: use `Option<T>` for Alacritty fields that are often omitted.

---

### Step 2: Implement TOML + YAML parsing

#### `parse/toml_parser.rs`

- Add function: `pub fn parse_alacritty_toml(input: &str) -> Result<AlacrittyTheme, ThemeConverterError>`
- Use `toml::from_str`

#### `parse/yaml_parser.rs`

- Add function: `pub fn parse_alacritty_yaml(input: &str) -> Result<AlacrittyTheme, ThemeConverterError>`
- Use `serde_yaml::from_str`

#### `parse/mod.rs`

- Add dispatcher:

```rust
pub fn parse_by_extension(path: &Path, content: &str) -> Result<AlacrittyTheme, ThemeConverterError>
```

Supported extensions:

- `.toml`
- `.yml`
- `.yaml`

Return a clear error for unsupported extensions.

---

### Step 3: Implement color helpers (`mapping/color.rs`)

Create helpers you will use everywhere:

- `parse_hex("#RRGGBB") -> RgbColor`
- `to_opengp_string(RgbColor) -> String` (for example `"Rgb(34, 56, 78)"`)
- `relative_luminance(RgbColor) -> f32`
- `contrast_ratio(fg, bg) -> f32`
- `adjust_brightness(color, factor)`

Keep this module very small and pure (no file I/O).

---

### Step 4: Implement mapping rules (`mapping/mapper.rs`)

Write one mapper entrypoint:

```rust
pub fn map_alacritty_to_opengp(input: &AlacrittyTheme) -> MappingResult
```

Where `MappingResult` includes:

- `theme: OpenGPTheme`
- `warnings: Vec<ConversionWarning>`

Recommended initial mapping table:

- `background <- primary.background`
- `foreground <- primary.foreground`
- `primary <- bright.blue` (or foreground if you prefer consistency)
- `secondary <- bright.magenta`
- `error <- normal.red`
- `warning <- normal.yellow`
- `success <- normal.green`
- `info <- normal.blue`
- `highlight <- bright.cyan`
- `selected <- bright.blue`

For appointment status fields, map to the nearest semantic meaning and document decisions.

---

### Step 5: Add fallback logic (`validation/fallbacks.rs`)

Create reusable fallback helpers:

- if `dim` colors missing -> derive from normal with brightness reduction
- if token missing -> derive from foreground/background based on role
- if still missing -> use deterministic safe default and add warning

Never silently drop missing fields.

---

### Step 6: Add contrast checks (`validation/contrast.rs`)

At minimum check:

- `foreground` vs `background`
- `text_secondary` vs `background`
- `disabled` vs `background`

Recommended thresholds:

- warning if ratio < `4.5`
- critical warning if ratio < `3.0`

Do not block generation by default. Report warnings and still emit output (unless `--strict`).

---

### Step 7: Emit deterministic TOML (`output.rs`)

Add function:

```rust
pub fn render_opengp_toml(theme: &OpenGPTheme) -> Result<String, ThemeConverterError>
```

Rules:

- stable field order
- stable section order
- newline at EOF

Deterministic output makes tests easy and diffs clean.

---

### Step 8: CLI commands (`main.rs`)

Use `clap` derive to expose:

- `convert <INPUT> --output <OUTPUT>`
- `validate <FILE>`
- `batch <DIR> --output-dir <DIR> [--recursive]`

Optional flags:

- `--strict` (warnings become errors)
- `--target dark|light|all`
- `--verbose`

User experience goal: one-command import for beginners.

---

## 5) Suggested command examples

```bash
# Convert one theme
cargo run -p opengp-theme-converter -- convert ./rose-pine.toml --output ~/.config/opengp/themes/rose-pine.toml

# Validate generated file
cargo run -p opengp-theme-converter -- validate ~/.config/opengp/themes/rose-pine.toml

# Batch convert all themes in a folder
cargo run -p opengp-theme-converter -- batch ./alacritty-themes --output-dir ./converted
```

---

## 6) Testing strategy (must-have)

### A) Golden tests (`tests/golden/mod.rs`)

For each sample input theme:

1. run converter
2. compare output to checked-in expected TOML

Start with:

- Catppuccin
- Rosé Pine
- Nord
- Dracula

### B) Unit tests

- hex parse tests
- luminance/contrast tests
- fallback tests for missing fields

### C) Error-path tests

- unsupported extension
- malformed TOML/YAML
- empty/invalid color values

---

## 7) Definition of done

You are done when all are true:

1. Converter imports `.toml` and `.yaml` Alacritty themes
2. Output TOML is valid for OpenGP and deterministic
3. Missing values generate explicit warnings
4. Contrast checks run and report issues
5. Golden tests pass
6. Beginner can run one command from docs and get a working theme

---

## 8) Common beginner pitfalls

1. **Hardcoding everything in `main.rs`**  
   Keep logic in modules and keep `main.rs` thin.

2. **Not handling missing optional fields**  
   Alacritty files vary a lot; always use `Option` + fallbacks.

3. **Nondeterministic output**  
   Sort and order output consistently.

4. **No warning channel**  
   Users need to know when mappings are approximate.

5. **Mixing parse errors and mapping errors**  
   Keep errors typed so troubleshooting is obvious.

---

## 9) Implementation order (quick checklist)

- [ ] Add models for source + target themes
- [ ] Implement TOML parser
- [ ] Implement YAML parser
- [ ] Implement extension-based parse dispatcher
- [ ] Implement color helper utilities
- [ ] Implement base mapping table
- [ ] Implement fallback rules
- [ ] Implement contrast checks
- [ ] Implement deterministic TOML renderer
- [ ] Implement CLI subcommands
- [ ] Add golden tests + unit tests
- [ ] Add README/docs usage examples

---

## 10) Final note

Keep OpenGP's semantic theme format as the canonical source of truth. The converter exists to make adoption easy, not to leak external format complexity into app runtime.

That principle keeps long-term maintenance simple and makes theme contributions from the community much easier to review.

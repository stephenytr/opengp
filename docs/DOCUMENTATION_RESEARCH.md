# OpenGP Documentation Rewrite: Research & Best Practices

**Research Date:** March 11, 2026  
**Project:** OpenGP - Australian General Practice Management Software (Rust TUI)

---

## 1. README.md Structure for Rust Open-Source Projects

### Essential Sections (Based on Tokio & Serde Analysis)

#### 1.1 Header & Badges
- **Project name** (prominent, clear)
- **One-line description** (what it does, not how)
- **Status badges**: Build status, license, version, MSRV (Minimum Supported Rust Version)
- **Quick links**: Website, docs, API docs, chat/community

**Example from Tokio:**
```markdown
# Tokio
A runtime for writing reliable, asynchronous, and slim applications with Rust.
- **Fast**: Zero-cost abstractions
- **Reliable**: Leverages Rust's ownership and type system
- **Scalable**: Minimal footprint, handles backpressure naturally

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[Website](https://tokio.rs) | [Guides](https://tokio.rs/tokio/tutorial) | [API Docs](https://docs.rs/tokio)
```

#### 1.2 Overview Section
- **What problem does it solve?** (2-3 sentences)
- **Key components** (bullet list of major features)
- **Architecture overview** (high-level, not implementation details)

#### 1.3 Quick Start / Getting Started
- **Installation instructions** (copy-paste ready)
- **Minimal working example** (5-10 lines of code)
- **Link to full examples** (point to `/examples` directory)

**For OpenGP TUI:**
```markdown
## Quick Start

### Installation
```bash
cargo install opengp
# or build from source
git clone https://github.com/your-org/opengp
cd opengp
cargo build --release
./target/release/opengp
```

### First Run
- Launch the app: `opengp`
- Navigate with arrow keys
- Press `?` for help
- See [examples/](examples/) for detailed workflows
```

#### 1.4 Documentation Links
- **Guides/Tutorials** (link to wiki or docs/)
- **API Documentation** (link to docs.rs)
- **Contributing** (link to CONTRIBUTING.md)
- **Getting Help** (Discord, discussions, issues)

#### 1.5 Features / Capabilities
- **Comprehensive feature list** (what makes it unique)
- **For healthcare apps**: Compliance features (HIPAA, Australian privacy, audit logging)
- **For TUI apps**: Platform support (Linux, macOS, Windows)

#### 1.6 Project Status & Roadmap
- **Current version** and release schedule
- **MSRV policy** (e.g., "6 months rolling")
- **LTS releases** (if applicable)
- **Known limitations** (be honest about stubs/incomplete features)

#### 1.7 Contributing
- **Link to CONTRIBUTING.md** (don't duplicate)
- **Code of conduct**
- **Development setup** (brief; full details in AGENTS.md)

#### 1.8 License & Legal
- **License type** (MIT, Apache-2.0, etc.)
- **Contribution licensing** (dual-license statement if applicable)

### Minimal vs. Comprehensive README

**Minimal README** (for small projects):
- Header + badges
- One-line description
- Quick start (3-5 lines)
- Link to docs
- License

**Comprehensive README** (for OpenGP):
- All sections above
- Feature matrix (if multiple variants)
- Architecture diagram (ASCII or link to wiki)
- Performance benchmarks (if relevant)
- Compliance/security highlights
- Related projects

---

## 2. AGENTS.md / AI Agent Knowledge Base Files

### What AGENTS.md Is

**AGENTS.md** is a **README for AI coding agents** — a vendor-neutral markdown file that tells AI tools (Claude, Cursor, GitHub Copilot, Codex, etc.) how to work effectively in your codebase.

**Key Difference from README.md:**
- README: For humans (quick start, project description, contribution guidelines)
- AGENTS.md: For AI agents (build steps, test commands, code conventions, project structure, gotchas)

### Standard AGENTS.md Sections

#### 2.1 Project Overview
```markdown
## Project Overview
- **Purpose**: Australian general practice management system
- **Language**: Rust
- **Framework**: Ratatui (TUI)
- **Database**: SQLite (with PostgreSQL migration path)
- **Key Features**: Patient management, appointments, clinical notes, e-prescribing, audit logging
- **Compliance**: Australian healthcare standards (Medicare, PBS, AIR integration stubs)
```

#### 2.2 Setup Commands
```markdown
## Setup Commands
- Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Install dependencies: `cargo build`
- Run app: `cargo run --release`
- Run tests: `cargo test`
- Run specific test: `cargo test --test integration_test`
- Format code: `cargo fmt`
- Lint: `cargo clippy`
- Build docs: `cargo doc --open`
```

#### 2.3 Code Style & Conventions
```markdown
## Code Style
- **Naming**: `snake_case` for files/functions, `PascalCase` for types
- **Error handling**: Use `thiserror` + `color-eyre`
- **Async**: `tokio::test` for async tests
- **Architecture**: Domain/Infrastructure/UI layers (Clean Architecture)
- **Repository pattern**: Trait in domain, impl in infrastructure
- **Testing**: TDD approach — write tests first
- **No**: `rm` commands without explicit permission, worktrees, `.env` files in commits
```

#### 2.4 Project Structure
```markdown
## Project Structure
```
opengp/
├── src/                          # Source code (monolithic, migration to workspace in progress)
│   ├── domain/                   # Business logic (patient, clinical, billing, etc.)
│   ├── infrastructure/           # Database, auth, crypto, fixtures
│   ├── integrations/             # Medicare/PBS/AIR stubs
│   └── ui/                       # Ratatui TUI components
├── crates/                       # Workspace crates (actual implementation)
│   ├── opengp-domain/
│   ├── opengp-infrastructure/
│   ├── opengp-ui/
│   └── opengp-config/
├── tests/                        # Integration tests
├── migrations/                   # SQL schema
├── wiki/                         # Git-backed documentation
└── AGENTS.md                     # This file
```
**Note**: Dual structure — code in `/src/` but workspace crates in `/crates/` contain actual implementation.
```

#### 2.5 Testing Instructions
```markdown
## Testing
- Run all tests: `cargo test`
- Run with output: `cargo test -- --nocapture`
- Run specific module: `cargo test domain::patient`
- Run integration tests: `cargo test --test '*_test'`
- Watch mode: `cargo watch -x test`
- Coverage: `cargo tarpaulin --out Html`
- **Important**: All tests must pass before committing
```

#### 2.6 Build & Deployment
```markdown
## Build & Deployment
- Debug build: `cargo build`
- Release build: `cargo build --release` (8.1MB binary)
- Run release: `./target/release/opengp`
- **MSRV**: Rust 1.70+
- **Platforms**: Linux, macOS, Windows (tested on Linux)
```

#### 2.7 Key Modules & Symbols
```markdown
## Key Modules & Symbols
| Symbol | Type | Location | Role |
|--------|------|----------|------|
| `Patient` | Model | `src/domain/patient/model.rs` | Core entity |
| `Appointment` | Model | `src/domain/appointment/model.rs` | Scheduling |
| `Consultation` | Model | `src/domain/clinical/model.rs` | Clinical visit |
| `ClinicalRepository` | Trait | `src/domain/clinical/repository.rs` | Data access |
| `AuditService` | Service | `src/domain/audit/service.rs` | Audit logging |
| `App` | Struct | `src/ui/app.rs` | TUI application |
```

#### 2.8 Common Gotchas & Anti-Patterns
```markdown
## Anti-Patterns (DO NOT DO)
- ❌ NEVER commit `.env` files or API keys
- ❌ NEVER use `rm` without explicit permission
- ❌ NEVER revert git changes without explicit permission
- ⚠️ `password_hash` field exists but unused (no bcrypt/argon2 yet)
- ⚠️ `EncryptionService` exists but unused in domain
- ⚠️ Medicare/PBS/AIR integrations are empty stubs
- ⚠️ Audit log has no hash chain (tamperable)
- ⚠️ No MFA/TOTP implementation despite audit flags
```

#### 2.9 Healthcare-Specific Context
```markdown
## Australian Healthcare Context
- **Medicare**: MBS (Medical Benefits Schedule) integration stub
- **PBS**: Pharmaceutical Benefits Scheme integration stub
- **AIR**: Australian Immunisation Register integration stub
- **Compliance**: Audit logging for all patient data access
- **Privacy**: Patient data encrypted at rest (AES-GCM)
- **Standards**: FHIR-ready (not yet implemented)
```

#### 2.10 Git & Commit Conventions
```markdown
## Git Conventions
- **Commit format**: `[module] Brief description` (e.g., `[patient] Add date-of-birth validation`)
- **Branch naming**: `feature/patient-search`, `fix/audit-logging-bug`
- **PR format**: Link to issue, describe changes, list tests added
- **Squash commits**: Yes, before merging
- **Update AGENTS.md**: When adding new conventions or modules
```

### AGENTS.md Best Practices

1. **Keep it concise** — 200-400 lines max
2. **Use tables** for symbol references and module mappings
3. **Link to files** — Use relative paths (`src/domain/patient/model.rs`)
4. **Be specific** — "Run `cargo test --test integration_test`" not "run tests"
5. **Update regularly** — Treat it as living documentation
6. **Nested AGENTS.md** — For monorepos, add AGENTS.md in each crate
7. **Version it** — Commit to git, review in PRs

### Tools That Support AGENTS.md (2026)

- ✅ Claude Code / Claude AI
- ✅ Cursor
- ✅ GitHub Copilot (Coding Agent)
- ✅ OpenAI Codex
- ✅ Google Jules
- ✅ Aider
- ✅ Zed AI
- ✅ VS Code AI
- ✅ Windsurf (Cognition)
- ✅ Devin (Cognition)
- ✅ 60k+ open-source projects use it

---

## 3. Wiki Integration Guides for Healthcare Software

### Standard Wiki Structure for Healthcare Apps

#### 3.1 Developer Integration Guide Sections

**A. Getting Started**
- System requirements (OS, Rust version, database)
- Installation steps (clone, build, run)
- First-time setup (database initialization, config)
- Troubleshooting common issues

**B. Architecture Overview**
- High-level system diagram (ASCII or Mermaid)
- Layer breakdown (Domain → Infrastructure → UI)
- Data flow (patient data → clinical → audit)
- External integrations (Medicare, PBS, AIR)

**C. Domain Models**
- Patient entity (demographics, contact, medical history)
- Appointment entity (scheduling, status, notes)
- Consultation entity (clinical visit, SOAP notes)
- Prescription entity (e-prescribing, PBS integration)
- Audit entity (compliance logging)

**D. API Reference**
- Repository traits (ClinicalRepository, PatientRepository)
- Service interfaces (ClinicalService, AuditService)
- Error types (RepositoryError, DomainError)
- Example usage (code snippets)

**E. Database Schema**
- Table definitions (patients, appointments, consultations)
- Relationships (foreign keys, constraints)
- Migrations (how to run, how to write new ones)
- Backup/restore procedures

**F. Testing Guide**
- Unit test patterns (mocking repositories)
- Integration test setup (test database)
- Running tests locally
- CI/CD pipeline (GitHub Actions, etc.)

**G. Deployment Guide**
- Building release binary
- Configuration (environment variables)
- Database setup (production)
- Monitoring & logging
- Backup strategy

**H. Healthcare Compliance**
- Audit logging requirements
- Patient privacy (encryption, access control)
- Data retention policies
- Regulatory compliance (Australian healthcare standards)

**I. Integration Guides**
- Medicare MBS integration (when implemented)
- PBS integration (when implemented)
- AIR integration (when implemented)
- FHIR standards (when implemented)

**J. Troubleshooting**
- Common errors and solutions
- Performance tuning
- Database optimization
- Debugging tips

#### 3.2 Example Wiki Structure (Markdown)

```
wiki/
├── 01-getting-started.md
├── 02-architecture.md
├── 03-domain-models.md
├── 04-api-reference.md
├── 05-database-schema.md
├── 06-testing-guide.md
├── 07-deployment.md
├── 08-healthcare-compliance.md
├── 09-integrations/
│   ├── medicare-mbs.md
│   ├── pbs.md
│   └── air.md
├── 10-troubleshooting.md
└── README.md (index)
```

#### 3.3 Healthcare-Specific Wiki Sections

**For Australian GP Software:**

1. **Medicare Integration**
   - MBS item numbers and descriptions
   - Claiming workflow
   - Rebate calculations
   - Bulk billing vs. private billing

2. **PBS Integration**
   - Pharmaceutical Benefits Scheme lookup
   - Prescription authority requirements
   - Restricted medicines
   - Generic substitution rules

3. **AIR Integration**
   - Immunisation schedule (NIP)
   - Vaccination recording
   - Adverse event reporting
   - Compliance requirements (mandatory reporting from Oct 2025)

4. **Privacy & Compliance**
   - Australian Privacy Principles (APPs)
   - My Health Record integration
   - Patient consent management
   - Data breach notification

5. **Clinical Workflows**
   - Patient encounter workflow
   - SOAP notes structure
   - Referral management
   - Pathology ordering

---

## 4. Australian Healthcare Software Documentation Standards

### Key Standards & Frameworks

#### 4.1 FHIR (Fast Healthcare Interoperability Resources)
- **Standard**: HL7 FHIR (Fast Healthcare Interoperability Resources)
- **Australian Implementation**: Medicare Records FHIR Implementation Guide (v2.2.0)
- **Use Cases**:
  - MBS (Medical Benefits Schedule) claims
  - PBS (Pharmaceutical Benefits Schedule) claims
  - Patient data exchange
  - My Health Record integration

**Documentation Pattern:**
```markdown
## FHIR Resources
- **ExplanationOfBenefit**: MBS/PBS claims
- **Patient**: Demographics
- **Encounter**: Clinical visit
- **Medication**: Prescription details
- **Immunization**: Vaccination records
```

#### 4.2 Medicare Integration Documentation
- **MBS Item Numbers**: Reference table of billable services
- **Claiming Workflow**: Step-by-step claiming process
- **Rebate Calculation**: How rebates are determined
- **Bulk Billing**: Requirements and compliance

**Documentation Pattern:**
```markdown
## Medicare MBS Integration
### Supported Item Numbers
| Item | Description | Rebate | Notes |
|------|-------------|--------|-------|
| 23 | GP consultation (standard) | $38.75 | Bulk billing eligible |
| 36 | GP consultation (extended) | $58.15 | Complex cases |

### Claiming Workflow
1. Record consultation (SOAP notes)
2. Select MBS item number
3. Generate claim
4. Submit to Medicare
5. Track rebate status
```

#### 4.3 PBS Integration Documentation
- **PBS Schedule**: Current list of subsidised medicines
- **Authority Requirements**: When prescriber authority needed
- **Restricted Medicines**: Special conditions for prescribing
- **Generic Substitution**: Rules for generic alternatives

**Documentation Pattern:**
```markdown
## PBS Integration
### PBS Schedule API
- **Endpoint**: PBS Schedule Data API
- **Update Frequency**: Monthly (1st of month)
- **Data Format**: JSON, CSV, XML
- **Authentication**: API key required

### Prescription Authority
- **When Required**: For restricted medicines
- **How to Request**: Online via PBS website
- **Approval Time**: Usually immediate
```

#### 4.4 AIR (Australian Immunisation Register) Integration
- **Vaccination Recording**: How to record immunisations
- **Compliance**: Mandatory reporting requirements (from Oct 2025)
- **Schedules**: NIP (National Immunisation Program) schedule
- **Adverse Events**: Reporting adverse events

**Documentation Pattern:**
```markdown
## AIR Integration
### Mandatory Reporting (from 24 Oct 2025)
- All COVID-19 vaccines: Mandatory
- Seasonal flu: Mandatory (from 1 Mar)
- NIP vaccines: Mandatory (from 1 Jul)

### Recording Workflow
1. Select vaccine from AIR schedule
2. Record administration date
3. Record provider details
4. Submit to AIR
5. Receive confirmation

### Antenatal Indicator
- New field for vaccinations during pregnancy
- Improves monitoring of maternal immunisation
```

#### 4.5 My Health Record Integration
- **Patient Consent**: How to obtain and manage consent
- **Data Sharing**: What data can be shared
- **Privacy Controls**: Patient privacy settings
- **Access Logging**: Audit trail of access

### Documentation Structure for Healthcare Apps

**Recommended Sections:**
1. **Compliance Overview** (regulatory requirements)
2. **Integration Guides** (Medicare, PBS, AIR, My Health Record)
3. **Data Standards** (FHIR, HL7, SNOMED CT)
4. **Privacy & Security** (encryption, audit logging, access control)
5. **Workflow Documentation** (clinical workflows, user journeys)
6. **API Reference** (if exposing APIs)
7. **Troubleshooting** (common issues, error codes)

---

## 5. Ratatui TUI Framework Documentation Patterns

### Ratatui Architecture Patterns

#### 5.1 The Elm Architecture (TEA)
Ratatui recommends The Elm Architecture for organizing TUI applications:

```
Model (State)
    ↓
Update (Event → Model → Model)
    ↓
View (Model → UI)
    ↓
Display
```

**Documentation Pattern:**
```markdown
## Architecture: The Elm Architecture (TEA)

### Model
- Holds all application state
- Immutable (new state created on each update)
- Example: `struct App { patients: Vec<Patient>, selected: usize }`

### Update
- Handles user input and events
- Takes current model + event → new model
- Example: `fn update(app: &mut App, event: Event) { ... }`

### View
- Renders model to terminal UI
- Uses Ratatui widgets (Paragraph, Table, List, etc.)
- Example: `fn view(app: &App, frame: &mut Frame) { ... }`
```

#### 5.2 Component Architecture
Alternative to TEA for larger apps:

```
Component Trait
├── init() — Initialize state
├── handle_events() — Process input
├── render() — Draw to screen
└── update() — Update internal state
```

**Documentation Pattern:**
```markdown
## Component Architecture

### PatientListComponent
- **State**: `patients: Vec<Patient>`, `selected: usize`
- **Events**: `Up`, `Down`, `Select`, `Delete`
- **Render**: Table widget with patient list
- **Update**: Modify state based on events

### ConsultationComponent
- **State**: `consultation: Consultation`, `editing: bool`
- **Events**: `Edit`, `Save`, `Cancel`
- **Render**: Form with SOAP notes
- **Update**: Validate and save consultation
```

#### 5.3 Layout & Rendering
Ratatui uses a constraint-based layout system:

**Documentation Pattern:**
```markdown
## Layout System

### Constraints
- `Length(n)` — Fixed size
- `Percentage(n)` — Percentage of available space
- `Ratio(num, denom)` — Ratio of available space
- `Min(n)` — Minimum size
- `Max(n)` — Maximum size

### Example: Two-Column Layout
```rust
let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
    .split(frame.area());

frame.render_widget(patient_list, chunks[0]);
frame.render_widget(consultation_form, chunks[1]);
```
```

#### 5.4 Event Handling
Ratatui uses crossterm for terminal events:

**Documentation Pattern:**
```markdown
## Event Handling

### Event Types
- `Key(KeyEvent)` — Keyboard input
- `Mouse(MouseEvent)` — Mouse input
- `Resize(width, height)` — Terminal resize
- `Paste(String)` — Clipboard paste

### Example: Keyboard Navigation
```rust
match event {
    Event::Key(KeyEvent { code: KeyCode::Up, .. }) => {
        app.selected = app.selected.saturating_sub(1);
    }
    Event::Key(KeyEvent { code: KeyCode::Down, .. }) => {
        app.selected = (app.selected + 1).min(app.patients.len() - 1);
    }
    _ => {}
}
```
```

#### 5.5 Widgets & Styling
Ratatui provides built-in widgets:

**Documentation Pattern:**
```markdown
## Widgets

### Common Widgets
- `Paragraph` — Text display
- `Table` — Tabular data
- `List` — Scrollable list
- `Block` — Container with border
- `Gauge` — Progress bar
- `Chart` — Line/bar charts
- `Popup` — Modal dialog

### Styling
- `Style` — Color, bold, italic, underline
- `Color` — 16 colors + RGB
- `Modifier` — Bold, italic, underline, dim, crossed out

### Example: Styled Table
```rust
let header = Row::new(vec!["Name", "Age", "Status"])
    .style(Style::default().fg(Color::Yellow).bold());

let rows = app.patients.iter().map(|p| {
    Row::new(vec![p.name.clone(), p.age.to_string(), p.status.clone()])
});

let table = Table::new(rows)
    .header(header)
    .block(Block::default().title("Patients").borders(Borders::ALL));
```
```

#### 5.6 Testing Ratatui Components
**Documentation Pattern:**
```markdown
## Testing

### Unit Tests
- Test `update()` function with various events
- Verify state changes
- No rendering needed

### Integration Tests
- Test full component lifecycle
- Verify rendering output
- Use `Buffer` for assertions

### Example: Testing Patient Selection
```rust
#[test]
fn test_patient_selection() {
    let mut app = App::new();
    app.patients = vec![patient1, patient2, patient3];
    app.selected = 0;
    
    app.update(Event::Key(KeyCode::Down));
    assert_eq!(app.selected, 1);
    
    app.update(Event::Key(KeyCode::Down));
    assert_eq!(app.selected, 2);
}
```
```

### Ratatui Documentation Structure for OpenGP

**Recommended Sections:**
1. **Architecture Overview** (TEA vs. Component pattern)
2. **Layout System** (constraints, splitting screen)
3. **Widgets** (which widgets used, custom widgets)
4. **Event Handling** (keyboard, mouse, resize)
5. **Styling** (colors, themes, accessibility)
6. **Performance** (rendering optimization, large datasets)
7. **Testing** (unit tests, integration tests)
8. **Examples** (common patterns, workflows)

---

## Summary: Documentation Rewrite Checklist for OpenGP

### Phase 1: README.md
- [ ] Add project name, badges, quick links
- [ ] Write one-line description + key features
- [ ] Add quick start (installation + minimal example)
- [ ] Link to wiki, API docs, contributing guide
- [ ] Add healthcare compliance highlights
- [ ] Add MSRV and platform support
- [ ] Add related projects section

### Phase 2: AGENTS.md
- [ ] Create AGENTS.md at project root
- [ ] Add project overview (purpose, language, framework)
- [ ] Document setup commands (build, test, run)
- [ ] Document code style (naming, error handling, architecture)
- [ ] Add project structure (with notes on workspace migration)
- [ ] Document testing instructions
- [ ] Add key modules & symbols table
- [ ] Document anti-patterns and gotchas
- [ ] Add Australian healthcare context
- [ ] Document git conventions

### Phase 3: Wiki Structure
- [ ] Create wiki/ directory with git tracking
- [ ] Write getting-started guide
- [ ] Document architecture (TEA pattern, component architecture)
- [ ] Document domain models (Patient, Appointment, Consultation, etc.)
- [ ] Write API reference (repositories, services, errors)
- [ ] Document database schema
- [ ] Write testing guide
- [ ] Write deployment guide
- [ ] Add healthcare compliance section
- [ ] Create integration guides (Medicare, PBS, AIR stubs)
- [ ] Add troubleshooting guide

### Phase 4: Ratatui-Specific Docs
- [ ] Document TEA architecture pattern
- [ ] Document layout system (constraints, splitting)
- [ ] Document widgets used (Table, List, Form, etc.)
- [ ] Document event handling (keyboard, mouse)
- [ ] Document styling (colors, themes)
- [ ] Add performance optimization tips
- [ ] Add testing patterns for TUI components

### Phase 5: Healthcare-Specific Docs
- [ ] Document Medicare MBS integration (stub)
- [ ] Document PBS integration (stub)
- [ ] Document AIR integration (stub)
- [ ] Document FHIR readiness
- [ ] Document privacy & compliance (encryption, audit logging)
- [ ] Document My Health Record integration (future)

---

## References & Resources

### AGENTS.md
- https://agents.md/ — Official AGENTS.md specification
- https://www.builder.io/blog/agents-md — Builder.io guide
- https://github.com/agentsmd/agents.md — GitHub repository
- 60k+ examples: https://github.com/search?q=path%3AAGENTS.md

### Rust Project Templates
- https://github.com/OpenZeppelin/rust-project-template — Quality baseline
- https://tokio.rs — Tokio documentation (excellent README)
- https://serde.rs — Serde documentation (excellent README)

### Healthcare Standards
- https://developer.digitalhealth.gov.au/fhir/medicare-records/ — Australian Medicare Records FHIR IG
- https://www.pbs.gov.au/info/news/2024/12/new-pbs-schedule-data-api-and-api-csv-files — PBS API
- https://www.health.gov.au/topics/immunisation/ — AIR documentation
- https://docs.smarthealthit.org/ — SMART on FHIR

### Ratatui Documentation
- https://ratatui.rs/concepts/application-patterns/the-elm-architecture/ — TEA pattern
- https://ratatui.rs/concepts/application-patterns/component-architecture/ — Component pattern
- https://ratatui.rs/concepts/layout/ — Layout system
- https://ratatui.rs/concepts/rendering/under-the-hood/ — Rendering internals

### Healthcare EHR Documentation
- https://www.open-emr.org/wiki/ — OpenEMR wiki (excellent healthcare app docs)
- https://openmrs.atlassian.net/wiki/ — OpenMRS wiki
- https://fhir.epic.com/ — Epic FHIR documentation
- https://open.epic.com/DeveloperResources — Epic developer resources


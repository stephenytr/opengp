# OpenGP Documentation Rewrite: Quick Reference Card

**Print this or bookmark it** — quick lookup for all 5 documentation areas.

---

## 1️⃣ README.md Checklist

```markdown
# OpenGP
Brief description (1 line)

[![Build][build-badge]][build-url]
[![License][license-badge]][license-url]
[![MSRV][msrv-badge]][msrv-url]

[Quick Links](wiki) | [API Docs](docs.rs) | [Chat](discord)

## Overview
- What problem does it solve?
- Key components (3-5 bullets)

## Quick Start
```bash
cargo build --release
./target/release/opengp
```

## Features
- Feature 1 (with healthcare compliance highlight)
- Feature 2 (with TUI highlight)

## Documentation
- [Wiki](wiki/) — Developer guide
- [Contributing](CONTRIBUTING.md)
- [License](LICENSE)

## Status
- MSRV: Rust 1.70+
- Platforms: Linux, macOS, Windows
- Known limitations: Medicare/PBS/AIR stubs
```

**Time:** 1-2 hours | **Audience:** Everyone

---

## 2️⃣ AGENTS.md Template

```markdown
# AGENTS.md

## Project Overview
- **Purpose**: Australian GP management system
- **Language**: Rust
- **Framework**: Ratatui (TUI)
- **Database**: SQLite

## Setup Commands
- Build: `cargo build`
- Test: `cargo test`
- Run: `cargo run --release`
- Format: `cargo fmt`
- Lint: `cargo clippy`

## Code Style
- Naming: `snake_case` functions, `PascalCase` types
- Error handling: `thiserror` + `color-eyre`
- Architecture: Domain/Infrastructure/UI layers
- Testing: TDD (tests first)

## Project Structure
```
opengp/
├── src/domain/          # Business logic
├── src/infrastructure/  # Database, auth
├── src/ui/              # Ratatui TUI
├── crates/              # Workspace (new)
└── tests/               # Integration tests
```

## Key Modules
| Symbol | Type | Location | Role |
|--------|------|----------|------|
| Patient | Model | src/domain/patient/model.rs | Core entity |
| Appointment | Model | src/domain/appointment/model.rs | Scheduling |
| App | Struct | src/ui/app.rs | TUI app |

## Anti-Patterns (DO NOT)
- ❌ Commit `.env` files or API keys
- ❌ Use `rm` without explicit permission
- ❌ Revert git changes without permission
- ⚠️ Medicare/PBS/AIR are stubs (not implemented)

## Australian Healthcare Context
- Medicare MBS: Stub (needs implementation)
- PBS: Stub (needs implementation)
- AIR: Stub (needs implementation)
- Audit logging: Implemented (all access logged)

## Git Conventions
- Commit: `[module] Brief description`
- Branch: `feature/name` or `fix/name`
- PR: Link issue, describe changes, list tests
```

**Time:** 2-3 hours | **Audience:** AI agents + developers

---

## 3️⃣ Wiki Structure (10 Files)

```
wiki/
├── README.md                    # Index
├── 01-getting-started.md        # Install, first run
├── 02-architecture.md           # TEA pattern, layers
├── 03-domain-models.md          # Patient, Appointment, etc.
├── 04-api-reference.md          # Repositories, services
├── 05-database-schema.md        # Tables, migrations
├── 06-testing-guide.md          # Unit, integration tests
├── 07-deployment.md             # Build, config, monitoring
├── 08-healthcare-compliance.md  # Audit, privacy, encryption
├── 09-integrations/
│   ├── medicare-mbs.md          # MBS items, claiming
│   ├── pbs.md                   # PBS schedule, authority
│   └── air.md                   # Immunisation, reporting
└── 10-troubleshooting.md        # Errors, debugging
```

**Time:** 4-6 hours | **Audience:** Developers

---

## 4️⃣ Australian Healthcare Standards

### FHIR (Fast Healthcare Interoperability Resources)
```markdown
## FHIR Resources
- ExplanationOfBenefit: MBS/PBS claims
- Patient: Demographics
- Encounter: Clinical visit
- Medication: Prescription
- Immunization: Vaccination
```

### Medicare MBS Integration
```markdown
## MBS Item Numbers
| Item | Description | Rebate |
|------|-------------|--------|
| 23 | GP consultation | $38.75 |
| 36 | Extended consultation | $58.15 |

## Claiming Workflow
1. Record consultation (SOAP notes)
2. Select MBS item
3. Generate claim
4. Submit to Medicare
5. Track rebate
```

### PBS Integration
```markdown
## PBS Schedule API
- Endpoint: PBS Schedule Data API
- Update: Monthly (1st of month)
- Format: JSON, CSV, XML
- Auth: API key required

## Prescription Authority
- When: For restricted medicines
- How: Online via PBS website
- Time: Usually immediate
```

### AIR Integration
```markdown
## Mandatory Reporting (from 24 Oct 2025)
- COVID-19 vaccines: Mandatory
- Seasonal flu: Mandatory (from 1 Mar)
- NIP vaccines: Mandatory (from 1 Jul)

## Recording Workflow
1. Select vaccine from AIR schedule
2. Record administration date
3. Record provider details
4. Submit to AIR
5. Receive confirmation
```

**Time:** 3-4 hours | **Audience:** Healthcare developers

---

## 5️⃣ Ratatui TUI Documentation

### Architecture: The Elm Architecture (TEA)
```
Model (State)
    ↓
Update (Event → Model)
    ↓
View (Model → UI)
    ↓
Display
```

### Key Concepts
```markdown
## Layout System
- Length(n): Fixed size
- Percentage(n): % of space
- Ratio(num, denom): Ratio
- Min(n), Max(n): Bounds

## Widgets
- Paragraph: Text
- Table: Tabular data
- List: Scrollable list
- Block: Container
- Gauge: Progress bar

## Event Handling
- Key(KeyEvent): Keyboard
- Mouse(MouseEvent): Mouse
- Resize(w, h): Terminal resize
- Paste(String): Clipboard

## Styling
- Color: 16 colors + RGB
- Modifier: Bold, italic, underline
- Style: Combination of above
```

### Testing Pattern
```rust
#[test]
fn test_patient_selection() {
    let mut app = App::new();
    app.patients = vec![p1, p2, p3];
    
    app.update(Event::Key(KeyCode::Down));
    assert_eq!(app.selected, 1);
}
```

**Time:** 2-3 hours | **Audience:** TUI developers

---

## Implementation Timeline

```
Week 1:
  Mon-Tue: Phase 1 (README.md)
  Wed-Thu: Phase 2 (AGENTS.md)
  Fri: Review + adjust

Week 2:
  Mon-Wed: Phase 3 (Wiki structure)
  Thu-Fri: Phase 4 (Ratatui docs)

Week 3:
  Mon-Wed: Phase 5 (Healthcare docs)
  Thu-Fri: Review + polish
```

**Total:** 12-18 hours

---

## Critical Success Factors

✅ **DO:**
- Keep README concise (< 500 lines)
- Make AGENTS.md specific (exact commands)
- Link to external standards (don't copy)
- Document anti-patterns explicitly
- Be honest about stubs

❌ **DON'T:**
- Duplicate README in AGENTS.md
- Hide limitations or stubs
- Make wiki too long
- Forget healthcare compliance
- Ignore workspace migration status

---

## Tools & Support

### AGENTS.md
- Spec: https://agents.md/
- Examples: 60k+ on GitHub
- Tools: Claude, Cursor, Copilot, Codex, Jules, Aider, Zed, VS Code, Windsurf, Devin

### Ratatui
- Docs: https://ratatui.rs/
- TEA Pattern: https://ratatui.rs/concepts/application-patterns/the-elm-architecture/
- Layout: https://ratatui.rs/concepts/layout/

### Healthcare Standards
- Medicare FHIR: https://developer.digitalhealth.gov.au/fhir/medicare-records/
- PBS API: https://www.pbs.gov.au/info/news/2024/12/new-pbs-schedule-data-api-and-api-csv-files
- AIR: https://www.health.gov.au/topics/immunisation/
- SMART on FHIR: https://docs.smarthealthit.org/

### Examples
- Tokio README: https://github.com/tokio-rs/tokio/blob/master/README.md
- Serde README: https://github.com/serde-rs/serde/blob/master/README.md
- OpenEMR Wiki: https://www.open-emr.org/wiki/

---

## Questions?

1. **Quick overview?** → Read DOCUMENTATION_SUMMARY.md
2. **Detailed reference?** → Read DOCUMENTATION_RESEARCH.md
3. **Specific section?** → Use this quick reference card
4. **Implementation help?** → Ask Claude/Cursor with AGENTS.md context

---

**Last Updated:** March 11, 2026  
**Status:** Ready for Implementation

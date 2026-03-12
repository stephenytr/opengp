# OpenGP Documentation Rewrite: Executive Summary

**Date:** March 11, 2026  
**Status:** Research Complete  
**Full Report:** `docs/DOCUMENTATION_RESEARCH.md`

---

## Key Findings

### 1. README.md Best Practices ✅

**For Rust TUI Healthcare Apps, include:**

| Section | Purpose | Example |
|---------|---------|---------|
| **Header** | First impression | Project name + 1-line description |
| **Badges** | Status signals | Build, license, MSRV, version |
| **Quick Start** | Reduce friction | 3-5 lines to get running |
| **Features** | Differentiation | What makes it unique (compliance, healthcare) |
| **Links** | Navigation | Wiki, API docs, contributing, chat |
| **Status** | Transparency | Version, MSRV policy, known limitations |
| **License** | Legal clarity | MIT/Apache-2.0 + contribution terms |

**OpenGP-Specific Additions:**
- Healthcare compliance highlights (audit logging, encryption, privacy)
- Platform support (Linux, macOS, Windows)
- TUI-specific features (keyboard navigation, accessibility)
- Australian healthcare context (Medicare, PBS, AIR)

---

### 2. AGENTS.md: The AI Agent Playbook ✅

**What it is:** README for AI coding agents (Claude, Cursor, Copilot, etc.)

**Why it matters:** 
- 60k+ open-source projects use it (2026)
- Tells AI agents how to work in YOUR codebase
- Prevents repeated corrections across sessions
- Vendor-neutral (works with all major AI tools)

**Essential Sections for OpenGP:**

```
## AGENTS.md Structure
├── Project Overview (purpose, language, framework)
├── Setup Commands (build, test, run)
├── Code Style (naming, error handling, architecture)
├── Project Structure (with workspace migration notes)
├── Testing Instructions (unit, integration, coverage)
├── Key Modules & Symbols (table of important types)
├── Anti-Patterns (what NOT to do)
├── Australian Healthcare Context (Medicare, PBS, AIR)
└── Git Conventions (commit format, branch naming)
```

**Critical for OpenGP:**
- Document workspace migration status (src/ vs crates/)
- List all anti-patterns (no `.env` commits, no `rm` without permission)
- Explain healthcare compliance requirements
- Specify TDD approach and test coverage expectations

---

### 3. Wiki Structure: Developer Integration Guide ✅

**Standard Sections for Healthcare Apps:**

```
wiki/
├── 01-getting-started.md          # Installation, first run
├── 02-architecture.md              # TEA pattern, layers, data flow
├── 03-domain-models.md             # Patient, Appointment, Consultation, etc.
├── 04-api-reference.md             # Repositories, services, errors
├── 05-database-schema.md           # Tables, relationships, migrations
├── 06-testing-guide.md             # Unit, integration, patterns
├── 07-deployment.md                # Build, config, monitoring
├── 08-healthcare-compliance.md     # Audit, privacy, encryption
├── 09-integrations/
│   ├── medicare-mbs.md             # MBS item numbers, claiming
│   ├── pbs.md                      # PBS schedule, authorities
│   └── air.md                      # Immunisation, mandatory reporting
├── 10-troubleshooting.md           # Common errors, debugging
└── README.md                       # Index
```

**Healthcare-Specific Content:**
- Medicare MBS integration (item numbers, rebate calculations)
- PBS integration (schedule API, prescription authority)
- AIR integration (vaccination recording, mandatory reporting from Oct 2025)
- Privacy & compliance (Australian Privacy Principles, My Health Record)
- Clinical workflows (SOAP notes, referrals, pathology)

---

### 4. Australian Healthcare Standards ✅

**Key Standards for OpenGP Documentation:**

| Standard | Purpose | OpenGP Status |
|----------|---------|---------------|
| **FHIR** | Health data exchange | Ready (not yet implemented) |
| **Medicare MBS** | GP billing | Stub (needs documentation) |
| **PBS** | Prescription subsidy | Stub (needs documentation) |
| **AIR** | Immunisation register | Stub (needs documentation) |
| **My Health Record** | Patient data sharing | Future (needs planning) |

**Documentation Patterns:**
- FHIR resources (ExplanationOfBenefit, Patient, Encounter, Medication)
- Medicare claiming workflow (record → select item → claim → track)
- PBS authority requirements (when needed, how to request)
- AIR mandatory reporting (COVID-19, flu, NIP vaccines from Oct 2025)

---

### 5. Ratatui TUI Documentation ✅

**Architecture Patterns:**

**Option A: The Elm Architecture (TEA)** ← Recommended for OpenGP
```
Model (State) → Update (Event) → View (Render) → Display
```

**Option B: Component Architecture** (for larger apps)
```
Component Trait
├── init() — Setup
├── handle_events() — Input
├── render() — Draw
└── update() — State
```

**Documentation Sections:**
- Layout system (constraints, splitting screen)
- Widgets (Table, List, Form, Block, Gauge)
- Event handling (keyboard, mouse, resize)
- Styling (colors, themes, accessibility)
- Performance (rendering optimization)
- Testing (unit tests, integration tests)

---

## Implementation Roadmap

### Phase 1: README.md (1-2 hours)
- [ ] Add badges (build, license, MSRV)
- [ ] Write 1-line description + key features
- [ ] Add quick start (installation + first run)
- [ ] Link to wiki, API docs, contributing
- [ ] Add healthcare compliance highlights

### Phase 2: AGENTS.md (2-3 hours)
- [ ] Create AGENTS.md at project root
- [ ] Document all setup commands
- [ ] List code style conventions
- [ ] Add project structure diagram
- [ ] Document testing instructions
- [ ] Create key modules table
- [ ] List anti-patterns
- [ ] Add healthcare context

### Phase 3: Wiki Structure (4-6 hours)
- [ ] Create wiki/ directory
- [ ] Write getting-started guide
- [ ] Document architecture (TEA pattern)
- [ ] Document domain models
- [ ] Write API reference
- [ ] Document database schema
- [ ] Write testing guide
- [ ] Write deployment guide
- [ ] Add healthcare compliance section
- [ ] Create integration guides (stubs)

### Phase 4: Ratatui Docs (2-3 hours)
- [ ] Document TEA pattern
- [ ] Document layout system
- [ ] Document widgets used
- [ ] Document event handling
- [ ] Add testing patterns

### Phase 5: Healthcare Docs (3-4 hours)
- [ ] Medicare MBS integration guide
- [ ] PBS integration guide
- [ ] AIR integration guide
- [ ] Privacy & compliance guide
- [ ] Clinical workflows guide

**Total Estimated Time:** 12-18 hours

---

## Critical Insights

### ✅ What Works Well

1. **AGENTS.md is now standard** — 60k+ projects use it, all major AI tools support it
2. **Rust projects have proven README patterns** — Tokio, Serde set the standard
3. **Healthcare apps need compliance documentation** — Privacy, audit, standards
4. **Ratatui has clear architecture patterns** — TEA is well-documented
5. **Australian healthcare standards are well-defined** — FHIR, Medicare, PBS, AIR

### ⚠️ Gotchas to Avoid

1. **Don't duplicate README in AGENTS.md** — They serve different audiences
2. **Don't hide anti-patterns** — Be explicit about what NOT to do
3. **Don't forget workspace migration status** — Document src/ vs crates/ clearly
4. **Don't skip healthcare compliance** — It's not optional for Australian GP software
5. **Don't make wiki too long** — Link to external standards instead of copying

### 🎯 OpenGP-Specific Recommendations

1. **Emphasize TDD approach** — Tests first, implementation second
2. **Document the dual structure** — src/ has old code, crates/ has new implementation
3. **Be honest about stubs** — Medicare, PBS, AIR are not yet implemented
4. **Highlight audit logging** — It's a key compliance feature
5. **Note encryption status** — AES-GCM exists but is unused (document why)

---

## Tools & Resources

### AGENTS.md
- **Official Spec:** https://agents.md/
- **Examples:** 60k+ on GitHub (search: `path:AGENTS.md`)
- **Supported Tools:** Claude, Cursor, Copilot, Codex, Jules, Aider, Zed, VS Code, Windsurf, Devin

### Rust Project Templates
- **OpenZeppelin Template:** https://github.com/OpenZeppelin/rust-project-template
- **Tokio README:** https://github.com/tokio-rs/tokio/blob/master/README.md
- **Serde README:** https://github.com/serde-rs/serde/blob/master/README.md

### Healthcare Standards
- **Australian Medicare FHIR IG:** https://developer.digitalhealth.gov.au/fhir/medicare-records/
- **PBS API:** https://www.pbs.gov.au/info/news/2024/12/new-pbs-schedule-data-api-and-api-csv-files
- **AIR Documentation:** https://www.health.gov.au/topics/immunisation/
- **SMART on FHIR:** https://docs.smarthealthit.org/

### Ratatui Documentation
- **TEA Pattern:** https://ratatui.rs/concepts/application-patterns/the-elm-architecture/
- **Component Architecture:** https://ratatui.rs/concepts/application-patterns/component-architecture/
- **Layout System:** https://ratatui.rs/concepts/layout/

### Healthcare EHR Examples
- **OpenEMR Wiki:** https://www.open-emr.org/wiki/ (excellent healthcare app docs)
- **OpenMRS Wiki:** https://openmrs.atlassian.net/wiki/
- **Epic Developer Resources:** https://open.epic.com/DeveloperResources

---

## Next Steps

1. **Review this summary** with your team
2. **Read full report:** `docs/DOCUMENTATION_RESEARCH.md`
3. **Start with Phase 1** (README.md) — highest impact, lowest effort
4. **Create AGENTS.md** (Phase 2) — enables AI agents to help
5. **Build wiki structure** (Phase 3) — comprehensive developer guide
6. **Add Ratatui docs** (Phase 4) — TUI-specific patterns
7. **Document healthcare** (Phase 5) — compliance and integrations

---

## Questions?

Refer to the full research document: `docs/DOCUMENTATION_RESEARCH.md`

Key sections:
- Section 1: README.md structure (with examples)
- Section 2: AGENTS.md best practices (with templates)
- Section 3: Wiki structure (with healthcare sections)
- Section 4: Australian healthcare standards (with patterns)
- Section 5: Ratatui documentation (with code examples)


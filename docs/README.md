# OpenGP Documentation Research & Planning

This directory contains research and planning documents for the OpenGP documentation rewrite.

## Documents

### 1. **DOCUMENTATION_SUMMARY.md** (START HERE)
Executive summary of all findings with:
- Key findings for each of the 5 areas
- Implementation roadmap (12-18 hours total)
- Critical insights and gotchas
- Tools & resources
- Next steps

**Read this first** — it's a 5-minute overview of everything.

### 2. **DOCUMENTATION_RESEARCH.md** (DETAILED REFERENCE)
Comprehensive research report with:
- **Section 1:** README.md structure for Rust projects (with examples)
- **Section 2:** AGENTS.md best practices (with templates)
- **Section 3:** Wiki structure for healthcare apps (with sections)
- **Section 4:** Australian healthcare standards (with patterns)
- **Section 5:** Ratatui TUI documentation (with code examples)

**Use this** when implementing each phase — it has detailed templates and examples.

## Quick Reference

### What Each Document Covers

| Document | Purpose | Length | Audience |
|----------|---------|--------|----------|
| DOCUMENTATION_SUMMARY.md | Overview + roadmap | 5 min read | Everyone |
| DOCUMENTATION_RESEARCH.md | Detailed reference | 30 min read | Implementers |

### Key Findings at a Glance

1. **README.md** — Follow Tokio/Serde patterns (badges, quick start, links)
2. **AGENTS.md** — Create vendor-neutral AI agent playbook (60k+ projects use it)
3. **Wiki** — Structure with 10 sections (getting-started through troubleshooting)
4. **Healthcare Standards** — Document FHIR, Medicare, PBS, AIR (with stubs)
5. **Ratatui** — Use Elm Architecture (TEA) pattern for TUI

### Implementation Phases

```
Phase 1: README.md (1-2 hours)
Phase 2: AGENTS.md (2-3 hours)
Phase 3: Wiki Structure (4-6 hours)
Phase 4: Ratatui Docs (2-3 hours)
Phase 5: Healthcare Docs (3-4 hours)
─────────────────────────────────
Total: 12-18 hours
```

## How to Use These Documents

### For Project Leads
1. Read DOCUMENTATION_SUMMARY.md
2. Review the implementation roadmap
3. Assign phases to team members

### For Documentation Writers
1. Read DOCUMENTATION_SUMMARY.md for overview
2. Read relevant section in DOCUMENTATION_RESEARCH.md
3. Use templates and examples provided
4. Follow the checklist in DOCUMENTATION_SUMMARY.md

### For AI Agents (Claude, Cursor, etc.)
1. These documents are in your context
2. Use DOCUMENTATION_RESEARCH.md as reference
3. Follow patterns and templates exactly
4. Ask for clarification if needed

## Key Resources

### Official Standards
- **AGENTS.md Spec:** https://agents.md/
- **Ratatui Docs:** https://ratatui.rs/
- **Australian Healthcare:** https://developer.digitalhealth.gov.au/

### Example Projects
- **Tokio README:** https://github.com/tokio-rs/tokio/blob/master/README.md
- **Serde README:** https://github.com/serde-rs/serde/blob/master/README.md
- **OpenEMR Wiki:** https://www.open-emr.org/wiki/

## Next Steps

1. ✅ **Research Complete** (this directory)
2. ⏭️ **Phase 1: README.md** — Start here
3. ⏭️ **Phase 2: AGENTS.md** — Create AI agent playbook
4. ⏭️ **Phase 3: Wiki** — Build comprehensive guide
5. ⏭️ **Phase 4: Ratatui** — Document TUI patterns
6. ⏭️ **Phase 5: Healthcare** — Document compliance

---

**Last Updated:** March 11, 2026  
**Status:** Research Complete, Ready for Implementation

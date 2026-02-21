# Plan: Update AGENTS.md Hierarchical Knowledge Base

## TL;DR

> **Quick Summary**: Update 4 existing AGENTS.md files with fresh discoveries from codebase analysis
> 
> **Deliverables**: 
> - Updated ./AGENTS.md (root)
> - Updated ./src/ui/AGENTS.md
> - Updated ./src/infrastructure/AGENTS.md  
> - Updated ./src/domain/AGENTS.md
> 
> **Estimated Effort**: Short
> **Parallel Execution**: NO - sequential file updates
> **Critical Path**: Root → Domain → Infrastructure → UI

---

## Context

### Analysis Completed
- 132 Rust files, ~24k lines, max depth 4
- 4 existing AGENTS.md files found and analyzed
- 6 explore agents ran in parallel, covering: structure, entry points, conventions, anti-patterns, build/CI, test patterns

### Existing Files
| File | Lines | Status |
|------|-------|--------|
| `./AGENTS.md` | 96 | Good base, needs git info + enhanced anti-patterns |
| `./src/ui/AGENTS.md` | 52 | Good, needs test patterns |
| `./src/infrastructure/AGENTS.md` | 53 | Good, needs mock patterns |
| `./src/domain/AGENTS.md` | 58 | Good, needs module status |

---

## Work Objectives

### Core Objective
Enhance existing AGENTS.md files with newly discovered patterns without redundancy.

### Concrete Deliverables
1. Root AGENTS.md: Add git commit/branch info, expand anti-patterns from ISSUE_REPORT.md
2. Domain AGENTS.md: Add module status (active/stub), domain-level anti-patterns
3. Infrastructure AGENTS.md: Add test utilities, mock patterns, fixture generators
4. UI AGENTS.md: Add test patterns, view model conventions

### Definition of Done
- [x] All 4 files updated
- [x] No parent content repeated in children
- [x] Each file 50-150 lines
- [x] Telegraphic style maintained

### Must Have
- Accurate file paths verified
- Project-specific discoveries (not generic)
- Cross-references between files

### Must NOT Have
- Generic Rust advice
- Duplicate parent content
- Verbose explanations

---

## Verification Strategy

**No testing needed** - This is documentation update only.

---

## Execution Strategy

### Sequential Updates

```
1. Root AGENTS.md
   - Add: Git info (commit, branch)
   - Add: More anti-patterns from ISSUE_REPORT.md
   - Enhance: CODE MAP with more symbols

2. Domain AGENTS.md  
   - Add: Module status table (active vs stub)
   - Add: Domain anti-patterns (no sqlx in domain)
   - Enhance: Module pattern section

3. Infrastructure AGENTS.md
   - Add: Test patterns section
   - Add: Mock repository patterns
   - Add: Fixture generators (fake crate)

4. UI AGENTS.md
   - Add: Test conventions
   - Add: View model patterns
   - Enhance: Component structure
```

---

## TODOs

### Root AGENTS.md Updates
- [x] 1. Update timestamp to 2026-02-21
- [x] 2. Add git commit/branch info (use git rev-parse)
- [x] 3. Expand ANTI-PATTERNS section with:
  - Password hashing missing
  - Encryption unused  
  - Empty integrations (Medicare/PBS/AIR)
  - Audit log tamperable
- [x] 4. Enhance CODE MAP with more symbols

### Domain AGENTS.md Updates
- [x] 5. Add module status table (billing/pathology/immunisation/referral = stub)
- [x] 6. Add domain-specific anti-patterns:
  - No infrastructure imports (sqlx, tokio)
  - No direct database access in services

### Infrastructure AGENTS.md Updates
- [x] 7. Add Test Patterns section:
  - In-memory SQLite for tests
  - Mock repositories in mocks.rs
  - Test utilities in test_utils.rs
- [x] 8. Add Fixture Generators (fake crate)

### UI AGENTS.md Updates
- [x] 9. Add Test Conventions (integration tests, embedded tests)
- [x] 10. Add View Model patterns

---

## Commit Strategy

| After Task | Message | Files |
|------------|---------|-------|
| All | docs: Update AGENTS.md knowledge base | AGENTS.md src/*/AGENTS.md |

---

## Success Criteria

- [x] All 4 AGENTS.md files updated
- [x] Each file maintains 50-150 line limit
- [x] No redundancy between parent/child files
- [x] Cross-references preserved
- [x] Telegraphic style consistent

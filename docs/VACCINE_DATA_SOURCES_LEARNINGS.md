# Vaccine Data Sources (Australia) — Learnings

_Updated: 2026-04-21_

## Decision Summary

After reviewing ARTG, AIR, AMT/NCTS, and PBS options, the best approach for OpenGP is:

1. **Primary source for vaccine lists in product workflows:**
   - **Services Australia AIR vaccine codes**
2. **Regulatory verification source:**
   - **TGA ARTG export/search**
3. **Structured terminology/interoperability source (when needed):**
   - **AMT (via NCTS monthly release)**
4. **Not suitable as primary vaccine list source:**
   - **PBS API** (subsidised products only)

## Why ARTG Is Not the Primary Source

- ARTG is the authoritative regulatory register, but is not designed as a clean software-first vaccine catalogue feed.
- Practical access is web/search/export oriented (CSV/Excel via tool workflows), not a straightforward public API-first pipeline.
- It is best used to **verify registration/status** and enrich records, not as the core list presented to clinical workflows.

## Why AIR Vaccine Codes Should Be Primary

- AIR codes are the reporting-aligned identifiers used in Australian immunisation workflows.
- The AIR list includes key practical fields (code, brand/name, dose/equivalence context).
- This is the safest source to drive selection lists and downstream AIR-related data handling.

## Where AMT Fits

- AMT is valuable when we need standardised clinical terminology and structured coding alignment.
- It supports stronger interoperability (SNOMED/Australian terminology context).
- AMT releases are periodic (monthly model), so it complements real-world product validation from ARTG and reporting alignment from AIR.

## Why PBS API Is Not Primary

- PBS is about subsidised products/schedule data, not the complete vaccine universe.
- Using PBS as the primary vaccine catalogue would miss non-PBS-but-relevant vaccines.

## Recommended Source Hierarchy for OpenGP

### Tier 1 (Core)
- **AIR vaccine codes** for application vaccine catalogue and reporting-safe identifiers.

### Tier 2 (Validation/Enrichment)
- **ARTG** for product registration checks, sponsor/status confirmation, and metadata reconciliation.

### Tier 3 (Interoperability)
- **AMT/NCTS** for terminology mapping and structured coding requirements.

### Excluded as Primary
- **PBS API** (useful for billing/subsidy context, not full vaccine catalogue authority).

## Practical Operating Model

1. Build and maintain local vaccine catalogue from AIR codes.
2. Run periodic ARTG reconciliation to validate active registration status/details.
3. Add AMT mappings where terminology-grade interoperability is needed.
4. Keep PBS integration scoped to subsidy/billing use cases only.

## Notes for This Repository

- Current codebase has no implemented ARTG feed integration.
- Existing external import pattern (MBS importer in infrastructure) is a reusable template if/when automated vaccine ingestion is added.
- Immunisation module should treat AIR-aligned vaccine codes as first-class identifiers in user workflows.

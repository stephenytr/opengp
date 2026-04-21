# Medicare + AIR Test Environments — Investigation Learnings

_Updated: 2026-04-21_

## Scope

This document summarizes practical learnings for engineering planning around **Medicare** and **Australian Immunisation Register (AIR)** testing in Australia.

It is a consolidated reference for OpenGP, based on current project docs and external investigation.

## Executive Summary

1. There is no **publicly available sandbox API** for Medicare/AIR in the style of typical open developer platforms.
2. Real production-intent integration testing follows the **authoritative Services Australia vendor pathway** (PRODA + Health Systems Developer Portal + test certificates + test data + formal testing steps).
3. **AIR testing is generally not standalone**; it is tied to broader Medicare Online/Services Australia integration channels.
4. Third-party wrappers can provide easier sandboxes, but they are not the authoritative government integration path.

## Core Environment Model

### 1) Services Australia Vendor Test Path (Authoritative)

Primary path for production-intent integrations.

Typical components:
- PRODA identity and organisation setup
- Health Systems Developer Portal onboarding
- Interface/legal agreement acceptance
- Test certificate setup (for secure channel/auth)
- Vendor test data usage
- Notice of Connection / conformance-style testing before production access

### 2) AIR in Practice

- AIR workflows are integrated with Services Australia channels used by Medicare-related integrations.
- Plan AIR testing as part of the same onboarding and test cycle, not as a separate "independent public AIR sandbox" project.

### 3) Third-Party Sandboxes (Optional)

- Can accelerate dev/test cycles and improve developer ergonomics.
- Useful for early product iteration and API prototyping.
- Must not replace government-path validation for compliance-grade go-live.

## What We Should Assume for Planning

### Access & Onboarding
- Access is controlled and takes lead time.
- Expect account/org verification and environment provisioning dependencies.
- Build timeline should include waiting periods for approvals and credentials.
- Onboarding requirements and timelines can change; confirm current requirements directly with Services Australia during planning.

### Test Data Reality
- Test data is constrained and synthetic.
- Edge-case coverage may need custom stubbing/mocking on our side.
- Third-party sandboxes can accelerate development but may not match production behavior for all edge cases.

### Technical Integration Reality
- Secure authentication/certificate handling is part of the engineering scope.
- Environment and endpoint differences between test and production should be expected.
- Standalone AIR testing without Medicare-channel participation is typically limited or unavailable.

### Certification/Readiness Reality
- Treat formal test milestones as delivery gates, not optional tasks.
- Keep evidence (test logs, message samples, issue history) as part of release artifacts.

## OpenGP-Specific Implications

1. Keep Medicare/AIR adapters behind domain traits (existing architecture pattern).
2. Add environment-aware config for test vs production endpoints/credentials.
3. Build a local simulator layer for deterministic CI tests (do not rely on external test environment uptime).
4. Track conformance tasks in roadmap as first-class milestones.
5. Keep operational runbooks for certificate renewal, credential rotation, and incident response.

## Recommended Delivery Strategy

### Phase 1 — Internal Development
- Implement client abstractions and contract tests with local mocks.
- Add strict request/response validation and audit logging.

### Phase 2 — Vendor Test Environment
- Connect to Services Australia vendor test environment.
- Execute scripted test matrix for Medicare + AIR scenarios.

### Phase 3 — Formal Testing Gates
- Complete required formal testing steps.
- Resolve defects and re-run targeted suites.

### Phase 4 — Production Readiness
- Finalize operational controls (monitoring, retries, failure handling, cert lifecycle).
- Promote with rollback plan and post-go-live verification checks.

## Risks to Watch

- Credential/certificate delays blocking sprint goals
- Environment drift between test and production
- Limited external test data reducing confidence in edge cases
- Hidden coupling between Medicare and AIR flows discovered late

## Practical Next Steps

1. Treat this as the planning baseline for Medicare/AIR integration work.
2. Convert onboarding/certification milestones into explicit Jira/GitHub tasks.
3. Define a minimum conformance evidence checklist before any production target date.
4. Keep this document updated as official guidance or portal workflows change.

## Related Project Docs

- `docs/Medicare-Test-Environments-2026.md`
- `docs/AIR_TEST_ENVIRONMENT_INVESTIGATION.md`
- `docs/AIR_QUICK_REFERENCE.md`
- `REQUIREMENTS.md` (Medicare/AIR/PRODA/HI roadmap and compliance milestones)

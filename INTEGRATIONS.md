# OpenGP Integrations Guide

> **Scope**: This document defines the **intended** architecture and implementation patterns for OpenGP external API integrations.
> The integration modules under `src/integrations/` are currently stubs; use this guide as the implementation blueprint.

OpenGP targets Australian healthcare providers, which means integrations are not “nice to have”.
They drive core workflows (Medicare claiming, PBS prescribing, AIR reporting, Healthcare Identifiers) and come with
strict legal, security, and conformance requirements.

This guide focuses on:

* Client usage patterns (how we structure integration clients)
* Error handling (typed, actionable, and safe)
* Retry logic (resilient but compliant)
* Rate limiting (vendor limits + protecting ourselves)
* Australian healthcare API specifics (auth, transport, formats)
* Compliance requirements (audit logging, encryption, PII handling)
* Testing strategy (mock HTTP, deterministic retries)

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Client Patterns](#client-patterns)
4. [Error Handling](#error-handling)
5. [Retry Logic](#retry-logic)
6. [Rate Limiting](#rate-limiting)
7. [Medicare Online](#medicare-online)
8. [PBS API](#pbs-api)
9. [AIR (Australian Immunisation Register)](#air-australian-immunisation-register)
10. [HI Service (Healthcare Identifiers)](#hi-service-healthcare-identifiers)
11. [Compliance Requirements](#compliance-requirements)
12. [Testing Strategy](#testing-strategy)
13. [Operational Guidance](#operational-guidance)

---

## Overview

### Why integrations matter

OpenGP’s primary purpose is to support clinical workflows.
In Australian general practice, the “system boundary” frequently crosses into external systems:

* **Medicare Online** for real-time claiming, eligibility checks, and claim status.
* **PBS** for medication schedule lookups, pricing/status, and authority request workflows.
* **AIR** for mandatory vaccination notifications **within 24 hours** of administration.
* **HI Service** to validate and search healthcare identifiers (IHI, HPI-I, HPI-O).

Integration design must prioritize:

* **Safety**: no silent failures; clear operator feedback; avoid partial submissions.
* **Resilience**: retry transient outages without duplicating side effects.
* **Compliance**: audit trail, encryption requirements, and PII minimization.
* **Conformance**: government services require formal conformance testing.

### Non-negotiables (healthcare context)

From `REQUIREMENTS.md` and project conventions:

* Never log sensitive patient data (PII, clinical notes, identifiers) in plaintext.
* Encrypt sensitive clinical data **before** storage.
* Audit log all patient record access and all external calls that process patient data.
* Fail safe: prefer “deny / retry / queue” over “best-effort submit and hope”.
* Keep patient data in Australia (data sovereignty).

---

## Architecture

### Integration layer location

Integrations live under:

```text
src/integrations/
  mod.rs
  medicare/
    mod.rs
  pbs/
    mod.rs
  air/
    mod.rs
  hi_service/
    mod.rs
```

Planned additions from `REFACTORING_PLAN.md` Phase 5 (external integrations):

```text
src/integrations/
  client.rs              # Shared HTTP client wrapper (timeouts, headers, tracing)
  error.rs               # Shared IntegrationError + helpers
  rate_limit.rs          # Token bucket / throttling (implementation detail)
  retry.rs               # backoff-based retry helpers
  medicare/
    client.rs
    models.rs
    error.rs
  pbs/
    client.rs
    models.rs
    error.rs
  air/
    client.rs
    models.rs
    error.rs
  hi_service/
    client.rs
    models.rs
    error.rs
```

### Dependency direction

Integrations are an “outer” layer.

* Domain may **define traits** for external capabilities (e.g., `MedicareValidator`, `AirClient`),
  but must not depend on concrete integration implementations.
* Integration clients depend on:
  * configuration (base URLs, credential sources)
  * audit logger (for call-level audit events)
  * crypto (when payloads contain sensitive content needing encryption at rest)

### Common integration capabilities

From `ARCHITECTURE.md` integration principles:

1. **Resilience** (retry with exponential backoff)
2. **Circuit breaker** (avoid cascading failures)
3. **Timeouts** (never wait indefinitely)
4. **Fallback** (graceful degradation)
5. **Audit** (log all external calls)
6. **Health checks** (monitor availability)

The integration layer should provide shared primitives for the above.

### External service “health check” contract

`ARCHITECTURE.md` sketches a common trait:

```rust
#[async_trait]
pub trait ExternalService: Send + Sync {
    async fn health_check(&self) -> Result<HealthStatus, IntegrationError>;
    fn service_name(&self) -> &str;
    fn is_critical(&self) -> bool;
}

pub struct HealthStatus {
    pub is_healthy: bool,
    pub latency_ms: u64,
    pub last_check: DateTime<Utc>,
    pub error_message: Option<String>,
}
```

Implementation guidance:

* `health_check()` must not include PII.
* Prefer a vendor “ping” endpoint if available.
* If no vendor health endpoint exists, use a minimal, non-side-effecting request.

---

## Client Patterns

### Goals of the base client wrapper

Every integration should share a consistent, boring foundation:

* timeouts
* correlation IDs (request IDs)
* structured tracing (without sensitive payloads)
* retry hooks
* rate limiting hooks
* response classification (success / retryable / non-retryable)

### Shared request context

Every integration call should carry a **request context** created at the application boundary
(UI → app → domain → integrations):

* `correlation_id`: UUID generated once per user action or background job
* `actor_user_id`: UUID of the authenticated user (or a system user for background tasks)
* `purpose`: why we’re calling the service (claim, eligibility check, etc.)
* `patient_ref`: **optional**, and must be a safe reference (never raw Medicare/IHI)

Example context type (conceptual):

```rust
#[derive(Debug, Clone)]
pub struct IntegrationContext {
    pub correlation_id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub purpose: &'static str,
    pub patient_ref: Option<Uuid>,
}
```

Guidance:

* Put `correlation_id` into all logs and audit entries.
* Do not include clinical payload data in `IntegrationContext`.
* If you must refer to a patient, prefer the internal `Patient.id` (UUID).

### Base HTTP client (from REFACTORING_PLAN.md 5.1)

The refactoring plan proposes a shared `ApiClient` wrapper:

```rust
//! Base HTTP client for external API integrations

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct ApiClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        Self { client, base_url, api_key }
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
    ) -> Result<T, IntegrationError> {
        // Implementation with retry logic, logging, error handling
    }

    pub async fn post<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<R, IntegrationError> {
        // Implementation with retry logic, logging, error handling
    }
}
```

**Important note**: production code must not use `.unwrap()`.
When implementing, convert builder failures into `IntegrationError::ClientBuild` (or similar) using `?`.

### SOAP-specific client considerations

Many Australian healthcare government services are SOAP-based.

Requirements for SOAP clients:

* Set `Content-Type` appropriately (often `text/xml` or `application/soap+xml`)
* Set `SOAPAction` header where required
* Parse SOAP faults:
  * fault code
  * fault string/message
  * vendor detail blocks (often include structured reason codes)
* Redact *everything* that might include PII before logging

Minimal “send SOAP envelope” shape (conceptual):

```rust
pub async fn post_soap(
    &self,
    path: &str,
    soap_action: &str,
    envelope_xml: &str,
    ctx: &IntegrationContext,
) -> Result<String, IntegrationError> {
    // 1) rate limit
    // 2) add auth header
    // 3) audit start
    // 4) send request
    // 5) classify response / parse fault
    // 6) audit end
    // 7) return response XML
}
```

Never log `envelope_xml`.

### Recommended production signature

Prefer explicit dependency injection and safe construction:

```rust
pub struct ApiClient {
    client: reqwest::Client,
    base_url: url::Url,
    auth: AuthStrategy,
    retry: RetryPolicy,
    limiter: Option<RateLimiter>,
    audit: Arc<AuditLogger>,
}

impl ApiClient {
    pub fn new(config: ApiClientConfig, audit: Arc<AuditLogger>) -> Result<Self, IntegrationError> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(IntegrationError::http_client_build)?;

        Ok(Self {
            client,
            base_url: config.base_url,
            auth: config.auth,
            retry: config.retry,
            limiter: config.limiter,
            audit,
        })
    }
}
```

This document references `url::Url` and custom structs for clarity; actual implementation can start simpler.

### Client module structure

Each integration should use the same internal shape:

```text
src/integrations/{service}/
  client.rs   # public methods
  models.rs   # request/response DTOs, parsers
  error.rs    # service-specific errors
```

Pattern:

* `client.rs` exposes a single main client type.
* `models.rs` contains only transport-level types (SOAP envelopes, JSON responses), not domain entities.
* `error.rs` defines `FooError` which can wrap `IntegrationError`.

### Audit logging integration calls

All external calls that touch patient data should be audited.

* Audit event should include:
  * service name (e.g., `medicare-online`)
  * operation (e.g., `submit_claim`)
  * correlation ID
  * user ID (actor)
  * result (`success` / `failure`)
  * timing/latency
* Audit event must not include raw payloads.

Example (conceptual):

```rust
#[instrument(
    skip_all,
    fields(service = "medicare-online", op = "submit_claim", correlation_id = %correlation_id)
)]
pub async fn submit_claim(&self, claim: &MedicareClaim, user: &User) -> Result<ClaimResponse, MedicareError> {
    let started = Instant::now();

    let result = self.inner_submit_claim(claim).await;

    self.audit.log(AuditEvent {
        user_id: user.id,
        action: AuditAction::ExternalIntegrationCall,
        entity_type: Some("MedicareClaim".to_string()),
        entity_id: None,
        metadata: Some(json!({
            "service": "medicare-online",
            "operation": "submit_claim",
            "correlation_id": correlation_id,
            "latency_ms": started.elapsed().as_millis(),
            "result": if result.is_ok() { "SUCCESS" } else { "FAILURE" }
        })),
        timestamp: Utc::now(),
        ..Default::default()
    }).await?;

    result
}
```

The actual `AuditAction` enum may evolve; treat this as an illustration of the desired shape.

---

## Error Handling

### Goals

Integration errors must be:

* **typed** (callers can respond appropriately)
* **safe** (no PII leaks in error strings)
* **actionable** (operators can understand next steps)
* **traceable** (correlation IDs, status codes, vendor error codes)

### Error taxonomy

At a minimum:

* **Transport**: DNS, TCP, TLS, connection refused
* **Timeout**: request exceeded configured limit
* **Authentication**: token expired, invalid client credentials, certificate invalid
* **Authorization**: insufficient scope/role
* **Client error** (4xx): invalid request, validation failed
* **Rate limited**: vendor throttling
* **Server error** (5xx): vendor outage
* **Serialization**: JSON/XML parse errors
* **Protocol**: SOAP fault parsing, schema mismatch
* **Conformance**: vendor-specific “business rule” errors

### Shared IntegrationError

Recommended shared error type (conceptual):

```rust
#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("http client build failed")]
    HttpClientBuild,

    #[error("transport error")]
    Transport,

    #[error("request timed out")]
    Timeout,

    #[error("authentication failed")]
    Auth,

    #[error("remote rate limit exceeded")]
    RateLimited,

    #[error("remote service error")]
    RemoteService,

    #[error("invalid response from remote service")]
    InvalidResponse,

    #[error("serialization error")]
    Serialization,

    #[error("circuit breaker open")]
    CircuitOpen,
}
```

Implementation guidance:

* Keep `Display` messages generic.
* Preserve details (HTTP status code, vendor error code) in structured fields, not in stringified messages.
* Always attach a correlation ID in logs (not payloads).

### Mapping vendor errors

Australian healthcare integrations often return “business errors” that are not transport errors.
Examples:

* Duplicate AIR notification
* Medicare claim rejected (item invalid, eligibility mismatch)
* HI Service demographic mismatch

These should become typed errors with a safe summary:

```rust
#[derive(Debug, thiserror::Error)]
pub enum MedicareError {
    #[error("claim rejected")]
    ClaimRejected { reason_code: String },

    #[error("soap fault")]
    SoapFault { fault_code: String },

    #[error(transparent)]
    Integration(#[from] IntegrationError),
}
```

### Error classification helpers

Make retry decisions in one place.

Recommended pattern:

```rust
impl IntegrationError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            IntegrationError::Transport
                | IntegrationError::Timeout
                | IntegrationError::RateLimited
                | IntegrationError::RemoteService
        )
    }
}
```

For SOAP APIs, consider a separate classifier:

* some SOAP faults are “permanent” (schema/validation)
* some SOAP faults are “transient” (service unavailable)

Do not guess.
If vendor documentation distinguishes retryable fault codes, encode that mapping explicitly.

### Do not leak PII

Never include these in error strings or logs:

* Medicare numbers, IRNs
* IHI / HPI-I / HPI-O
* patient name/DOB
* claim details and clinical notes
* raw SOAP envelopes / JSON payloads

If an operator needs payload access for debugging, use a secure redacted trace mechanism,
stored encrypted and access controlled, with explicit opt-in.

---

## Retry Logic

### When to retry

Retry only when:

* the request is **idempotent** (GET, safe lookup)
* or the operation is made idempotent via:
  * an **idempotency key**,
  * a vendor-provided transaction/reference ID,
  * or a “check status first” pattern.

Retry candidates:

* network timeouts
* connection resets
* 502/503/504 server errors
* rate limiting responses (after server-provided delay)

Do not retry:

* validation errors (4xx)
* authentication failures (fix credentials)
* business rule rejections (claim rejected)

### Backoff strategy

`REFACTORING_PLAN.md` Phase 5 suggests adding:

```toml
reqwest = { version = "0.12", features = ["json"] }
backoff = "0.4"
```

Preferred pattern:

* exponential backoff with jitter
* max elapsed time cap
* max retries cap
* respect vendor `Retry-After` if present

Example (conceptual):

```rust
use backoff::ExponentialBackoff;

pub async fn retry_with_backoff<F, Fut, T>(mut op: F) -> Result<T, IntegrationError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, IntegrationError>>,
{
    let mut backoff = ExponentialBackoff {
        max_elapsed_time: Some(Duration::from_secs(60)),
        ..Default::default()
    };

    backoff::future::retry(backoff, || async {
        match op().await {
            Ok(v) => Ok(v),
            Err(e) if e.is_retryable() => Err(backoff::Error::transient(e)),
            Err(e) => Err(backoff::Error::permanent(e)),
        }
    }).await
}
```

### Handling “submitted but unknown outcome”

The most dangerous failure mode is:

* request sent
* connection drops before response
* caller doesn’t know whether the operation succeeded

For side-effecting operations (claim submission, AIR notification):

* generate a stable internal reference ID
* include it in the vendor request if possible
* persist a “pending submission” record with that reference
* if outcome unknown, **query status** before retrying submit

This avoids duplicate submissions.

### Outbox pattern for side effects

For legally important submissions (claims, AIR notifications), prefer an **outbox** record:

* Write a `pending_external_submission` row in the DB inside the same transaction that creates/updates domain state.
* A background worker sends submissions and marks them `sent` / `failed`.
* The UI shows pending items and allows operator review.

This reduces the chance that a UI crash or network error loses a required submission.

---

## Circuit Breaker (recommended)

`ARCHITECTURE.md` calls out circuit breakers to prevent cascading failures.

Concept:

* Track recent failures per service.
* If failure rate exceeds a threshold, “open” the breaker for a cool-down period.
* While open, reject calls quickly with `IntegrationError::CircuitOpen`.
* Half-open after cool-down: allow a small number of trial calls.

Implementation note:

* Keep the circuit breaker state in-memory.
* Reset after successful calls.
* Do not include PII in breaker metrics.

---

---

## Rate Limiting

### Why we rate limit

We rate limit for two reasons:

1. **Vendor compliance**: some APIs enforce strict request ceilings.
2. **Self-protection**: avoid stampedes (e.g., bulk patient import verifying IHIs).

### Where rate limiting lives

Rate limiting belongs in the shared integration client layer,
so every caller gets the same behavior.

Recommended shape:

* a global limiter per service
* optionally sub-limiters per operation

### Token bucket (no extra dependencies)

You can implement a token bucket with `tokio::sync::Semaphore` + periodic refill.

Conceptual example:

```rust
pub struct RateLimiter {
    tokens: Arc<Semaphore>,
    capacity: usize,
}

impl RateLimiter {
    pub async fn acquire(&self) {
        // Await a permit before sending
        let _permit = self.tokens.acquire().await;
        // Permit dropped at end of request
    }
}
```

In practice:

* refill tokens at a configured interval
* allow short bursts (capacity > steady rate)
* prefer per-service configuration

### Vendor-specific limits (what we know)

From `REQUIREMENTS.md`:

* **PBS public**: documented as **~1 request per 20 seconds** (public access).

For other services:

* Medicare Online / AIR / HI Service rate limits are typically part of vendor technical specs and may vary
  by environment, contract, and conformance profile.

**Implementation requirement**:

* rate limits must be configurable
* the limiter must enforce a conservative default
* adjust values during conformance testing

Suggested safe defaults (until confirmed):

* Medicare Online: low steady rate + small burst
* AIR: low steady rate, with batch support for historical uploads
* HI Service: low steady rate; prefer caching to reduce calls

Avoid hard-coding numbers unless sourced from the vendor documentation available to the project.

### Suggested configuration keys

Keep rate limits per service in config (names illustrative):

```toml
[integrations.pbs]
rate_limit_permits = 1
rate_limit_interval_ms = 20000

[integrations.medicare]
rate_limit_permits = 2
rate_limit_interval_ms = 1000

[integrations.air]
rate_limit_permits = 1
rate_limit_interval_ms = 1000

[integrations.hi_service]
rate_limit_permits = 1
rate_limit_interval_ms = 1000
```

Only the PBS values above are directly stated in `REQUIREMENTS.md`.
The others are **conservative defaults** and must be validated during integration testing/conformance.

---

## Medicare Online

### What it is

Medicare Online is the Services Australia integration surface for claiming and verification.

From `REQUIREMENTS.md`:

* SOAP-based web services
* PRODA OAuth 2.0 authentication
* Conformance testing required
* Production certification needed

### Typical operations

* Patient eligibility verification
* Claim submission (bulk billing, patient claims)
* Claim status retrieval
* Claim deletion (same-day, where permitted)
* DVA claiming (where applicable)

### Authentication

Medicare Online uses **PRODA** as the identity and access gateway.

Pattern (from `ARCHITECTURE.md`):

* `ProdaAuthService` fetches access tokens
* tokens are cached until expiry
* clients attach bearer token to requests

```rust
pub struct ProdaAuthService {
    client_id: String,
    client_secret: String,
    token_url: String,
    http_client: reqwest::Client,
    token_cache: Arc<RwLock<Option<CachedToken>>>,
}
```

### Transport and data format

* Transport: HTTPS
* Data format: SOAP (XML envelopes)
* Expect SOAP faults; parse them into typed errors.

### Endpoint handling

Do not hard-code endpoints in code.

* Test and production base URLs differ.
* Some operations may use different paths/ports.
* Keep all endpoints in configuration.

In documentation and examples, use placeholders:

* `MEDICARE_BASE_URL=https://example.invalid/medicare`

### Request identifiers

For all claim submissions:

* generate an internal claim UUID
* include that UUID in the request where possible (vendor reference / message ID)
* store a mapping from internal claim UUID → vendor claim ID (when returned)

### Claim lifecycle (high level)

Typical lifecycle:

1. Build claim from billing domain data
2. Validate locally (item codes, required fields)
3. Submit claim
4. Record vendor reference ID
5. Poll / query claim status until finalised
6. Update billing state and present outcome

Each step should be auditable.

### Client shape (from REFACTORING_PLAN.md 5.2)

The refactoring plan proposes:

```rust
pub struct MedicareClient {
    client: ApiClient,
}

impl MedicareClient {
    pub async fn verify_medicare_number(&self, number: &str, irn: u8) -> Result<bool>;
    pub async fn submit_claim(&self, claim: &MedicareClaim) -> Result<ClaimResponse>;
    pub async fn check_claim_status(&self, claim_id: &str) -> Result<ClaimStatus>;
}
```

### Operational notes

* Claim submission must be auditable:
  * who submitted
  * when
  * which local reference
  * outcome code
* Treat “no response” as unknown outcome; query status before resubmitting.
* Avoid logging request bodies (SOAP).

---

## PBS API

### What it is

The PBS API provides medication schedule data and related services.

From `REQUIREMENTS.md`:

* RESTful API
* JSON and XML support
* Monthly schedule updates
* Rate limits exist (public access noted as ~1 request per 20 seconds)

### Typical operations

* search medication by name/code
* retrieve PBS status (PBS/RPBS listing, restrictions)
* authority request workflows (where supported)
* pricing lookup

### Authentication

Authentication varies by PBS endpoint and access tier.
Design for multiple auth strategies:

* API key header
* OAuth2 (if required in higher tiers)

### Client shape (from REFACTORING_PLAN.md 5.3)

```rust
pub struct PBSClient {
    client: ApiClient,
}

impl PBSClient {
    pub async fn search_medication(&self, query: &str) -> Result<Vec<Medication>>;
    pub async fn get_pbs_status(&self, medication_code: &str) -> Result<PBSStatus>;
    pub async fn request_authority(&self, request: &AuthorityRequest) -> Result<AuthorityResponse>;
}
```

### Rate limiting and caching

Given the public limit in `REQUIREMENTS.md`, design PBS calls to:

* cache results aggressively (schedule data changes monthly)
* batch or pre-load common codes
* enforce a strict client-side limiter

### Data freshness

PBS schedule data changes monthly.

Design implications:

* cache by “schedule version” (month)
* support background refresh on schedule change
* keep a deterministic “effective date” for lookups

### Embargo APIs

`REQUIREMENTS.md` mentions a PBS Embargo API (advance schedule changes).
Treat embargo access as a separate capability with separate credentials and stricter logging controls.

---

## AIR (Australian Immunisation Register)

### What it is

AIR is a national register; reporting is mandatory.

From `REQUIREMENTS.md`:

* SOAP web services via Medicare Online
* PRODA authentication
* Real-time notifications within 24 hours
* Must handle duplicate notifications

### Typical operations

* submit immunisation notification (encounter)
* upload historical immunisation data (batch)
* retrieve immunisation history

### Client shape (from REFACTORING_PLAN.md 5.4)

```rust
pub struct AIRClient {
    client: ApiClient,
}

impl AIRClient {
    pub async fn submit_immunisation(&self, immunisation: &Immunisation) -> Result<String>;
    pub async fn get_immunisation_history(&self, patient_ihi: &str) -> Result<Vec<Immunisation>>;
}
```

### Duplicate handling

Duplicates are a known vendor scenario.

Guidance:

* Prefer idempotency: include a stable internal event ID if the API supports it.
* If vendor signals “duplicate”, treat as a successful no-op when safe.
* Audit duplicate detection as `SUCCESS` with a “duplicate accepted” flag.

### Compliance timing

AIR reporting must occur within 24 hours.

Implementation implication:

* if immediate submission fails, enqueue for retry (with operator visibility)
* persist submission attempts and outcomes
* alert if approaching SLA breach

### Data format

AIR is SOAP via Medicare Online.

Practical guidance:

* keep SOAP message construction in `models.rs`
* keep parsers for response/faults close to the DTOs
* ensure duplicate handling is explicit and covered by tests

---

## HI Service (Healthcare Identifiers)

### What it is

The HI Service verifies and searches healthcare identifiers.

From `REQUIREMENTS.md`:

* SOAP-based B2B gateway
* Certificate-based authentication
* Conformance testing required
* Covers:
  * IHI (patients)
  * HPI-I (practitioners)
  * HPI-O (organizations)

### Typical operations

* verify IHI
* search IHI by demographics
* verify HPI-I
* verify HPI-O (future)
* registration status checks

### Client shape (from REFACTORING_PLAN.md 5.5)

```rust
pub struct HIServiceClient {
    client: ApiClient,
}

impl HIServiceClient {
    pub async fn verify_ihi(&self, ihi: &str) -> Result<bool>;
    pub async fn search_ihi(&self, demographics: &Demographics) -> Result<Option<String>>;
    pub async fn verify_hpi_i(&self, hpi_i: &str) -> Result<bool>;
}
```

### PII handling

HI Service requests frequently require demographic identifiers.

Rules:

* never log demographics
* redact identifiers from errors
* audit only “operation performed” + result codes

### Caching strategy

HI validations may be called frequently.

Caching guidance:

* cache positive verification results for a short TTL
* cache negative results carefully (demographics can be corrected)
* never cache raw demographics in plaintext
* prefer caching by internal patient UUID + verified identifier status

---

## Appendix: Redaction checklist

Before adding any log line in integration code, verify it does not include:

* request body
* response body
* SOAP envelope
* patient identifiers (Medicare/IHI/HPI)
* names/DOB/address

Safe to log:

* correlation ID
* service name
* operation
* HTTP status code
* latency
* vendor error code (if it contains no PII)
* retry attempt count

---

## Compliance Requirements

### Audit logging

From `REQUIREMENTS.md`:

* all patient data access must be logged
* audit logs must be immutable (append-only)
* retention: 7 years minimum
* audit logs must support tamper detection

For integrations:

* audit every external call that processes patient data
* include correlation ID and operation name
* do not store raw payloads in the audit log

### Encryption

From `REQUIREMENTS.md` and `ARCHITECTURE.md`:

* AES-256-GCM application-level encryption
* encrypt clinical notes and other sensitive fields before persistence
* never store encryption keys in code or git

Integration implications:

* if an integration response contains sensitive clinical content that must be stored,
  encrypt before writing to DB
* do not write raw payloads to disk

### Data sovereignty

From `REQUIREMENTS.md`:

* primary requirement: patient data must be stored in Australia
* cloud deployments must use Australian regions
* avoid cross-border disclosure without consent

Integration implication:

* prefer vendors with Australian hosting for any third-party services
* review any “telemetry” or “support capture” features

### Conformance and certification

From `REQUIREMENTS.md`:

* Services Australia (Medicare Online, AIR) conformance testing is required
* HI Service conformance is required
* My Health Record conformance has additional security requirements

Engineering implications:

* build integrations with strict schema validation and deterministic error handling
* keep a “conformance mode” that logs additional safe diagnostics (no PII)
* maintain test scenario fixtures for vendor conformance suites

---

## Testing Strategy

### What to test

For each integration client:

* request building (SOAP envelope or JSON payload shape)
* response parsing
* error classification (retryable vs permanent)
* retry behavior (caps, jitter, Retry-After)
* rate limiting behavior
* audit event emission (no payload leakage)

### Mock HTTP responses

`REFACTORING_PLAN.md` suggests:

```toml
[dev-dependencies]
mockito = "1.5"
```

Use mock servers to:

* simulate 200 OK success
* simulate 4xx validation errors
* simulate 429 throttling with Retry-After
* simulate 5xx outages
* simulate malformed XML/JSON

### Deterministic retries

To keep tests stable:

* inject retry policy (disable in most unit tests)
* in retry tests, use a small max elapsed time and a fixed backoff schedule
* avoid real sleeps when possible (or keep them tiny)

### Contract tests

Where vendor schemas are fixed (FHIR, HL7, SOAP XSD):

* keep sample payloads under `tests/fixtures/`
* validate parsing against known examples
* ensure error extraction works for vendor-specific fault formats

---

## Operational Guidance

### Observability

Use structured logs (`tracing`) for integration events:

* service
* operation
* correlation_id
* status_code (if applicable)
* latency
* retry_attempt

Never include patient identifiers or raw payloads.

### Configuration

Integration configuration should be centralized and environment-driven:

* base URLs (test vs prod)
* auth credentials (client ID/secret, certificates)
* timeouts
* retry policy
* rate limits

Never commit credentials.

### Safe failure modes

When an integration is unavailable:

* the UI should show a clear “degraded” state
* the domain workflow should queue operations where legal and safe
* operators should be able to see pending submissions and errors

### Security reviews

Before production use:

* verify TLS configuration (TLS 1.2 minimum, TLS 1.3 preferred)
* perform dependency audit (`cargo-audit`) in CI
* ensure logs are scrubbed of PII
* confirm audit log immutability controls

---

## Appendix: Quick reference (Phase 9 / Task 9.3)

This document implements Phase 9 documentation deliverable “INTEGRATIONS.md”.
Key referenced sources:

* `REFACTORING_PLAN.md` Phase 5.1–5.5 for integration client skeletons
* `REQUIREMENTS.md` Integration Requirements + Security Requirements
* `ARCHITECTURE.md` Integration Architecture principles

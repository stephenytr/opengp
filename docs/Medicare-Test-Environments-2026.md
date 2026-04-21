# Medicare Claiming & Integration Test Environments for Australian Software Vendors (2026)

**Last Updated:** April 2026  
**Scope:** Official test/sandbox environments available to software vendors integrating with Australian Medicare, My Health Record, and related systems.

---

## EXECUTIVE SUMMARY

Australian software vendors have access to **three primary integration channels** with distinct test environments:

1. **Medicare Online (PRODA/Web Services)** — Real-time claim submission, patient verification, AIR, DVA
2. **My Health Record (ADHA)** — Clinical document exchange, healthcare identifiers
3. **Claiming.com.au API** — Third-party Medicare/DVA/ECLIPSE claiming wrapper (optional alternative)

All require **PRODA registration** and **NASH PKI test certificates**. Testing is **mandatory** before production access.

---

## 1. MEDICARE ONLINE (PRODA/WEB SERVICES)

### Overview
Medicare Online is the primary channel for real-time claim submission, patient verification, and AIR notifications. It uses **PRODA** (Provider Digital Access) for authentication and **web services** for B2B integration.

**Supported Functions:**
- Bulk bill claims (real-time & store-and-forward)
- Patient claims (real-time & store-and-forward)
- Same-day delete requests (real-time patient claims only)
- Department of Veterans' Affairs (DVA) medical & pathology claims
- Australian Immunisation Register (AIR) notifications
- Online patient verification (Medicare & private health insurers)
- ECLIPSE (in-hospital claims)
- Eligibility checking

### Test Environment Access

#### Prerequisites
1. **PRODA Account** (Individual)
   - Register at: https://proda.humanservices.gov.au/
   - Requires 3 forms of ID (passport, driver's licence, Medicare card)
   - Takes ~10 minutes
   - Provides PRODA RA number (sent via email)

2. **Organisation Registration** (if representing a practice/organisation)
   - Authorised officer registers organisation in PRODA
   - Links organisation to Medicare Online service
   - Provides Minor ID (Software ID / Location ID) — unique per location
   - Can have multiple Minor IDs per organisation

3. **NASH PKI Test Certificate**
   - Email: `developerliaison@servicesaustralia.gov.au`
   - Request test kit with explanation of use case
   - Receive test certificate package (2 test organisations included)
   - Valid for **2 years**
   - Includes:
     - Active test NASH PKI certificate for Healthcare Provider Organisations (×2)
     - Revoked test certificate (optional, for testing revocation scenarios)
     - Supporting Organisation certificate (optional)

#### Test Environment Details
- **Portal:** https://healthsoftware.humanservices.gov.au/claiming/ext-vnd/
- **Support Hours:** 8:30am–5:00pm Monday–Friday (Eastern Standard Time)
- **Support Email:** `hi.itest@servicesaustralia.gov.au` (HI Service), `developerliaison@servicesaustralia.gov.au` (general)
- **Test Data:** Provided upon registration; test patients/providers only
- **Connectivity:** Web services (SOAP/REST) over HTTPS with certificate-based authentication

#### Self-Service Testing
✅ **Available:** Vendors can test independently in the SVT environment using provided test data.

---

## 2. HEALTHCARE IDENTIFIERS (HI) SERVICE

### Overview
The HI Service provides access to healthcare identifiers (IHI for individuals, HPI-I for practitioners, HPI-O for organisations). Required for patient lookup and clinical document exchange.

### Test Environment Access

#### Prerequisites
1. **PRODA Account** (same as Medicare Online)

2. **Health Systems Developer Portal Registration**
   - URL: https://healthsoftware.humanservices.gov.au/claiming/ext-vnd/register-company-account
   - Authorised officer registers organisation
   - Integrated Developer Management Office (IDMO) review required
   - **Timeline:** 2–3 weeks for approval

3. **NASH PKI Test Certificate** (same as Medicare Online)
   - Request via: `developerliaison@servicesaustralia.gov.au`
   - Includes test HPI-O and HPI-I identifiers

#### Test Environment Details
- **Portal:** https://healthsoftware.humanservices.gov.au/claiming/ext-vnd/
- **Support Hours:** 8:30am–5:00pm Monday–Friday (Eastern Standard Time)
- **Support Email:** `hi.itest@servicesaustralia.gov.au`
- **Test Data:** Provided upon registration
- **Connectivity:** B2B web services (SOAP) with NASH PKI certificate authentication

#### Mandatory Testing
Two types of testing **required** before production access:

1. **Notice of Connection (NoC) Testing**
   - Formal test conducted by Services Australia
   - Ensures software can connect to HI Service and perform operations correctly
   - Conducted via Developer Testing Support System (DTSS)
   - Access via portal → Certification tile → Integration Testing → Apply for Certification

2. **HI Service Conformance Testing**
   - Conducted by ADHA Conformance test team
   - Ensures software safely implements healthcare identifiers in clinical environments
   - Steps:
     - Read: [Use of Healthcare Identifiers in Health Software Systems - FAQs](https://developer.digitalhealth.gov.au/resources/hi-service-faqs)
     - Complete: Implementation Conformance Statement (v5.0.1)
     - Self-assess: Conformance Test Specification (v5.0.1) using provided test data
     - Submit: Both documents to `help@digitalhealth.gov.au`
     - ADHA reviews and schedules observed assessment
   - Upon success: ADHA issues Conformance Declaration
   - **Timeline:** 2–4 weeks after submission

#### Production Access
- Granted within **2 weeks** of submitting conformance documentation to Services Australia
- Requires signed Conformance Declaration from ADHA

---

## 3. MY HEALTH RECORD (ADHA)

### Overview
My Health Record is the national electronic health record system. Software vendors must integrate via the **B2B Gateway** (or FHIR Gateway for newer implementations) to upload clinical documents and access patient records.

### Test Environment Access

#### Prerequisites
1. **PRODA Account** (same as above)

2. **Health Systems Developer Portal Registration** (same as HI Service)
   - URL: https://healthsoftware.humanservices.gov.au/claiming/ext-vnd/register-company-account
   - IDMO review required
   - **Timeline:** 2–3 weeks

3. **NASH PKI Test Certificate**
   - Request via: `developerliaison@servicesaustralia.gov.au`
   - Includes test HPI-O and HPI-I
   - Valid for **2 years**

4. **Test Healthcare Identifiers**
   - Test HPI-O (Healthcare Provider Identifier–Organisation)
   - Test HPI-I (Healthcare Provider Identifier–Individual)
   - Embedded in test NASH PKI certificate

#### Test Environment Details
- **Environment Name:** Software Vendor Test (SVT)
- **Portal:** https://healthsoftware.humanservices.gov.au/claiming/ext-vnd/
- **Support Hours:** 8:30am–5:00pm Monday–Friday (Eastern Standard Time)
- **Support Email:** `help@digitalhealth.gov.au`
- **Test Data:** Provided upon registration; test patients/documents only
- **Connectivity:** B2B Gateway (SOAP) or FHIR Gateway (REST) with NASH PKI certificate authentication
- **MFA Update (May 2026):** SVT environment supporting National Consumer Portal (NCP) will introduce new multi-factor authentication (MFA) from **15 May 2026**

#### Mandatory Testing
Two types of testing **required** before production access:

1. **Notice of Connection (NoC) Testing**
   - Formal test conducted by Services Australia
   - Ensures software can connect to My Health Record B2B Gateway and perform operations
   - Conducted via Developer Testing Support System (DTSS)
   - Access via portal → Certification tile → Integration Testing → Apply for Certification

2. **Conformance, Compliance & Declaration (CCD) Testing**
   - Conducted by ADHA Conformance test team
   - Ensures software conforms to My Health Record technical specifications
   - Steps:
     - Complete: Conformance Vendor Declaration Form
     - Submit: To `help@digitalhealth.gov.au`
     - ADHA reviews and schedules observed assessment
   - Upon success: ADHA issues Conformance Declaration
   - **Timeline:** 2–4 weeks after submission

#### Production Access
- Granted within **2 weeks** of submitting conformance documentation to Services Australia
- Requires signed Conformance Declaration from ADHA

---

## 4. CLAIMING.COM.AU API (THIRD-PARTY ALTERNATIVE)

### Overview
Claiming.com.au is a **third-party wrapper** around Medicare Online, DVA, and ECLIPSE claiming. It provides a modern REST API alternative to direct web services integration. **Optional** — not required by Services Australia but used by some vendors.

### Test Environment Access

#### Prerequisites
1. **Contact Claiming.com.au**
   - Email: `support@claiming.com.au`
   - Explain use case and request sandbox access

2. **Receive oAuth Credentials**
   - Client ID and secret for sandbox environment
   - Issued by Claiming.com.au (not Services Australia)

#### Test Environment Details
- **Sandbox URL:** https://api.claiming.com.au/dev
- **Demo URL (public):** http://api.claiming.com.au/demo (no auth required, limited functionality)
- **Production URL:** https://api.claiming.com.au/v1
- **Support Email:** `support@claiming.com.au`
- **Test Data:** Provided upon request; test patients/providers only
- **Connectivity:** REST API (JSON) with oAuth 2.0 authentication
- **Features:**
  - Medicare verification & claims
  - DVA verification & claims
  - ECLIPSE claims
  - AIR immunisation recording
  - Payment status checking
  - Simulation/validation endpoints

#### Self-Service Testing
✅ **Available:** Vendors can test independently in sandbox using provided test data.

#### Production Access
- Issued by Claiming.com.au after integration testing complete
- Separate from Services Australia/ADHA production access
- Requires vendor to have production access to Medicare Online (via Services Australia)

---

## 5. PRODA & NASH PKI CERTIFICATE MANAGEMENT

### PRODA (Provider Digital Access)

**Purpose:** Online identity verification and authentication system for accessing government health services.

**Registration:**
- URL: https://proda.humanservices.gov.au/
- Individual account required (10 minutes)
- Provides PRODA RA number (unique identifier)

**Linking to Services:**
- Log in to PRODA
- Add Medicare Online, HI Service, My Health Record as linked services
- Accept Linking Terms and Conditions
- Provide organisation Minor ID (for Medicare Online)
- Provide PKI RA number (if organisation has legacy PKI certificate)

**Support:**
- Phone: 1800 700 199 (option 1)
- Email: `proda@servicesaustralia.gov.au`
- Hours: 8am–5pm Monday–Friday

### NASH PKI Test Certificates

**What They Are:**
- Public Key Infrastructure (PKI) certificates for secure authentication
- Replace legacy Medicare PKI certificates
- Used for B2B (unattended) integration with My Health Record, HI Service, electronic prescribing, secure messaging

**How to Obtain:**
1. Email: `developerliaison@servicesaustralia.gov.au`
2. Explain use case and request test kit
3. Receive certificate package (within days)
4. Install certificates locally (Windows/Linux/macOS)

**Certificate Contents:**
- Active test NASH PKI certificate for Healthcare Provider Organisations (×2)
- Revoked test certificate (optional, for testing revocation)
- Supporting Organisation certificate (optional)
- Personal Identification Code (PIC) for each certificate
- CA certificate chain (for TLS validation)

**Validity:**
- **2 years** from issue date
- Renewal: Contact `developerliaison@servicesaustralia.gov.au` at least 1 month before expiry

**Certificate Compatibility Matrix:**

| Certificate Type | My Health Record B2B | NASH Directory | HI Service B2B | Secure Messaging |
|---|---|---|---|---|
| NASH PKI for Healthcare Provider Organisations (1.20.1.1) | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes |
| NASH PKI for Supporting Organisations (1.22.1.1) | ✅ Yes | ✅ Yes | ✅ Yes | ❌ No |
| Individual PRODA | ❌ No | ✅ Yes | ❌ No | ✅ Yes |

**Production Certificates:**
- Obtained via HPOS (Health Professional Online Services)
- Different CA certificates than test environment
- Download from: https://www.certificates-australia.com.au/

---

## 6. TESTING ROADMAP & TIMELINES

### Typical Vendor Onboarding Timeline

| Phase | Activity | Duration | Notes |
|---|---|---|---|
| **Week 1** | Create PRODA account | 10 min | Individual registration |
| **Week 1** | Register organisation in Health Systems Developer Portal | 20 min | Authorised officer required |
| **Week 2–3** | IDMO review of organisation registration | 1–2 weeks | Services Australia review |
| **Week 3–4** | Request NASH PKI test certificates | 1–3 days | Email `developerliaison@servicesaustralia.gov.au` |
| **Week 4** | Install test certificates locally | 1 day | Platform-specific setup |
| **Week 4–8** | Development & self-service testing | 2–4 weeks | Use SVT environment independently |
| **Week 8–10** | Apply for NoC testing via DTSS | 1–2 weeks | Services Australia schedules test |
| **Week 10–12** | Complete NoC testing | 1–2 weeks | Formal test with Services Australia |
| **Week 12–14** | Submit Conformance documentation | 1–2 weeks | ADHA review |
| **Week 14–16** | ADHA Conformance assessment | 1–2 weeks | Observed testing |
| **Week 16–18** | Production access granted | 1–2 weeks | After ADHA declaration submitted |

**Total:** 4–5 months from PRODA registration to production access.

---

## 7. SELF-SERVICE VS. VENDOR CONFORMANCE TESTING

### Self-Service Testing (Vendor-Controlled)
✅ **Available for:**
- Medicare Online (PRODA/Web Services) — unlimited testing in SVT
- HI Service — unlimited testing in SVT
- My Health Record — unlimited testing in SVT
- Claiming.com.au API — unlimited testing in sandbox

**Characteristics:**
- No formal approval required
- Use provided test data
- Test independently in vendor environment
- No time limits
- Can test error scenarios, edge cases, revocation, etc.

### Vendor Conformance Testing (Mandatory)
❌ **Not self-service:**
- **Notice of Connection (NoC)** — Formal test conducted by Services Australia
- **Conformance Testing (HI/MHR)** — Formal test conducted by ADHA

**Characteristics:**
- Scheduled via formal process (DTSS portal or submission)
- Conducted by government agency
- Observed assessment (ADHA)
- Pass/fail determination
- Required for production access
- Timeline: 2–4 weeks per test type

---

## 8. PRACTICAL CONSTRAINTS & CAVEATS

### Public Documentation Limitations
⚠️ **Known Gaps:**
- Exact test data specifications not fully public (provided upon registration)
- Detailed API specifications require portal access (not publicly available)
- Specific error codes/rejection reasons documented in portal only
- Performance/load testing guidelines not publicly documented

### Certificate & PKI Constraints
- Test certificates **cannot** be used in production
- Production certificates require separate request via HPOS
- Certificate installation is **platform-specific** (Windows/Linux/macOS differ)
- Lost/compromised certificates require revocation + replacement (1–3 days)
- PIC (Personal Identification Code) locked after 3 incorrect attempts

### Environment Constraints
- SVT environment **not guaranteed** to match production exactly
- Maintenance windows: Services Australia notifies vendors via email
- Test data is **limited** (small pool of test patients/providers)
- No load testing in SVT (production-like performance not guaranteed)
- Claiming.com.au sandbox connects to Medicare's test environment (not Services Australia SVT directly)

### Compliance & Legal
- NASH PKI test kit requires **terms & conditions acceptance**
- Healthcare Identifiers Act 2010 governs use of healthcare identifiers
- Certificates must be used **only for healthcare provision**
- Audit logging required for all certificate usage

---

## 9. CONTACT INFORMATION & SUPPORT CHANNELS

### Services Australia (Medicare Online, HI Service, My Health Record)
- **General Developer Liaison:** `developerliaison@servicesaustralia.gov.au`
- **HI Service Testing:** `hi.itest@servicesaustralia.gov.au`
- **PRODA Support:** 1800 700 199 (option 1), `proda@servicesaustralia.gov.au`
- **Hours:** 8:30am–5:00pm Monday–Friday (Eastern Standard Time)

### ADHA (My Health Record, HI Service Conformance)
- **Developer Support:** `help@digitalhealth.gov.au`
- **Developer Hub:** https://developer.digitalhealth.gov.au/
- **Hours:** Business hours (specific times not published)

### Claiming.com.au (Third-Party API)
- **Support:** `support@claiming.com.au`
- **Documentation:** https://docs.claiming.com.au/
- **Demo Walkthrough:** https://walkthrough.claiming.com.au/

---

## 10. REFERENCE DOCUMENTATION

### Official Sources
- **Services Australia Medicare Online:** https://www.servicesaustralia.gov.au/medicare-online-for-software-developers
- **HI Service Registration & Certificates:** https://developer.digitalhealth.gov.au/resources/hi-service-registration-and-certificates
- **HI Service Test & Go Live:** https://developer.digitalhealth.gov.au/resources/hi-service-test-and-go-live
- **NASH PKI for Vendors:** https://www.servicesaustralia.gov.au/software-vendors-and-developers-for-nash-pki
- **PRODA for Web Services:** https://hpe.servicesaustralia.gov.au/HTML/PRODA/PRODAM02.htm
- **ADHA Digital Health Implementer Hub:** https://developer.digitalhealth.gov.au/

### Claiming.com.au
- **API Documentation:** https://docs.claiming.com.au/
- **Public Demo:** https://walkthrough.claiming.com.au/
- **Security & Environments:** https://dochub.claiming.com.au/docs/security.html

---

## 11. QUICK REFERENCE: WHICH CHANNEL TO USE?

| Use Case | Channel | Test Environment | Mandatory Testing |
|---|---|---|---|
| Real-time Medicare claim submission | Medicare Online (PRODA) | SVT | NoC only |
| Patient verification (Medicare) | Medicare Online (PRODA) | SVT | NoC only |
| AIR immunisation notifications | Medicare Online (PRODA) | SVT | NoC only |
| DVA claims | Medicare Online (PRODA) or Claiming.com.au | SVT or Claiming sandbox | NoC only |
| ECLIPSE in-hospital claims | Medicare Online (PRODA) or Claiming.com.au | SVT or Claiming sandbox | NoC only |
| Healthcare identifier lookup (IHI, HPI-I, HPI-O) | HI Service | SVT | NoC + Conformance |
| Clinical document upload/access | My Health Record | SVT | NoC + Conformance |
| Secure messaging | My Health Record | SVT | NoC + Conformance |
| Modern REST API alternative | Claiming.com.au | Sandbox | None (vendor-managed) |

---

## 12. KNOWN GAPS & FUTURE CONSIDERATIONS

### As of April 2026
- **Medicare/PBS/AIR integrations:** MBS XML importer exists, but end-to-end Medicare claiming and AIR reporting not fully automated
- **Security roadmap:** Auth hardening and broader compliance automation still evolving
- **API maturity:** Services Australia web services stable; ADHA FHIR Gateway still in development
- **Load testing:** No public load testing environment; vendors must estimate production capacity
- **Sandbox parity:** SVT environment may lag production by 1–2 months during major releases

### Recommended Monitoring
- Subscribe to ADHA notifications: https://developer.digitalhealth.gov.au/about/service-status
- Monitor Services Australia site notices: https://www.servicesaustralia.gov.au/site-notices
- Follow Claiming.com.au release notes: https://docs.claiming.com.au/docs/introduction/version-notes/

---

**Document Version:** 1.0  
**Last Verified:** April 2026  
**Prepared for:** OpenGP Engineering Planning

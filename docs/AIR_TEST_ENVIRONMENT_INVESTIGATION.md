# Australian Immunisation Register (AIR) Test/Sandbox Environment Investigation

**Date:** April 2026  
**Status:** Authoritative summary based on official Services Australia and ADHA sources  
**Scope:** Test environments, integration channels, onboarding, capabilities, and common blockers

---

## EXECUTIVE SUMMARY

The Australian Immunisation Register (AIR) does **not have a dedicated standalone test/sandbox environment**. Instead, testing is conducted through:

1. **Medicare Online web services** (primary integration channel) via the **Health Systems Developer Portal** vendor environment
2. **Third-party API providers** (e.g., Claiming.com.au) offering sandbox/demo environments
3. **Direct HPOS/AIR site access** for manual testing (no automation)

AIR is **tightly coupled to Medicare Online claiming infrastructure** and uses the same authentication (PRODA) and web services framework. There is **no separate AIR-specific test environment** managed by Services Australia.

---

## 1. INTEGRATION CHANNELS & REPORTING REQUIREMENTS

### 1.1 Mandatory Reporting Requirements

Under the **Australian Immunisation Register Act 2015**, vaccination providers must report:

- **COVID-19 vaccines** (administered on or after 20 February 2021)
- **Influenza vaccines** (administered on or after 1 March 2021)
- **National Immunisation Program (NIP) vaccines** (administered on or after 1 July 2021)
- **Japanese Encephalitis Virus (JEV) vaccines** (administered on or after 21 December 2022)
- **Antenatal status** (for above vaccines, from 1 March 2025 onwards)

**Reporting methods:**
- Electronic (preferred): via PMS/vaccination software or AIR site
- Written form (fallback): if electronic is not reasonably practical

### 1.2 Primary Integration Channel: Medicare Online Web Services

**AIR is integrated into Medicare Online**, not as a standalone service.

| Aspect | Details |
|--------|---------|
| **Channel** | Medicare Online (MCOL) web services |
| **Authentication** | PRODA (Provider Digital Access) |
| **Scope** | Transmit immunisation data alongside Medicare claims |
| **Requirement** | Web services-compatible PMS/vaccination software (mandatory from 31 March 2023) |
| **Legacy** | Adaptor technology no longer supported (deprecated 31 March 2023) |

**Related channels:**
- **ECLIPSE**: Extension of Medicare Online for in-hospital claiming (includes AIR functionality)
- **DVA**: Department of Veterans' Affairs claiming (integrated with Medicare Online)

### 1.3 Access Methods for Vaccination Providers

1. **PMS/Vaccination Software** (automated)
   - Web services-enabled software submits immunisation data automatically
   - Data transmitted to AIR when service is completed
   - Requires PRODA organisation account linked to Medicare Online/ECLIPSE/DVA/AIR

2. **AIR Site via HPOS** (manual)
   - Access via Health Professional Online Services (HPOS)
   - Requires individual PRODA account
   - Functions: view history, record encounters, manage exemptions, generate reports
   - No API access; web UI only

3. **Form Upload via HPOS** (manual, slow)
   - AIR Immunisation History Form (IM013) for domestic vaccines
   - AIR Immunisation Encounter Form (IM018) for overseas vaccines
   - Processing time: up to 14 business days

---

## 2. TEST ENVIRONMENTS: WHAT EXISTS

### 2.1 Health Systems Developer Portal (Services Australia)

**URL:** https://healthsoftware.humanservices.gov.au/claiming/ext-vnd/

**Purpose:** Central hub for software developers integrating with Medicare Online, ECLIPSE, DVA, and AIR.

**What it provides:**
- Vendor (test) environment access for Medicare Online/ECLIPSE/DVA/AIR
- Interface Agreement acceptance
- API documentation (licensed material)
- Developer Testing and Support System (DTSS) for Notice of Connection (NoC) testing
- Test certificates and test data upon registration

**Registration process:**
1. Register organisation on the portal
2. Authorised officer completes registration with PRODA account
3. Accept Interface Agreement
4. Services Australia contacts you to discuss requirements
5. Receive test certificates and test data

**Support:**
- Online Technical Support (OTS) Desk: 1300 550 115
- Business hours: 8:30am–5:00pm Monday–Friday (Eastern Standard Time)
- Email: developerliaison@servicesaustralia.gov.au

**Current status (April 2026):**
- Vendor environment available with business hours support
- Quarterly software developer information sessions (next: May 2026)
- Some channels experiencing degradation (PHIR Registration, Aged Care)

### 2.2 Claiming.com.au AIR API (Third-Party Provider)

**URL:** https://air.claiming.com.au/

**Purpose:** Third-party sandbox/demo environment for AIR integration testing.

**What it provides:**
- **V2 testing environment** for AIR features
- Dummy patient test data (non-production)
- OAuth credentials for API access
- Comprehensive API documentation (Postman-ready)
- Support for:
  - Recording immunisations
  - Fetching individual details
  - Medical exemptions
  - Catch-up dates
  - Risk factors and indigenous status
  - Immunisation history (PDF and detailed)

**Getting started:**
1. Contact support@claiming.com.au for OAuth credentials
2. Request test data for dummy patients
3. Use Postman or preferred API tool
4. Test against demo/sandbox servers (dummy patients only)
5. Upon completion, request production credentials

**Limitations:**
- Demo and sandbox servers access **dummy patients only**
- Not an official Services Australia environment
- Separate from Medicare Online web services integration

### 2.3 HPOS/AIR Site (Manual Testing Only)

**URL:** Via Health Professional Online Services (HPOS)

**Purpose:** Manual web UI for testing AIR functionality without automation.

**What it provides:**
- View immunisation history
- Record immunisation encounters
- Manage medical exemptions
- Generate immunisation reports
- Check immunisation claims and payments
- Print immunisation history statements

**Limitations:**
- No API access
- No automation capability
- Manual data entry only
- Suitable for functional testing, not integration testing

---

## 3. ONBOARDING STEPS FOR SOFTWARE INTEGRATION

### 3.1 For Medicare Online/AIR Web Services Integration

**Step 1: Register on Health Systems Developer Portal**
- URL: https://healthsoftware.humanservices.gov.au/claiming/ext-vnd/
- Requires: Authorised officer + PRODA individual account
- Time: ~10 minutes for PRODA registration

**Step 2: Accept Interface Agreement**
- Review and digitally accept Interface Agreement
- Integrated third-party security policy applies

**Step 3: Receive Test Certificates & Data**
- Services Australia contacts you within days
- Provides: NASH test certificates, test data, technical resources
- Certificates required for SOAP/web services calls

**Step 4: Develop Against Vendor Environment**
- Use test certificates to connect to vendor (test) environment
- Test data provided for integration testing
- Endpoints: Vendor environment URLs (separate from production)

**Step 5: Notice of Connection (NoC) Testing**
- Conducted by Services Australia using DTSS
- Confirms your software connects successfully
- Organised via Health Systems Developer Portal
- Steps:
  1. Log in to portal
  2. Click 'View Certification' in Certification tile
  3. Go to Developer Testing and Support System (DTSS)
  4. Under 'Integration Testing', click 'Untested' tab
  5. Select 'Apply for NoC testing'
  6. Fill in required fields and submit
  7. Services Australia Developer Liaison team contacts you

**Step 6: Production Access**
- Upon successful NoC testing, production access granted
- Typically within 2 weeks
- Production certificates issued

### 3.2 For Third-Party API Integration (Claiming.com.au)

**Step 1: Contact Support**
- Email: support@claiming.com.au
- Request: OAuth credentials for sandbox/demo environment

**Step 2: Set Up API Client**
- Use Postman or preferred API tool
- Import Claiming.com.au API collections
- Configure OAuth credentials

**Step 3: Test Against Dummy Patients**
- Demo server: dummy patient data only
- Sandbox server: dummy patient data only
- Request additional test data if needed

**Step 4: Integration Testing**
- Test all AIR functions (record, fetch, exemptions, etc.)
- Verify error handling and edge cases

**Step 5: Production Credentials**
- Upon completion, request production OAuth credentials
- Claiming.com.au issues credentials for production environment

---

## 4. WHAT CAN & CANNOT BE TESTED

### 4.1 What CAN Be Tested

#### In Medicare Online Vendor Environment (Services Australia)
- ✅ Web services connectivity and authentication
- ✅ Immunisation data transmission format and validation
- ✅ Error handling and response codes
- ✅ Integration with Medicare Online claiming workflow
- ✅ PRODA authentication and authorisation
- ✅ Test data provided by Services Australia

#### In Claiming.com.au Sandbox
- ✅ AIR API endpoints (record, fetch, exemptions, etc.)
- ✅ OAuth authentication flow
- ✅ Immunisation history retrieval
- ✅ Medical exemption management
- ✅ Risk factors and indigenous status updates
- ✅ PDF report generation
- ✅ Error scenarios and edge cases

#### In HPOS/AIR Site (Manual)
- ✅ User workflows (view history, record encounters)
- ✅ Report generation
- ✅ Exemption recording
- ✅ Data visibility and permissions
- ✅ PRODA access and linking

### 4.2 What CANNOT Be Tested

#### NOT Available in Test Environments
- ❌ **Real patient data** (test environments use dummy/synthetic data only)
- ❌ **Real Medicare integration** (vendor environment is isolated)
- ❌ **Real AIR database** (test data is separate)
- ❌ **Real payment processing** (no financial transactions in test)
- ❌ **Real PRODA accounts** (test uses separate test certificates)
- ❌ **Real HI Service integration** (separate test environment required)
- ❌ **Real My Health Record integration** (separate test environment required)
- ❌ **Bulk historical data migration** (not supported in test)
- ❌ **Performance/load testing** (vendor environment not designed for this)
- ❌ **Production-scale data volumes** (test environment has limits)

#### Limitations by Channel
| Channel | Limitation |
|---------|-----------|
| Medicare Online Vendor | Isolated from production; test data only; limited scale |
| Claiming.com.au | Dummy patients only; third-party, not official; separate from Medicare Online |
| HPOS/AIR Site | Manual only; no API; no automation; slow form processing (14 days) |

---

## 5. COMMON BLOCKERS & TROUBLESHOOTING

### 5.1 Registration & Access Blockers

| Blocker | Cause | Resolution |
|---------|-------|-----------|
| Cannot register on Health Systems Developer Portal | Missing PRODA individual account | Register for PRODA first (10 minutes); requires authorised officer status |
| Interface Agreement not accessible | Not logged in or permissions issue | Log in with PRODA account; ensure authorised officer role |
| Test certificates not received | Services Australia hasn't processed request | Contact developerliaison@servicesaustralia.gov.au; allow 3–5 business days |
| DTSS (Developer Testing and Support System) not accessible | Portal access not fully provisioned | Ensure organisation registration complete; contact OTS Desk (1300 550 115) |

### 5.2 Integration & Testing Blockers

| Blocker | Cause | Resolution |
|---------|-------|-----------|
| Web services connection fails | Wrong endpoint URL or test certificates not installed | Verify vendor environment endpoint; install NASH test certificates correctly |
| SOAP/XML parsing errors | Incorrect message format or schema version | Review AIR CDA Implementation Guide v1.0 or v1.2; validate against schema |
| Immunisation data rejected by AIR | Missing mandatory fields (e.g., dose number, valid postcode) | Ensure patient has valid Medicare details and postcode; check vaccine code validity |
| Test data not available | Services Australia hasn't provided test data | Request from developerliaison@servicesaustralia.gov.au; specify use case |
| NoC testing fails | Software doesn't meet conformance requirements | Review DTSS feedback; fix issues; resubmit for testing |
| Claiming.com.au OAuth fails | Credentials not issued or expired | Contact support@claiming.com.au; request new credentials |

### 5.3 Data & Validation Blockers

| Blocker | Cause | Resolution |
|---------|-------|-----------|
| Patient not found in AIR | Patient not yet registered or IHI mismatch | Verify patient IHI (Individual Health Identifier); check HI Service integration |
| Vaccine code not recognised | Invalid or outdated vaccine code | Use current SNOMED CT vaccine codes; check AIR vaccine code list |
| Medical exemption rejected | Practitioner not eligible or exemption type invalid | Verify practitioner credentials; use approved exemption types (medical contraindication, natural immunity) |
| Immunisation history not updating | Data not transmitted or processing delay | Check AIR Claims tab in PMS; verify service completion; allow 24 hours for processing |
| Form upload takes 14 days | Manual processing queue | Use web services integration instead; forms are fallback only |

### 5.4 Operational Blockers

| Blocker | Cause | Resolution |
|---------|-------|-----------|
| Vendor environment unavailable | Scheduled maintenance or outage | Check Health Systems Developer Portal announcements; subscribe to OTS notifications |
| OTS support unavailable | Outside business hours (8:30am–5:00pm EST Mon–Fri) | Use portal self-service; email developerliaison@servicesaustralia.gov.au for async support |
| PRODA account locked | Multiple failed login attempts | Contact PRODA Helpdesk; provide organisation details |
| PMS not web services ready | Legacy adaptor-based software | Upgrade to web services-compatible version (mandatory from 31 March 2023) |

---

## 6. RELATIONSHIP: AIR REPORTING vs. MEDICARE/SERVICES AUSTRALIA INTEGRATION

### 6.1 Architectural Relationship

```
┌─────────────────────────────────────────────────────────────┐
│                    Services Australia                        │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │         Medicare Online Web Services                 │  │
│  │  (PRODA authentication, web services framework)      │  │
│  │                                                      │  │
│  │  ├─ Medicare Bulk Bill Claiming                     │  │
│  │  ├─ Medicare Patient Claiming                       │  │
│  │  ├─ DVA Claiming                                    │  │
│  │  └─ AIR Immunisation Reporting ◄─── (integrated)   │  │
│  │                                                      │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │         ECLIPSE (extension of Medicare Online)       │  │
│  │  (In-hospital claiming + AIR functionality)          │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │    Health Professional Online Services (HPOS)        │  │
│  │  (Manual web UI for AIR, Medicare, DVA access)       │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│         Australian Digital Health Agency (ADHA)             │
│                                                              │
│  ├─ My Health Record (separate test environment)            │
│  ├─ Healthcare Identifiers (HI Service) (separate test)     │
│  └─ AIR Specifications (CDA Implementation Guide v1.2)      │
└─────────────────────────────────────────────────────────────┘
```

### 6.2 Key Points

1. **AIR is NOT a standalone service**: It is integrated into Medicare Online web services.
2. **Single authentication**: PRODA is used for all channels (Medicare Online, ECLIPSE, DVA, AIR).
3. **Single vendor environment**: Health Systems Developer Portal provides one test environment for all channels.
4. **Shared infrastructure**: AIR uses the same web services framework as Medicare claiming.
5. **Separate specifications**: ADHA publishes AIR CDA Implementation Guide (v1.2 current); Services Australia manages web services integration.
6. **Separate test environments**: HI Service and My Health Record have their own test environments (not shared with AIR).

### 6.3 Integration Flow

```
PMS/Vaccination Software
    │
    ├─ Records immunisation encounter
    ├─ Completes service
    │
    └─ Transmits via Medicare Online web services
        │
        ├─ PRODA authentication
        ├─ SOAP/XML message
        │
        └─ Services Australia
            │
            ├─ Validates against AIR schema
            ├─ Checks mandatory fields
            │
            └─ Stores in AIR database
                │
                └─ Available in AIR site (HPOS)
                   and via AIR API (if applicable)
```

---

## 7. CURRENT STATUS & KNOWN GAPS (April 2026)

### 7.1 Operational Status

| Component | Status | Notes |
|-----------|--------|-------|
| Health Systems Developer Portal | ✅ Operational | Business hours support; some channels degraded |
| Medicare Online vendor environment | ✅ Operational | Supported 8:30am–5:00pm EST Mon–Fri |
| DTSS (Notice of Connection testing) | ✅ Operational | Available via portal |
| HPOS/AIR Site | ✅ Operational | Manual access; no API |
| Claiming.com.au sandbox | ✅ Operational | Third-party; dummy patients only |

### 7.2 Known Gaps & Limitations

1. **No dedicated AIR test environment**: AIR testing is bundled with Medicare Online; no isolated AIR-only vendor environment.

2. **Limited test data**: Services Australia provides basic test data; complex scenarios may require custom test data requests.

3. **No performance testing**: Vendor environment not suitable for load/performance testing; production-scale testing not available pre-go-live.

4. **Manual form processing slow**: Form upload via HPOS takes up to 14 days; not suitable for rapid iteration.

5. **Third-party API gap**: Claiming.com.au is not an official Services Australia environment; integration differs from Medicare Online web services.

6. **HI Service integration separate**: If your software also needs HI Service (Healthcare Identifiers), separate test environment and registration required.

7. **My Health Record integration separate**: If your software uploads AIR data to My Health Record, separate test environment and registration required.

8. **No real-time validation**: Test environment may not catch all production issues; NoC testing is mandatory before go-live.

---

## 8. RECOMMENDED TESTING STRATEGY FOR OPENGP

### 8.1 Phase 1: Development & Unit Testing
- Use Claiming.com.au sandbox for API exploration
- Mock Medicare Online web services locally
- Test AIR data validation against CDA Implementation Guide v1.2

### 8.2 Phase 2: Integration Testing
- Register on Health Systems Developer Portal
- Obtain test certificates and test data
- Develop against vendor environment
- Test web services connectivity and message format
- Verify PRODA authentication flow

### 8.3 Phase 3: Conformance Testing
- Submit Notice of Connection (NoC) testing request via DTSS
- Services Australia tests your software
- Fix any conformance issues
- Receive production access approval

### 8.4 Phase 4: Manual Testing
- Use HPOS/AIR Site to verify data visibility
- Test user workflows (view history, record encounters, exemptions)
- Verify report generation

### 8.5 Phase 5: Production Deployment
- Use production certificates (issued after NoC testing)
- Connect to production Medicare Online web services
- Monitor AIR Claims tab for successful uploads
- Verify data appears in AIR site within 24 hours

---

## 9. REFERENCES & OFFICIAL SOURCES

### Services Australia (Official)
- **Health Systems Developer Portal**: https://healthsoftware.humanservices.gov.au/claiming/ext-vnd/
- **Medicare Online for Software Developers**: https://www.servicesaustralia.gov.au/medicare-online-for-software-developers
- **Software for Medicare Online, ECLIPSE and AIR**: https://www.servicesaustralia.gov.au/software-for-medicare-online-eclipse-and-air
- **Web Services for Digital Health Channels**: https://www.servicesaustralia.gov.au/web-services-for-digital-health-channels
- **Access the Australian Immunisation Register**: https://www.servicesaustralia.gov.au/access-australian-immunisation-register
- **AIRM03 - Access to AIR**: https://hpe.servicesaustralia.gov.au/TRANS/AIR/AIRM03.htm
- **AIRM04 - Submitting Information to AIR**: https://hpe.servicesaustralia.gov.au/TRANS/AIR/AIRM04.htm

### Australian Digital Health Agency (ADHA)
- **Digital Health Developer Portal**: https://developer.digitalhealth.gov.au/
- **Australian Immunisation Register v1.2**: https://developer.digitalhealth.gov.au/resources/australian-immunisation-register-v1-2
- **AIR CDA Implementation Guide v1.0**: Current specification for AIR data structure
- **Testing and Conformance**: https://developer.digitalhealth.gov.au/resources/resource-topics/testing-and-conformance

### Third-Party Providers
- **Claiming.com.au AIR API**: https://air.claiming.com.au/
- **Communicare (Telstra Health)**: AIR integration documentation and release notes

### Legislation
- **Australian Immunisation Register Act 2015**: Defines mandatory reporting requirements
- **Australian Immunisation Register Rule 2015**: Specifies vaccines, data elements, and reporting methods

---

## 10. CONTACT INFORMATION

| Role | Contact | Hours |
|------|---------|-------|
| **Developer Liaison** | developerliaison@servicesaustralia.gov.au | Business hours |
| **Online Technical Support (OTS)** | 1300 550 115 | 8:30am–5:00pm EST Mon–Fri |
| **PRODA Helpdesk** | Via PRODA portal | Business hours |
| **HPOS Helpdesk** | Via HPOS portal | Business hours |
| **Claiming.com.au Support** | support@claiming.com.au | Business hours |
| **ADHA Help Centre** | help@digitalhealth.gov.au | Business hours |

---

## CONCLUSION

The Australian Immunisation Register does not have a dedicated test/sandbox environment. Instead, testing is conducted through:

1. **Medicare Online vendor environment** (primary, official) via Health Systems Developer Portal
2. **Third-party APIs** (e.g., Claiming.com.au) for supplementary testing
3. **HPOS/AIR Site** for manual functional testing

AIR is tightly integrated with Medicare Online and uses the same authentication (PRODA) and web services framework. Onboarding requires registration on the Health Systems Developer Portal, acceptance of Interface Agreement, and completion of Notice of Connection (NoC) testing before production access is granted.

For OpenGP, the recommended approach is to:
1. Register on Health Systems Developer Portal
2. Develop against vendor environment with test certificates
3. Complete NoC testing with Services Australia
4. Deploy to production with production certificates

**Key constraint:** There is no isolated AIR test environment; all testing is conducted within the Medicare Online web services framework.

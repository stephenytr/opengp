# AIR Integration Quick Reference

**Last Updated:** April 2026  
**For:** OpenGP Australian Immunisation Register Integration

---

## TL;DR

- **No dedicated AIR test environment** — AIR testing is bundled with Medicare Online
- **Primary channel:** Medicare Online web services via Health Systems Developer Portal
- **Authentication:** PRODA (Provider Digital Access)
- **Test environment:** Health Systems Developer Portal vendor environment
- **Onboarding:** Register → Accept Interface Agreement → Get test certificates → Develop → NoC testing → Production

---

## Quick Links

| Resource | URL |
|----------|-----|
| **Developer Portal** | https://healthsoftware.humanservices.gov.au/claiming/ext-vnd/ |
| **ADHA Resources** | https://developer.digitalhealth.gov.au/ |
| **AIR Specifications** | https://developer.digitalhealth.gov.au/resources/australian-immunisation-register-v1-2 |
| **Claiming.com.au Sandbox** | https://air.claiming.com.au/ |
| **Services Australia Support** | developerliaison@servicesaustralia.gov.au |
| **OTS Desk** | 1300 550 115 (8:30am–5:00pm EST Mon–Fri) |

---

## Integration Channels

### Medicare Online Web Services (Primary)
- **What:** Integrated AIR reporting within Medicare Online claiming
- **How:** SOAP/XML web services
- **Auth:** PRODA
- **Test:** Health Systems Developer Portal vendor environment
- **Status:** Mandatory from 31 March 2023 (adaptor technology deprecated)

### HPOS/AIR Site (Manual)
- **What:** Web UI for manual AIR access
- **How:** Browser-based
- **Auth:** PRODA
- **Functions:** View history, record encounters, manage exemptions, generate reports
- **Limitation:** No API; manual only

### Claiming.com.au API (Third-Party)
- **What:** Third-party sandbox for AIR API testing
- **How:** REST API with OAuth
- **Test Data:** Dummy patients only
- **Status:** Separate from Medicare Online; not official Services Australia

---

## Mandatory Reporting Requirements

| Vaccine Type | Effective Date | Mandatory |
|--------------|----------------|-----------|
| COVID-19 | 20 Feb 2021 | ✅ Yes |
| Influenza | 1 Mar 2021 | ✅ Yes |
| NIP vaccines | 1 Jul 2021 | ✅ Yes |
| JEV | 21 Dec 2022 | ✅ Yes |
| Antenatal status | 1 Mar 2025 | ✅ Yes |

---

## Onboarding Checklist

- [ ] Register organisation on Health Systems Developer Portal
- [ ] Authorised officer creates PRODA individual account
- [ ] Accept Interface Agreement
- [ ] Receive NASH test certificates (3–5 business days)
- [ ] Receive test data from Services Australia
- [ ] Develop against vendor environment
- [ ] Test web services connectivity
- [ ] Submit Notice of Connection (NoC) testing request via DTSS
- [ ] Services Australia tests your software
- [ ] Fix any conformance issues
- [ ] Receive production access approval (typically 2 weeks)
- [ ] Receive production certificates
- [ ] Deploy to production

---

## What Can Be Tested

✅ Web services connectivity  
✅ Message format and validation  
✅ PRODA authentication  
✅ Error handling  
✅ Immunisation data transmission  
✅ Medical exemptions  
✅ Risk factors and indigenous status  
✅ Immunisation history retrieval  

---

## What CANNOT Be Tested

❌ Real patient data  
❌ Real Medicare integration  
❌ Real AIR database  
❌ Real payment processing  
❌ Performance/load testing  
❌ Production-scale data volumes  
❌ HI Service integration (separate test environment)  
❌ My Health Record integration (separate test environment)  

---

## Common Blockers & Fixes

| Issue | Fix |
|-------|-----|
| No PRODA account | Register at https://www.proda.gov.au/ (10 minutes) |
| Test certificates not received | Email developerliaison@servicesaustralia.gov.au; allow 3–5 days |
| Web services connection fails | Verify vendor endpoint URL; install NASH test certificates |
| SOAP/XML parsing errors | Review AIR CDA Implementation Guide v1.2; validate schema |
| Immunisation data rejected | Ensure patient has valid Medicare details and postcode |
| NoC testing fails | Review DTSS feedback; fix conformance issues; resubmit |
| Vendor environment unavailable | Check portal announcements; contact OTS Desk |

---

## Key Contacts

| Role | Contact | Hours |
|------|---------|-------|
| Developer Liaison | developerliaison@servicesaustralia.gov.au | Business hours |
| OTS Desk | 1300 550 115 | 8:30am–5:00pm EST Mon–Fri |
| ADHA Help | help@digitalhealth.gov.au | Business hours |
| Claiming.com.au | support@claiming.com.au | Business hours |

---

## Architecture Overview

```
OpenGP (PMS)
    ↓
Medicare Online Web Services
    ├─ PRODA authentication
    ├─ SOAP/XML message
    └─ Test: Health Systems Developer Portal vendor environment
        ↓
Services Australia
    ├─ Validates against AIR schema
    ├─ Checks mandatory fields
    └─ Stores in AIR database
        ↓
AIR Site (HPOS)
    └─ Available for manual access
```

---

## Testing Strategy for OpenGP

1. **Phase 1:** Explore Claiming.com.au sandbox (API familiarisation)
2. **Phase 2:** Register on Health Systems Developer Portal
3. **Phase 3:** Develop against vendor environment with test certificates
4. **Phase 4:** Submit NoC testing request
5. **Phase 5:** Fix conformance issues
6. **Phase 6:** Receive production access
7. **Phase 7:** Deploy to production

---

## Key Constraint

**There is no isolated AIR test environment.** All testing is conducted within the Medicare Online web services framework. AIR is not a standalone service; it is integrated into Medicare Online.

---

## References

- **Full Investigation:** See `AIR_TEST_ENVIRONMENT_INVESTIGATION.md`
- **Services Australia:** https://www.servicesaustralia.gov.au/
- **ADHA:** https://www.digitalhealth.gov.au/
- **Legislation:** Australian Immunisation Register Act 2015


# OpenGP Gap Analysis

**Document Version**: 1.0  
**Date**: 2026-02-11  
**Status**: Critical Review Required

---

## Executive Summary

This document identifies critical gaps between the current OpenGP requirements documentation and the comprehensive research findings on Australian GP practice management software. These gaps represent **mandatory features for regulatory compliance and market viability**.

### Impact Assessment

| Category | Missing Items | Priority | Impact |
|----------|--------------|----------|--------|
| **Regulatory Compliance** | AIR integration, Conformance certification | CRITICAL | Cannot operate legally |
| **Clinical Features** | Drug database, Clinical decision support | HIGH | Patient safety risk |
| **Integrations** | Secure messaging, Pathology labs | HIGH | Workflow incomplete |
| **Billing** | WorkCover, PIP/WIP, EFTPOS | HIGH | Revenue loss |
| **Modern Features** | Telehealth, Patient portal | MEDIUM | Market competitiveness |

---

## 1. Drug Database Integration (CRITICAL)

### Current State
❌ **NOT MENTIONED** in requirements

### Required Implementation

**Drug Databases** (choose one or both):
- **MIMS (Monthly Index of Medical Specialties)**: Market leader, used by 90%+ of Australian GPs
  - Subscription required (~$2,000/year)
  - API access via MIMS Online
  - Updated monthly
- **AusDI (Australian Drug Information)**: Alternative, open data
  - Free access
  - Community-maintained
  - Less comprehensive than MIMS

**Core Features Required**:
1. **Drug Interaction Checking**
   - Real-time checking when prescribing
   - Severity levels (minor, moderate, severe, contraindicated)
   - Clinical guidance on interactions
   - Check against patient's current medications

2. **Allergy Cross-Checking**
   - Alert when prescribing contraindicated medication
   - Cross-sensitivity warnings (e.g., penicillin allergy)
   - Severity indicators

3. **Dosage Calculations**
   - Weight-based dosing (especially paediatrics)
   - Age-appropriate dosing
   - Renal/hepatic adjustment suggestions

4. **Drug Information**
   - Generic and brand names
   - Indications and contraindications
   - Side effects and warnings
   - PBS/RPBS status and pricing
   - Pregnancy/breastfeeding categories

5. **60-Day Prescribing Rules**
   - Identify eligible medications
   - Calculate appropriate quantities
   - PBS compliance checking

### Implementation Priority
**PHASE 2** (Clinical Core) - Cannot safely prescribe without drug database

### Estimated Effort
- MIMS API integration: 2-3 weeks
- Drug interaction engine: 3-4 weeks
- UI integration: 1-2 weeks
- **Total**: 6-9 weeks

### Regulatory Impact
- **Not legally required** but standard of care
- **Medical negligence risk** without interaction checking
- **RACGP Standards**: Implicit expectation of clinical decision support

---

## 2. Australian Immunisation Register (AIR) Integration (CRITICAL)

### Current State
❌ **NOT MENTIONED** in requirements

### Required Implementation

**Mandatory Integration** per Commonwealth requirements:

**Core Features**:
1. **Immunisation Recording**
   - Record vaccinations administered
   - Real-time upload to AIR within 24 hours
   - Batch upload for catch-up records
   - Support for all vaccine types (NIP and non-NIP)

2. **AIR History Retrieval**
   - Download patient immunisation history
   - Display in chronological timeline
   - Identify overdue vaccinations
   - Due/upcoming vaccination alerts

3. **Vaccination Schedules**
   - National Immunisation Program (NIP) schedule
   - Catch-up schedule calculator
   - Adult vaccination schedules
   - Travel vaccination tracking

4. **Reporting**
   - Practice immunisation statistics
   - Coverage reports for practice population
   - Medicare incentive payment tracking (PIP)

**Technical Requirements**:
- **SOAP-based web services** via Medicare Online
- **PRODA authentication** required
- **Conformance testing** through Services Australia
- **Error handling** for duplicate notifications

### Implementation Priority
**PHASE 2** (Clinical Core) - Required for PIP payments

### Estimated Effort
- AIR API integration: 2-3 weeks
- Schedule engine: 2 weeks
- UI components: 1-2 weeks
- **Total**: 5-7 weeks

### Regulatory Impact
- **Legally required** for practices receiving PIP immunisation incentives
- **Medicare compliance**: Mandatory for claiming immunisation services
- **Public health reporting**: Commonwealth data collection

---

## 3. Secure Messaging Integration (HIGH PRIORITY)

### Current State
❌ **NOT MENTIONED** in requirements

### Required Implementation

**Secure Message Delivery (SMD)** for clinical communication:

**Networks to Integrate**:
1. **HealthLink** - Market leader (60%+ market share)
2. **Medical Objects** - Second largest
3. **Argus** - Growing market share
4. **Secure eReferral Network (SeNT)** - Government-funded

**Core Features**:
1. **Referral Management**
   - Send electronic referrals to specialists
   - Receive incoming referrals
   - Attachment support (PDF, CDA documents)
   - Delivery confirmation and read receipts

2. **Pathology/Imaging Results**
   - Receive results electronically
   - HL7 v2.x message parsing
   - Automatic patient matching
   - Result acknowledgment

3. **Discharge Summaries**
   - Receive hospital discharge summaries
   - CDA document rendering
   - Automatic filing to patient record

4. **Specialist Letters**
   - Receive consultation notes
   - Two-way communication
   - Thread tracking

**Technical Standards**:
- **SOAP-based web services** (legacy)
- **FHIR messaging** (modern)
- **CDA R2** document format
- **End-to-end encryption**
- **Digital signatures** for authenticity

### Implementation Priority
**PHASE 3** (Prescribing & Billing) or **PHASE 4** (Integrations)

### Estimated Effort
- Per network integration: 3-4 weeks
- Common messaging framework: 2 weeks
- CDA renderer: 2 weeks
- **Total**: 12-16 weeks (for 3 networks)

### Regulatory Impact
- **Not legally required** but industry standard
- **Workflow efficiency**: Manual fax/mail is slow
- **Patient safety**: Faster communication of critical results

---

## 4. Extended Billing Features (HIGH PRIORITY)

### Current State
✅ Basic Medicare and DVA mentioned  
❌ Missing: WorkCover, third-party, EFTPOS, incentive programs

### Required Implementation

**Additional Billing Types**:

1. **WorkCover (Workers' Compensation)**
   - State-specific billing (varies by state)
   - WorkCover item codes
   - Certificate of capacity generation
   - Employer/insurer details
   - Separate claiming process per state

2. **Third-Party Billing**
   - Private health insurance billing
   - Compensation cases (MVA, public liability)
   - Self-insured organizations
   - Invoice generation
   - Payment tracking

3. **EFTPOS Integration**
   - Payment terminal integration
   - Tyro, Smartpay, Westpac terminals
   - Receipt printing
   - End-of-day reconciliation
   - Refund processing

4. **Practice Incentive Program (PIP)**
   - PIP eligibility tracking
   - Quality Improvement payments
   - eHealth incentive
   - Diabetes SIP, Asthma SIP
   - Automated claims

5. **Workforce Incentive Program (WIP)**
   - Rural and remote practice support
   - Tier calculations
   - Quarterly reporting
   - Payment tracking

**Financial Management**:
- Accounts receivable
- Payment plans
- Debt collection integration
- Financial reporting (P&L, cash flow)
- Practice benchmarking

### Implementation Priority
**PHASE 3** (Prescribing & Billing)

### Estimated Effort
- WorkCover (multi-state): 4-6 weeks
- Third-party billing: 2-3 weeks
- EFTPOS integration: 2-3 weeks
- PIP/WIP: 3-4 weeks
- **Total**: 11-16 weeks

### Regulatory Impact
- **WorkCover**: State-specific legal requirements
- **Revenue impact**: Significant (10-20% of practice revenue)

---

## 5. FHIR/HL7 Standards Implementation (HIGH PRIORITY)

### Current State
✅ Mentioned in references only  
❌ No implementation details

### Required Implementation

**FHIR (Fast Healthcare Interoperability Resources)**:

1. **FHIR AU Base Profiles**
   - AU Patient profile
   - AU Practitioner profile
   - AU Organization profile
   - AU Medication profiles
   - AU Immunisation profile

2. **AU Core Implementation**
   - Core data elements
   - Terminology bindings (SNOMED-CT AU, AMT)
   - Identifier systems (Medicare, IHI)
   - Must Support elements

3. **Use Cases**:
   - My Health Record FHIR Gateway integration
   - Pathology/imaging results (FHIR DiagnosticReport)
   - Medication management (FHIR MedicationRequest)
   - Care planning (FHIR CarePlan)

**HL7 v2.x Messaging**:
- **ORU^R01**: Pathology results
- **ORD^O02**: Imaging orders
- **ADT messages**: Patient demographics
- Message parsing and validation
- Error handling

**CDA (Clinical Document Architecture)**:
- **CDA R2** rendering
- **Event Summary** generation
- **Discharge Summary** parsing
- **Referral** documents
- Stylesheet-based rendering

### Implementation Priority
**PHASE 4** (Integrations) for FHIR  
**PHASE 3** (Prescribing & Billing) for basic HL7

### Estimated Effort
- FHIR AU Base: 6-8 weeks
- HL7 v2.x parser: 4-5 weeks
- CDA renderer: 3-4 weeks
- **Total**: 13-17 weeks

### Regulatory Impact
- **My Health Record**: FHIR required for modern integration
- **Interoperability**: National Healthcare Interoperability Plan compliance

---

## 6. Modern Features (MEDIUM PRIORITY)

### Current State
❌ **NOT MENTIONED** in requirements

### Required Implementation

**6.1 Telehealth Integration**

Post-COVID expectation:
- Video consultation platform
- Screen sharing
- Waiting room
- Session recording (with consent)
- Billing integration (MBS telehealth items)
- Integration with Zoom, MS Teams, or dedicated platform

**Regulatory**:
- Privacy Act compliance for video storage
- Informed consent for recording
- Secure video transmission (end-to-end encryption)

**6.2 Patient Portal**

Online patient access:
- View medical history
- Book appointments online
- Request prescriptions
- View test results
- Secure messaging with practice
- Document upload

**6.3 Online Booking**

24/7 appointment booking:
- Real-time availability
- Appointment type selection
- Practitioner preferences
- SMS/email confirmation
- Reminder integration
- Payment deposit (no-show prevention)

**6.4 Mobile App (Practitioner)**

On-the-go access:
- View schedule
- Access patient records (read-only)
- View test results
- Secure messaging
- Task management
- Push notifications

**6.5 Patient Self-Service Kiosk**

In-practice check-in:
- Touch screen interface
- Patient identification (Medicare card scan)
- Update demographics
- Health questionnaires
- Payment processing
- Queue management integration

**6.6 SMS/Email Automation**

Enhanced communication:
- Appointment reminders (24hr, 2hr before)
- Birthday messages
- Recall reminders
- Test results available notification
- Practice announcements
- Campaign management

### Implementation Priority
**PHASE 5** (Advanced Features) or later

### Estimated Effort
- Telehealth: 8-12 weeks
- Patient portal: 12-16 weeks
- Online booking: 6-8 weeks
- Mobile app: 16-20 weeks
- Kiosk: 6-8 weeks
- SMS/email: 4-6 weeks
- **Total**: 52-70 weeks (spread over time)

### Regulatory Impact
- **Competitive advantage**: Market expectation
- **Patient satisfaction**: Improved accessibility
- **Revenue protection**: Reduce no-shows

---

## 7. Specific Pathology Lab Integrations (MEDIUM PRIORITY)

### Current State
✅ Generic pathology integration mentioned  
❌ No specific lab integrations

### Major Australian Labs

**Top 5 Labs (90%+ market share)**:
1. **Australian Clinical Labs (ACL)** - 25% market share
2. **Sonic Healthcare** - 25% market share
3. **Healius (Laverty Pathology)** - 20% market share
4. **QML Pathology** (Queensland) - 15% market share
5. **Douglass Hanly Moir** (NSW) - 10% market share

**Integration Methods**:
- **HL7 v2.x ORU messages** (most common)
- **FHIR DiagnosticReport** (modern)
- **Proprietary web portals** (manual download)
- **Secure messaging networks** (HealthLink, Medical Objects)

**Features Required**:
- Electronic ordering (HL7 ORM messages)
- Results download (HL7 ORU messages)
- PDF report retrieval
- Automatic patient matching
- Result interpretation flags
- Reference ranges
- Trending/graphing

### Implementation Priority
**PHASE 3** (Prescribing & Billing) or **PHASE 4** (Integrations)

### Estimated Effort
- Per lab integration: 2-3 weeks
- Generic HL7 framework: 4 weeks
- **Total**: 14-19 weeks (for 5 labs)

---

## 8. Clinical Decision Support (HIGH PRIORITY)

### Current State
❌ **NOT MENTIONED** in requirements

### Required Implementation

**RACGP Standards Expectation**:

**Features Required**:

1. **Clinical Guidelines Integration**
   - Therapeutic Guidelines (eTG) integration
   - RACGP "Red Book" (preventive care)
   - Chronic disease management guidelines
   - Context-sensitive help

2. **Preventive Care Reminders**
   - Age/gender-based health checks
   - Cervical screening (CST) reminders
   - Bowel screening reminders
   - Cardiovascular risk assessment
   - Mental health screening

3. **Chronic Disease Management**
   - Diabetes management (HbA1c tracking, foot checks)
   - Asthma action plans
   - COPD management
   - Cardiovascular disease
   - Mental health care plans

4. **Health Assessment Prompts**
   - 45-49 Health Assessment
   - 75+ Health Assessment
   - Aboriginal and Torres Strait Islander Health Assessment
   - GP Management Plans (GPMPs)
   - Team Care Arrangements (TCAs)

5. **Quality Assurance Checks**
   - Coding accuracy
   - Billing compliance
   - Clinical note completeness
   - Follow-up tracking
   - Outstanding test results

6. **Risk Calculators**
   - Framingham cardiovascular risk
   - BMI calculator
   - eGFR calculator (kidney function)
   - AUSDRISK (diabetes risk)
   - Fall risk assessment

### Implementation Priority
**PHASE 2** (Clinical Core) - Basic preventive care  
**PHASE 5** (Advanced Features) - Full CDS

### Estimated Effort
- Preventive care engine: 4-6 weeks
- Guidelines integration: 3-4 weeks
- Risk calculators: 2-3 weeks
- Chronic disease modules: 8-10 weeks
- **Total**: 17-23 weeks

### Regulatory Impact
- **RACGP Standards**: Indirect requirement
- **Quality of care**: Improved patient outcomes
- **Medico-legal**: Defensibility in malpractice cases

---

## 9. Document Management System (MEDIUM PRIORITY)

### Current State
✅ Basic document management mentioned  
❌ Missing: OCR, versioning, fax integration

### Required Implementation

**Enhanced Features**:

1. **Scanning and OCR**
   - TWAIN/WIA scanner support
   - Batch scanning
   - OCR for text extraction
   - Searchable PDFs
   - Auto-rotation and deskew

2. **Document Classification**
   - Automatic document type detection
   - Patient matching via OCR
   - Date extraction
   - Barcode recognition

3. **Version Control**
   - Document versions
   - Change tracking
   - Rollback capability
   - Audit trail

4. **Fax Integration**
   - Fax-to-email gateway
   - Incoming fax routing
   - Outgoing fax from system
   - Fax confirmations
   - eFax services (SRFax, iFax)

5. **Templates**
   - Letter templates
   - Medical certificate templates
   - Referral templates
   - Custom forms
   - Mail merge capability

6. **E-Signature**
   - Digital signatures
   - DocuSign integration
   - Audit trail
   - Legal compliance

### Implementation Priority
**PHASE 3** (Prescribing & Billing) - Basic scanning  
**PHASE 5** (Advanced Features) - Full DMS

### Estimated Effort
- Scanning/OCR: 4-5 weeks
- Version control: 2-3 weeks
- Fax integration: 3-4 weeks
- Templates: 3-4 weeks
- E-signature: 2-3 weeks
- **Total**: 14-19 weeks

---

## 10. Conformance and Certification Process (CRITICAL)

### Current State
✅ Conformance testing mentioned  
❌ No process, timeline, or cost details

### Required Implementation

**Certification Roadmap**:

**10.1 Services Australia Conformance**

Required for Medicare Online, AIR, DVA claiming:

**Process**:
1. **Developer Registration**
   - PRODA account creation
   - Organisation registration
   - Staff invitations

2. **Access Test Environment**
   - Request test credentials
   - Set up development environment
   - Test data provisioning

3. **Build and Test**
   - Implement web services
   - Unit testing
   - Integration testing
   - Security testing

4. **Conformance Testing**
   - Submit for testing
   - Services Australia validation
   - Fix issues identified
   - Re-test until pass

5. **Production Registration**
   - Apply for production access
   - Conformance certificate issued
   - Listed on conformance register
   - Go live

**Timeline**: 6-12 months  
**Cost**: Free (testing), ongoing support time

**10.2 My Health Record Conformance**

**Process**:
1. **Register as Developer**
   - Digital Health Agency developer portal
   - Agree to terms and conditions

2. **Choose Integration Method**
   - B2B Gateway (SOAP)
   - FHIR Gateway (REST)
   - CIS to NPP (browser-based)

3. **Access Test Environment**
   - Request NASH certificates
   - Set up test environment

4. **Develop and Test**
   - Implement chosen API
   - Conformance test specification
   - Test scenarios execution

5. **Security Conformance**
   - Penetration testing (external assessor)
   - Vulnerability assessment
   - Security conformance profile checklist

6. **Conformance Assessment**
   - Submit test results
   - ADHA review
   - Conformance certificate issued
   - Listed on register

**Timeline**: 12-18 months  
**Cost**: $5,000-$15,000 (penetration testing)

**10.3 HI Service Conformance**

**Process**: Similar to Services Australia  
**Timeline**: 3-6 months  
**Cost**: Free

**10.4 TGA Medical Device Classification**

Required if software provides diagnostic or therapeutic decision support:

**Classification**:
- **Class I**: Low risk (most practice management software)
- **Class IIa**: Medium-low risk (clinical decision support)
- **Class IIb**: Medium-high risk
- **Class III**: High risk

**Process**:
1. **Self-Assessment**
   - Determine classification
   - Apply medical device rules

2. **Quality Management System**
   - ISO 13485 certification
   - QMS documentation
   - Design controls
   - Risk management

3. **ARTG Registration**
   - Apply to TGA
   - Submit evidence
   - Sponsor appointment
   - Pay fees

**Timeline**: 6-24 months (depending on class)  
**Cost**: $5,000-$50,000+ (ISO 13485 certification, TGA fees)

### Implementation Priority
**PHASE 4** (Integrations) - Conformance testing  
**Ongoing** - Maintain certification

### Estimated Effort
- Conformance preparation: 8-12 weeks
- Testing and fixes: 12-20 weeks
- **Total**: 20-32 weeks (plus wait times)

---

## Summary of Gaps

| # | Gap | Priority | Phase | Effort (weeks) | Regulatory |
|---|-----|----------|-------|----------------|------------|
| 1 | Drug Database (MIMS/AusDI) | CRITICAL | 2 | 6-9 | Standard of care |
| 2 | AIR Integration | CRITICAL | 2 | 5-7 | Mandatory for PIP |
| 3 | Secure Messaging (SMD) | HIGH | 4 | 12-16 | Industry standard |
| 4 | Extended Billing | HIGH | 3 | 11-16 | State-specific laws |
| 5 | FHIR/HL7 Standards | HIGH | 3-4 | 13-17 | Interoperability |
| 6 | Modern Features | MEDIUM | 5 | 52-70 | Competitive |
| 7 | Pathology Labs | MEDIUM | 4 | 14-19 | Workflow efficiency |
| 8 | Clinical Decision Support | HIGH | 2, 5 | 17-23 | RACGP expectation |
| 9 | Document Management | MEDIUM | 3, 5 | 14-19 | Practice workflow |
| 10 | Conformance Process | CRITICAL | 4 | 20-32 | Legally required |

**Total Additional Effort**: 164-228 weeks (3-4+ years of development)

---

## Recommendations

### Immediate Actions (Phase 1)

1. **Update REQUIREMENTS.md** to include all identified gaps
2. **Prioritize critical gaps** (Drug database, AIR, Conformance)
3. **Research MIMS vs AusDI licensing** costs and restrictions
4. **Contact Services Australia** for developer registration process
5. **Budget for conformance testing** ($15,000-$30,000 for external assessments)

### Phase Adjustments

**Current Phase 1 (Foundation)**: OK as-is

**Revised Phase 2 (Clinical Core)**:
- ADD: Drug database integration (MIMS or AusDI)
- ADD: AIR integration
- ADD: Basic clinical decision support (preventive care)

**Revised Phase 3 (Prescribing & Billing)**:
- ADD: WorkCover billing
- ADD: EFTPOS integration
- ADD: PIP/WIP tracking
- ADD: Basic HL7 v2.x support

**Revised Phase 4 (Integrations)**:
- ADD: Secure messaging (HealthLink, Medical Objects)
- ADD: Specific pathology labs (5 major labs)
- ADD: FHIR AU Base implementation
- ADD: Conformance testing and certification

**New Phase 5 (Modern Features)**:
- Telehealth integration
- Patient portal
- Online booking
- Mobile app
- Advanced clinical decision support
- Full document management system

### Long-Term Considerations

1. **Budget**: Estimated **$200,000-$500,000** additional development cost
2. **Timeline**: Add **2-3 years** to original roadmap
3. **Team**: Requires **2-4 additional developers** (specialist skills needed)
4. **Partnerships**: Consider white-labeling drug database, secure messaging
5. **Compliance**: Ongoing maintenance costs for conformance

---

## Conclusion

The current OpenGP requirements document provides a **solid foundation** but is **missing critical features** required for:

1. **Regulatory compliance** (AIR, conformance certification)
2. **Clinical safety** (drug interactions, clinical decision support)
3. **Workflow efficiency** (secure messaging, pathology integration)
4. **Revenue optimization** (WorkCover, incentive programs)
5. **Market competitiveness** (telehealth, patient portal)

**Recommendation**: Update requirements to include all identified gaps, adjust roadmap to 5 phases, and increase budget/timeline estimates accordingly.

---

**Document Status**: DRAFT - Requires review and approval  
**Next Action**: Update REQUIREMENTS.md with critical gaps  
**Owner**: OpenGP Architecture Team

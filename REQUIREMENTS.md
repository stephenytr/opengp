# OpenGP - Open Source General Practice Management Software

> An open-source, terminal-based general practice management system for Australian healthcare providers.

## Table of Contents

1. [Project Overview](#project-overview)
2. [Australian Regulatory Compliance](#australian-regulatory-compliance)
3. [Core Features](#core-features)
4. [Technical Architecture](#technical-architecture)
5. [Database Design](#database-design)
6. [Integration Requirements](#integration-requirements)
7. [Security Requirements](#security-requirements)
8. [UI/UX Design (TUI)](#uiux-design-tui)
9. [Development Roadmap](#development-roadmap)
10. [Open Source Considerations](#open-source-considerations)

---

## Project Overview

**OpenGP** is an open-source general practice management software designed specifically for Australian healthcare providers. Built with Rust and Ratatui for a terminal-based interface, it prioritizes:

- **Privacy & Security**: Full compliance with Australian healthcare regulations
- **Performance**: Rust's memory safety and speed for handling sensitive health data
- **Portability**: Terminal-based UI works across any system with a terminal
- **Open Source**: Community-driven development under a permissive license
- **Database**: PostgreSQL — reliable, concurrent, production-ready

### Target Users

- General Practitioners (GPs)
- Practice Managers
- Medical Receptionists
- Practice Nurses
- Allied Health Professionals working in GP settings

---

## Australian Regulatory Compliance

### Privacy Act 1988 (Cth) - Australian Privacy Principles (APPs)

The Privacy Act applies to all health service providers. Key requirements:

| APP | Requirement | Implementation |
|-----|-------------|----------------|
| APP 1 | Open and transparent management | Privacy policy, clear data handling notices |
| APP 2 | Anonymity and pseudonymity | Allow patients to use pseudonyms where practical |
| APP 3 | Collection of solicited information | Only collect necessary health information |
| APP 5 | Notification of collection | Inform patients about data collection at time of collection |
| APP 6 | Use and disclosure | Use health info only for primary purpose or with consent |
| APP 7 | Direct marketing restrictions | Strict limitations on marketing use of health data |
| APP 8 | Cross-border disclosure | Restrictions on sending data overseas |
| APP 10 | Quality of personal information | Keep records accurate, complete, up-to-date |
| APP 11 | Security of personal information | Protect from misuse, interference, loss, unauthorized access |
| APP 12 | Access to personal information | Patients can request access to their records |
| APP 13 | Correction of personal information | Patients can request corrections |

### My Health Records Act 2012

Requirements for software connecting to My Health Record:

- **Conformance Assessment**: Must pass ADHA conformance testing
- **Security Conformance Profile**: Meet security requirements including penetration testing, vulnerability assessments, secure development practices
- **Clinical Information System (CIS) Requirements**: Specific functional and technical requirements

### Healthcare Identifiers Act 2010

- Must support Healthcare Identifiers (HI):
  - **IHI**: Individual Healthcare Identifier (16 digits for patients)
  - **HPI-I**: Healthcare Provider Identifier - Individual (for practitioners)
  - **HPI-O**: Healthcare Provider Identifier - Organisation (for practices)
- Integration with HI Service for identifier validation

### RACGP Standards for General Practices (5th Edition)

Key standards relevant to software:

- **Core Standard 6**: Information security
  - Criterion C6.1: Health and other information management
  - Criterion C6.2: Content of patient records
  - Criterion C6.3: Privacy and confidentiality
  - Criterion C6.4: Information security

### Data Sovereignty Requirements

- **Primary Requirement**: All patient health data MUST be physically stored within Australian territory
- **Cloud Providers**: Must use Australian-based data centres (Sydney, Melbourne, Canberra regions)
- **Cross-border transfers**: Require explicit patient consent and adequate protections under Privacy Act
- **Backup locations**: All backups must remain within Australia

### Information Security Manual (ISM) Compliance

**Mandatory for Australian Government and Healthcare**:

The Australian Signals Directorate (ASD) **Information Security Manual** sets baseline security requirements. OpenGP implements:

- **Essential Eight Maturity Level 2**: All 8 mitigation strategies (detailed in Security Requirements section)
- **System monitoring**: Comprehensive event logging per ISM guidelines
- **Incident response**: Documented procedures for security incidents
- **Personnel security**: Background checks for staff with system access (practice responsibility)

### Compliance Penalties and Legal Obligations

**Privacy Act 1988 Penalties**:
- **Serious or repeated breaches**: Civil penalties up to **$2.1 million** for organizations
- **Notifiable Data Breaches (NDB) Scheme**: Must notify OAIC and affected individuals within 30 days
- **Criminal penalties**: Up to 2 years imprisonment for improper disclosure

**Healthcare Identifiers Act 2010 Penalties**:
- **Unauthorized collection/use**: Criminal penalties up to 2 years imprisonment and/or 120 penalty units
- **Loss of access**: Revocation of HI Service access

**My Health Record Penalties**:
- **Non-conformance**: Disconnection from My Health Record system
- **Improper access**: Criminal penalties under My Health Records Act 2012

**State/Territory Penalties**:
- Additional penalties may apply under state health records legislation
- Professional disciplinary action through AHPRA for healthcare providers

---

## Core Features

### 1. Patient Management

- Personal details (name, DOB, gender, address)
- Medicare number and healthcare identifiers (IHI)
- Contact information and emergency contacts
- Concession card details (DVA, Pensioner, Healthcare Card)
- Patient search, registration, and duplicate detection

### 2. Appointment Scheduling

- Daily/weekly/monthly calendar views
- Multi-practitioner scheduling
- Appointment types with configurable durations
- Waitlist management
- SMS/email reminders (integration)

### 3. Clinical Records

- SOAP Notes (Subjective, Objective, Assessment, Plan)
- Medical/surgical/family/social history
- Allergies and adverse reactions
- Current medications
- Immunization records
- Vital signs tracking
- Clinical templates

### 4. Prescriptions & Medications

#### Core Prescription Features
- Electronic prescriptions (e-prescribing with token-based delivery)
- Paper prescriptions (legacy support)
- PBS/RPBS integration with real-time pricing
- Authority prescriptions (complex and streamlined authority)
- Repeat prescriptions management
- 60-day prescription support
- Prescription history and tracking

#### Drug Database Integration (CRITICAL)
- **MIMS (Monthly Index of Medical Specialties)** or **AusDI** integration
- Real-time drug interaction checking
  - Severity levels: minor, moderate, severe, contraindicated
  - Clinical guidance on managing interactions
  - Check against all current medications
- Allergy cross-checking and cross-sensitivity warnings
- Dosage calculations:
  - Weight-based dosing (paediatric and adult)
  - Age-appropriate dosing recommendations
  - Renal/hepatic adjustment guidance
- Drug information database:
  - Generic and brand names
  - Indications and contraindications
  - Side effects and warnings
  - PBS/RPBS status and pricing
  - Pregnancy/breastfeeding categories (A, B1, B2, B3, C, D, X)
- Medication reconciliation tools

### 5. Pathology & Imaging

#### Electronic Ordering
- HL7 ORM (Order Message) generation
- FHIR ServiceRequest creation
- Common test panels and favorites
- Order tracking and status updates
- Integration with major Australian labs:
  - Australian Clinical Labs (ACL)
  - Sonic Healthcare
  - Healius (Laverty Pathology)
  - QML Pathology (Queensland)
  - Douglass Hanly Moir (NSW)

#### Results Management
- Electronic results delivery (HL7 ORU messages)
- FHIR DiagnosticReport parsing
- Automatic patient matching
- Result acknowledgment workflow
- Abnormal result flagging with alerts
- Critical result notifications
- Result trending/graphing over time
- PDF report retrieval and storage
- Reference range display

#### Imaging Integration
- Radiology ordering (X-ray, CT, MRI, ultrasound)
- PACS (Picture Archiving and Communication System) integration
- Image viewing (DICOM viewer or external)
- Report viewing and filing

### 6. Billing & Medicare

#### Medicare Claiming
- **Medicare Online** real-time claiming via PRODA
- **ECLIPSE** (Electronic Claim Lodgement and Information Processing)
- Bulk billing with same-day processing
- Private billing (patient invoice + Medicare gap claim)
- Patient eligibility verification
- MBS (Medicare Benefits Schedule) item search and selection
- Time-based item validation
- Multiple item claiming
- Same-day claim deletion

#### DVA (Department of Veterans' Affairs)
- DVA medical claiming
- DVA pathology claiming
- Gold card, White card, Orange card support
- DVA-specific item codes

#### WorkCover (Workers' Compensation)
- State-specific WorkCover billing:
  - NSW (icare)
  - VIC (WorkSafe Victoria)
  - QLD (WorkCover Queensland)
  - SA (ReturnToWorkSA)
  - WA (WorkCover WA)
  - TAS (WorkCover Tasmania)
- Certificate of capacity generation
- Employer and insurer details
- Injury tracking and case management

#### Third-Party Billing
- Private health insurance claiming
- Compensation cases (motor vehicle accidents, public liability)
- Self-insured organizations
- Invoice generation and tracking
- Payment plans and debt management

#### Payment Processing
- **EFTPOS integration** (Tyro, Smartpay, Westpac terminals)
- Cash payments
- Credit/debit card processing
- Split payments (Medicare + patient gap)
- Receipt generation and reprinting
- End-of-day reconciliation
- Refund processing

#### Medicare Incentive Programs
- **Practice Incentive Program (PIP)**:
  - Quality Improvement (QI) incentive
  - eHealth incentive
  - Diabetes SIP
  - Asthma SIP
  - After Hours incentive
  - Eligibility tracking and reporting
- **Workforce Incentive Program (WIP)**:
  - Rural and remote practice support
  - Tier calculations (Tier 1, 2, 3)
  - Quarterly reporting to Services Australia

#### Financial Management
- Accounts receivable tracking
- Outstanding accounts management
- Payment plan administration
- Debt collection workflow
- Financial reporting (profit/loss, cash flow)
- Fee schedule management
- Practice benchmarking against industry averages

### 7. Referrals & Secure Messaging

#### Referral Management
- Specialist referrals (12-month validity)
- Allied health referrals
- Hospital referrals
- Referral templates and auto-population
- Referral tracking (sent, received, appointment made)
- Incoming referral processing and triage
- Specialist database with contact details

#### Secure Message Delivery (SMD)
Integration with Australian secure messaging networks:

- **HealthLink** - Market leader (60%+ market share)
  - Electronic referral delivery
  - Pathology/imaging results
  - Specialist letters and discharge summaries
  - Delivery confirmation and read receipts
  
- **Medical Objects** - Second largest provider
  - Same capabilities as HealthLink
  - Redundancy option for practices
  
- **Argus Connect** - Growing market share
  - Secure clinical messaging
  - Document exchange
  
- **Secure eReferral Network (SeNT)** - Government-funded
  - Free for GP to specialist referrals
  - NSW, VIC, QLD coverage

**Technical Standards**:
- SOAP-based web services (legacy)
- FHIR messaging (modern)
- CDA R2 document format support
- End-to-end encryption
- Digital signatures for authenticity
- Audit trail for all messages

### 8. Immunisation Management (AIR Integration)

#### Australian Immunisation Register (AIR) Integration (MANDATORY)

**Core Features**:
- **Real-time AIR reporting** within 24 hours of vaccination
- **AIR history retrieval** for all patients
- **Vaccination recording**:
  - Vaccine type and batch number
  - Dose number in series
  - Anatomical site
  - Provider details
  - Adverse event reporting
  
**National Immunisation Program (NIP)**:
- Birth to 4 years schedule
- School-age vaccines (Year 7, Year 10)
- Adult schedule (influenza, COVID-19, pneumococcal)
- Catch-up schedule calculator
- Overdue vaccination alerts

**Reporting and Compliance**:
- Practice immunisation coverage statistics
- AIR notification status tracking
- Medicare incentive payment tracking (PIP immunisation)
- Public health reporting

**Technical Requirements**:
- SOAP-based web services via Medicare Online
- PRODA authentication required
- Error handling for duplicate notifications
- Batch upload for historical records

### 9. Recalls & Reminders

#### Preventive Health Recalls
- Age/gender-based health checks
- Cervical screening (National Cervical Screening Program)
- Bowel screening (National Bowel Cancer Screening Program)
- Cardiovascular risk assessment (45+ Health Assessment)
- Diabetes risk screening (AUSDRISK)
- Falls risk assessment (75+)
- Mental health screening (K10, EPDS)

#### Chronic Disease Management Recalls
- Diabetes: HbA1c, lipids, kidney function, eye checks, foot checks
- Asthma: Spirometry, action plan review
- COPD: Spirometry, exacerbation tracking
- Cardiovascular disease: BP, lipids, ECG
- Mental health: Care plan review, outcome measures

#### Health Assessment Reminders
- 45-49 Health Assessment
- 75+ Health Assessment
- Aboriginal and Torres Strait Islander Health Assessment
- GP Management Plans (GPMP) review
- Team Care Arrangements (TCA) review

#### Immunisation Recalls
- Childhood vaccination schedule
- Adult influenza (65+, chronic disease)
- COVID-19 boosters
- Pneumococcal (65+)
- Shingles (70+)

#### Recall Engine Features
- Customizable recall criteria
- Bulk recall generation
- Recall due date calculations
- Recall completion tracking
- Patient exclusion management
- Bulk SMS/email communication

### 10. Clinical Decision Support

#### Preventive Care Engine
- Age/gender-based health check reminders
- Cervical screening tracking (25-74 years, 5-yearly)
- Bowel cancer screening (50-74 years, 2-yearly)
- Cardiovascular risk assessment tools
- Mental health screening prompts (K10, EPDS)
- Falls risk assessment (65+)

#### Chronic Disease Management
- **Diabetes Management**:
  - HbA1c tracking and targets
  - Annual cycle of care reminders
  - Foot examination tracking
  - Eye examination reminders
  - Kidney function monitoring (eGFR, ACR)
- **Asthma Management**:
  - Asthma action plan templates
  - Spirometry tracking
  - Preventer/reliever usage
- **COPD Management**:
  - Spirometry results tracking
  - Exacerbation frequency
  - Vaccination status
- **Cardiovascular Disease**:
  - BP monitoring
  - Lipid management
  - Medication adherence

#### Clinical Guidelines Integration
- Therapeutic Guidelines (eTG) integration
- RACGP "Red Book" (Guidelines for preventive activities)
- Context-sensitive clinical guidance
- Evidence-based treatment protocols

#### Risk Calculators
- Framingham cardiovascular risk score
- Australian CVD risk calculator
- BMI calculator and interpretation
- eGFR calculator (kidney function)
- AUSDRISK (Type 2 diabetes risk)
- QRISK cardiovascular calculator
- Falls risk assessment tools

#### Health Assessment Templates
- 45-49 Health Assessment (MBS 701)
- 75+ Health Assessment (MBS 701)
- Aboriginal and Torres Strait Islander Health Assessment (MBS 715)
- GP Management Plans (GPMP - MBS 721)
- Team Care Arrangements (TCA - MBS 723)

#### Quality Assurance
- Coding accuracy checking
- Billing compliance validation
- Clinical note completeness checking
- Outstanding test results alerts
- Follow-up tracking
- Medication review reminders

### 11. Reporting & Analytics

- Clinical audit reports
- Practice activity reports (consultations, procedures, revenue)
- Financial reports (P&L, cash flow, AR aging)
- Accreditation reports (RACGP, AGPAL)
- Quality improvement metrics
- Patient demographic statistics
- Chronic disease registers
- Immunisation coverage reports
- Preventive health activity reports
- Custom report builder (future)

### 12. Multi-Practitioner Support

- User roles and permissions (Admin, Doctor, Nurse, Receptionist, Billing)
- Individual practitioner schedules and availability
- Shared patient records with controlled access
- Billing split and revenue distribution
- User-specific preferences and templates
- Practitioner performance analytics
- Locum/temporary practitioner support

### 13. Document Management

#### Core Document Features
- Secure storage and retrieval of clinical documents
- Document categorization and tagging
- Quick document search
- Document preview
- Version control and audit trail
- Document templates with auto-population

#### Scanning and OCR
- Scanner integration (TWAIN/WIA protocols)
- Batch scanning workflow
- Optical Character Recognition (OCR)
- Searchable PDF generation
- Auto-rotation and deskew
- Barcode recognition for patient matching

#### Fax Integration
- Fax-to-email gateway integration
- Incoming fax routing to patient records
- Outgoing fax from system
- Fax delivery confirmation
- eFax services (SRFax, iFax, RingCentral Fax)

#### Document Types
- Pathology/imaging reports (if not electronically received)
- Specialist letters
- Hospital discharge summaries
- Legal documents
- Patient correspondence
- Consent forms
- Advance care directives

### 14. Modern Digital Features

#### Telehealth (Video Consultations)
- Video consultation platform integration
- Waiting room functionality
- Screen sharing capability
- Session recording (with patient consent)
- Billing integration (MBS telehealth item numbers)
- Platform options: Zoom, MS Teams, or dedicated healthcare platform
- Privacy Act compliance for video storage

#### Online Patient Portal
- Patient self-service account creation
- View medical history (with practitioner approval)
- Book appointments online
- Request prescription renewals
- View test results (when released by doctor)
- Secure messaging with practice
- Document upload (health summaries from other providers)
- Update demographics and contact details

#### Online Booking System
- 24/7 appointment availability
- Real-time calendar synchronization
- Appointment type selection (standard, long, urgent)
- Practitioner preference selection
- New patient intake forms
- SMS/email confirmation
- Automated reminders
- Payment deposit for no-show prevention

#### SMS/Email Automation
- Appointment reminders:
  - 24-hour advance reminder
  - 2-hour advance reminder
- Recall reminders (preventive care, chronic disease)
- Test results available notifications
- Birthday messages
- Practice announcements
- Vaccination due reminders
- Campaign management for health initiatives

#### Patient Self-Service Kiosk (In-Practice)
- Touch screen interface
- Patient check-in
- Medicare card scanning
- Update demographics
- Health questionnaires (pre-consultation)
- Payment processing
- Queue management integration
- Accessibility features

#### Mobile App (Practitioner Access)
- View daily schedule
- Access patient records (read-only for security)
- View test results
- Secure messaging
- Task management and reminders
- Push notifications for critical results
- Offline mode with sync

---

## Integration Requirements

### Australian Government Services

#### PRODA (Provider Digital Access)

**Purpose**: Authentication gateway for all Services Australia services

**Requirements**:
- Individual PRODA account creation (identity verification)
- Organisation registration in Health Systems Developer Portal
- Staff invitations and delegation management
- Multi-factor authentication
- Certificate-based access for development environments
- Service-specific linking (Medicare, PBS, HI Service, AIR)

**Process**:
1. Create individual PRODA account (myGov identity verification)
2. Register organization
3. Request developer access
4. Obtain test environment credentials
5. Complete conformance testing
6. Apply for production access

#### Medicare Online Services

**Integration Points**:
- **Patient Claiming**: Real-time and store-and-forward
- **Bulk Billing**: Immediate claim submission
- **Patient Verification**: Eligibility checking
- **DVA Claims**: Medical and pathology
- **Claim Status**: Track outcomes and payments

**Technical**:
- SOAP-based web services
- PRODA OAuth 2.0 authentication
- Conformance testing required
- Production certification needed

#### PBS (Pharmaceutical Benefits Scheme) API

**Integration Points**:
- **PBS Schedule Data API**: Monthly medication updates
- **PBS Embargo API**: Advance access to schedule changes (May 2026+)
- **Authority Approvals**: Submit complex authority requests
- **Real-Time Pricing**: Current PBS/RPBS pricing
- **Eligibility Checking**: Patient PBS entitlement

**Technical**:
- RESTful API (JSON and XML support)
- Rate limits: 1 request per 20 seconds (public) or higher for registered
- Monthly schedule updates (1st of each month)
- API authentication required for high-volume access

#### Healthcare Identifiers (HI) Service

**Purpose**: Validate and manage healthcare identifiers

**Identifier Types**:
- **IHI**: Individual Healthcare Identifier (patients)
- **HPI-I**: Healthcare Provider Identifier - Individual (practitioners)
- **HPI-O**: Healthcare Provider Identifier - Organisation (practices)

**Integration Features**:
- Search and verify patient IHI
- Provider verification
- Bulk IHI uploads
- Registration status checking
- Demographic verification

**Technical**:
- SOAP-based B2B Gateway
- Certificate-based authentication
- Integration toolkit (.NET and Java available)
- Conformance testing required

#### My Health Record

**Integration Methods** (choose one):
- **FHIR Gateway v4.0.0**: Modern RESTful API (RECOMMENDED)
- **B2B Gateway**: Legacy SOAP-based API
- **CIS to NPP**: Browser-based (minimal development)

**Document Types Supported**:
- Event summaries
- Specialist letters
- Discharge summaries
- Pathology reports
- Diagnostic imaging reports
- Prescription and dispense records
- Medicare DVA-funded services

**Requirements**:
- NASH certificate for authentication
- Conformance testing mandatory
- Security Conformance Profile compliance
- Penetration testing by approved assessor

#### Australian Immunisation Register (AIR)

**Integration Points**:
- Record vaccinations (encounter and historical)
- Retrieve immunisation history
- Vaccination due/overdue notifications
- Batch uploads for historical data

**Technical**:
- SOAP web services via Medicare Online
- PRODA authentication
- Real-time notifications within 24 hours
- Error handling for duplicates

### International Standards

#### HL7 v2.x Messaging

**Message Types**:
- **ORU^R01**: Unsolicited observation results (pathology/imaging)
- **ORM^O01**: Order message (pathology/imaging orders)
- **ADT**: Admission/discharge/transfer messages
- **MDM**: Medical document management

**Implementation**:
- HL7 v2.x parser and validator
- Segment parsing (MSH, PID, OBR, OBX, etc.)
- Error handling and acknowledgments (ACK/NACK)
- Message routing and filing

#### FHIR (Fast Healthcare Interoperability Resources)

**Australian Profiles**:
- **FHIR AU Base v4.2.0+**: Australian base profiles
  - AU Patient
  - AU Practitioner
  - AU Organization
  - AU Medication
  - AU Immunisation
- **AU Core**: Core data elements for exchange
- **AU eRequesting**: Pathology/imaging ordering

**Use Cases**:
- My Health Record FHIR Gateway
- Pathology results (DiagnosticReport)
- Medication management (MedicationRequest, MedicationStatement)
- Immunisation records (Immunization)
- Care planning (CarePlan)

**Terminology**:
- SNOMED-CT AU (clinical terminology)
- AMT (Australian Medicines Terminology)
- LOINC (lab test codes)
- ICD-10-AM (diagnosis coding)

#### CDA (Clinical Document Architecture)

**Document Types**:
- Event Summary (CDA R2)
- Discharge Summary
- Referral documents
- Specialist reports

**Requirements**:
- CDA R2 parser
- Stylesheet-based rendering
- Narrative text extraction
- Structured data extraction
- Document validation

### Third-Party Integrations

#### Secure Messaging Networks
- HealthLink
- Medical Objects
- Argus Connect
- SeNT (Secure eReferral Network)

#### Pathology Laboratories
- Australian Clinical Labs (ACL)
- Sonic Healthcare
- Healius (Laverty Pathology)
- QML Pathology
- Douglass Hanly Moir

#### Drug Databases
- MIMS (Monthly Index of Medical Specialties) - Subscription required
- AusDI (Australian Drug Information) - Open data option

---

## Technical Architecture

### Technology Stack

```
┌─────────────────────────────────────────────────────────┐
│                    OpenGP Application                    │
├─────────────────────────────────────────────────────────┤
│  UI Layer (Ratatui + Crossterm)                         │
├─────────────────────────────────────────────────────────┤
│  Application Layer (Component Architecture, Tokio)       │
├─────────────────────────────────────────────────────────┤
│  Domain Layer (Patient, Appointment, Clinical, Billing)  │
├─────────────────────────────────────────────────────────┤
│  Data Layer (SQLx - PostgreSQL)                          │
├─────────────────────────────────────────────────────────┤
│  Infrastructure (Encryption, Audit, Auth, API Clients)   │
└─────────────────────────────────────────────────────────┘
```

### Directory Structure

```
opengp/
├── Cargo.toml
├── README.md
├── REQUIREMENTS.md
├── ARCHITECTURE.md
├── GAP_ANALYSIS.md
├── LICENSE
├── .env.example
├── config/
│   ├── default.toml
│   ├── development.toml
│   └── production.toml
├── migrations/
│   ├── 001_initial_schema.sql
│   ├── 002_audit_tables.sql
│   └── ...
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── app.rs
│   ├── config.rs
│   ├── error.rs
│   │
│   ├── ui/                      # UI Layer
│   │   ├── mod.rs
│   │   ├── tui.rs              # Terminal setup
│   │   ├── event.rs            # Event handling
│   │   ├── theme.rs            # Styling
│   │   └── widgets/            # Custom widgets
│   │       ├── mod.rs
│   │       ├── patient_table.rs
│   │       ├── calendar.rs
│   │       └── search_modal.rs
│   │
│   ├── components/             # UI Components
│   │   ├── mod.rs
│   │   ├── patient/
│   │   │   ├── mod.rs
│   │   │   ├── list.rs
│   │   │   ├── detail.rs
│   │   │   └── form.rs
│   │   ├── appointment/
│   │   ├── clinical/
│   │   ├── prescription/
│   │   ├── billing/
│   │   └── immunisation/
│   │
│   ├── domain/                 # Domain Layer
│   │   ├── mod.rs
│   │   ├── patient/
│   │   │   ├── mod.rs
│   │   │   ├── model.rs
│   │   │   ├── service.rs
│   │   │   └── repository.rs
│   │   ├── appointment/
│   │   ├── clinical/
│   │   ├── prescription/
│   │   ├── billing/
│   │   ├── immunisation/
│   │   ├── referral/
│   │   └── user/
│   │
│   ├── infrastructure/         # Infrastructure Layer
│   │   ├── mod.rs
│   │   ├── database/
│   │   │   ├── mod.rs
│   │   │   ├── pool.rs
│   │   │   ├── migrations.rs
│   │   │   └── repositories/
│   │   ├── crypto/
│   │   │   ├── mod.rs
│   │   │   ├── encryption.rs
│   │   │   └── hashing.rs
│   │   ├── audit/
│   │   │   ├── mod.rs
│   │   │   └── logger.rs
│   │   └── auth/
│   │       ├── mod.rs
│   │       ├── session.rs
│   │       └── rbac.rs
│   │
│   └── integrations/           # External Integrations
│       ├── mod.rs
│       ├── medicare/           # Medicare Online
│       │   ├── mod.rs
│       │   ├── client.rs
│       │   └── models.rs
│       ├── pbs/                # PBS API
│       ├── air/                # Immunisation Register
│       ├── hi_service/         # Healthcare Identifiers
│       ├── mhr/                # My Health Record (FHIR)
│       ├── proda/              # PRODA Authentication
│       ├── secure_messaging/   # HealthLink, Medical Objects
│       ├── pathology/          # Lab integrations
│       │   ├── acl.rs
│       │   ├── sonic.rs
│       │   └── healius.rs
│       ├── drug_database/      # MIMS or AusDI
│       ├── hl7/                # HL7 v2.x parser
│       └── fhir/               # FHIR client and parsers
│
├── tests/
│   ├── unit/
│   ├── integration/
│   └── fixtures/
│
└── docs/
    ├── conformance/            # Conformance test results
    ├── security/               # Security documentation
    └── user-guide/             # User documentation
```

---

## Database Design

### Design Principles

1. **Audit Trail**: All clinical data includes audit columns
2. **Soft Deletes**: Never hard delete clinical data
3. **Encryption**: Sensitive fields encrypted at application level
4. **UUID primary keys**: All tables use UUID PKs via `gen_random_uuid()`

### Core Tables

- **users**: Authentication and authorization
- **practitioners**: Doctors, nurses with HPI-I
- **patients**: Demographics, Medicare, IHI
- **appointments**: Scheduling with status tracking
- **consultations**: Clinical encounters with SOAP notes
- **prescriptions**: Medications with PBS support
- **patient_allergies**: Allergen tracking
- **audit_log**: Append-only audit trail

### Database Stack

- **Engine**: PostgreSQL 14+
- **Driver**: SQLx with compile-time query validation
- **Migrations**: sqlx-cli (`sqlx migrate run`)
- **Connection**: Pool via `DATABASE_URL` env var

---

## Security Requirements

### Australian Regulatory Compliance

OpenGP must comply with the **Information Security Manual (ISM)** published by the Australian Signals Directorate (ASD).

#### Essential Eight Mitigation Strategies (MANDATORY)

All 8 strategies must be implemented at Maturity Level 2 minimum:

1. **Application Control**: Whitelist approved applications, block unapproved executables
2. **Patch Applications**: Security updates applied within 48 hours for critical vulnerabilities
3. **Configure Microsoft Office Macro Settings**: Block macros from internet, only allow signed macros
4. **User Application Hardening**: Disable Flash, ads, Java in web browsers
5. **Restrict Administrative Privileges**: Least privilege access, separate admin accounts
6. **Patch Operating Systems**: Security updates applied within 48 hours for critical vulnerabilities  
7. **Multi-Factor Authentication (MFA)**: Required for all privileged access, recommended for all users
8. **Daily Backups**: Automated daily backups with 7-year retention, tested quarterly

**Non-compliance penalties**: Up to **$2.1 million** for serious or repeated privacy breaches under Privacy Act.

### Authentication

- **Password Requirements**:
  - Minimum 12 characters
  - Complexity: uppercase, lowercase, numbers, special characters
  - Password history: prevent reuse of last 10 passwords
  - Password expiry: 90 days (configurable)
- **Account lockout**: 5 failed attempts, 15-minute lockout
- **Session timeout**: 15 minutes inactivity (configurable)
- **Multi-Factor Authentication (MFA)**: TOTP-based (Google Authenticator compatible)

### Authorization (RBAC)

- **Roles**: Admin, Doctor, Nurse, Receptionist, Billing
- **Field-level permissions**: Granular access control
- **Break-the-glass emergency access**: With mandatory audit logging
- **Principle of least privilege**: Users granted minimum necessary permissions

### Encryption

#### Data at Rest
- **Algorithm**: AES-256-GCM (Galois/Counter Mode)
- **Key Management**:
  - Master encryption key stored in environment variable or KMS
  - Data encryption keys (DEKs) derived from master key
  - Key rotation: Annually or on suspected compromise
  - Keys never stored in database or version control
- **Scope**: 
  - Clinical notes (SOAP notes, confidential notes)
  - Prescription details
  - Social history
  - Patient financial information
  - Any PII marked as sensitive
- **Database Encryption**: 
  - Column-level encryption at application layer (AES-256-GCM before insert)
  - PostgreSQL transparent data encryption at filesystem level (production)

#### Data in Transit
- **Protocol**: TLS 1.3 (minimum TLS 1.2 for legacy compatibility)
- **Certificate Management**: 
  - Valid certificates from trusted CA
  - Certificate rotation every 12 months
  - Certificate pinning for critical integrations
- **End-to-End Encryption**: For patient communication features (future)

#### Key Management Best Practices
- **Separation of duties**: Different keys for different data types
- **Secure deletion**: Keys securely wiped on rotation
- **Backup encryption**: Backup files encrypted with separate keys
- **HSM consideration**: Hardware Security Module for production (future)

### Audit Logging

#### Comprehensive Audit Trail (MANDATORY)

All patient data access MUST be logged per Privacy Act APP 11:

**Events to Log**:
- User authentication (login, logout, failed attempts, MFA events)
- Patient record access (view, create, update, delete, search, export)
- Clinical data operations (consultations, prescriptions, test results)
- Configuration changes (user management, system settings)
- Data exports and report generation
- Break-the-glass access events
- Failed authorization attempts

**Audit Log Format**:
```json
{
  "timestamp": "ISO8601 with timezone",
  "user_id": "UUID",
  "action": "enum (LOGIN, PATIENT_READ, etc.)",
  "entity_type": "Patient|Consultation|Prescription",
  "entity_id": "UUID",
  "ip_address": "IPv4/IPv6",
  "user_agent": "string",
  "session_id": "UUID",
  "result": "SUCCESS|FAILURE",
  "metadata": { "additional_context": "..." }
}
```

**Audit Log Protection**:
- **Immutability**: Append-only, no deletion or modification allowed
- **Integrity**: Digital signatures or cryptographic hashing (SHA-256)
- **Tamper detection**: Hash chain linking logs together
- **Separate storage**: Audit logs stored separately from application data
- **Retention**: Minimum **7 years** (aligns with medical record retention)
- **Backup**: Daily backups with same retention as operational data
- **Access control**: Read-only access for auditors, restricted for admins

**Compliance Monitoring**:
- Weekly audit log review for suspicious activity
- Quarterly access pattern analysis
- Annual compliance audit by external auditor
- Real-time alerts for break-the-glass access

---

## UI/UX Design (TUI)

### Design Principles

1. **Efficiency**: Minimize keystrokes for common tasks
2. **Discoverability**: Clear key bindings shown on screen
3. **Consistency**: Same patterns across all screens
4. **Accessibility**: High contrast support

### Key Bindings

| Key | Action |
|-----|--------|
| F1-F12 | Main module navigation |
| Tab | Next field/pane |
| Enter | Select/Confirm |
| Esc | Cancel/Back |
| / | Search |
| ? | Help |

---

## Development Roadmap

**Revised Timeline**: 5 phases over 3-4 years

### Phase 1: Foundation (MVP)
**Duration**: 3-4 months  
**Team**: 1-2 developers

**Core Deliverables**:
- [ ] Project architecture and module structure
- [ ] Database schema design (PostgreSQL)
- [ ] Migration framework (sqlx-cli)
- [ ] Authentication system (password-based, session management)
- [ ] User management (CRUD, RBAC)
- [ ] Basic patient management (CRUD)
- [ ] Basic TUI framework (component architecture, event handling)
- [ ] Patient list and detail screens
- [ ] Encryption service (AES-256-GCM)
- [ ] Audit logging foundation
- [ ] Configuration management
- [ ] Error handling framework
- [ ] Unit and integration test framework

**Deliverable**: Working prototype with patient management

### Phase 2: Clinical Core
**Duration**: 4-6 months  
**Team**: 2-3 developers

**Core Deliverables**:
- [ ] Appointment scheduling system
  - [ ] Daily/weekly/monthly calendar views
  - [ ] Multi-practitioner support
  - [ ] Appointment types and durations
  - [ ] Waitlist management
- [ ] Consultation/clinical notes (SOAP format)
- [ ] Medical/surgical/family/social history
- [ ] Allergy and adverse reaction management
- [ ] Current medications list
- [ ] Vital signs recording
- [ ] Clinical templates and quick text

**NEW - Critical Additions**:
- [ ] **Drug Database Integration** (MIMS or AusDI)
  - [ ] Drug interaction checking
  - [ ] Allergy cross-checking
  - [ ] Dosage calculation support
  - [ ] Drug information lookup
- [ ] **AIR (Australian Immunisation Register) Integration**
  - [ ] Vaccination recording
  - [ ] AIR history retrieval
  - [ ] NIP schedule engine
  - [ ] Overdue vaccination alerts
- [ ] **Basic Clinical Decision Support**
  - [ ] Preventive care reminders
  - [ ] Health assessment prompts
  - [ ] Basic risk calculators (BMI, eGFR)

**Deliverable**: Complete clinical workflow for consultations

### Phase 3: Prescribing & Billing
**Duration**: 4-6 months  
**Team**: 2-3 developers

**Core Deliverables**:
- [ ] Prescription management
  - [ ] Electronic prescriptions (e-prescribing)
  - [ ] Paper prescriptions
  - [ ] Repeat prescriptions
  - [ ] PBS authority prescriptions
  - [ ] Drug database integration for safety checks
- [ ] Medicare claiming
  - [ ] Medicare Online integration (PRODA setup)
  - [ ] Bulk billing workflow
  - [ ] Private billing with gap claims
  - [ ] MBS item selection and validation
- [ ] Invoice generation and tracking
- [ ] Basic financial reporting

**NEW - Critical Additions**:
- [ ] **Extended Billing Support**
  - [ ] WorkCover billing (state-specific)
  - [ ] DVA extended claiming
  - [ ] Third-party billing
  - [ ] EFTPOS integration (Tyro/Smartpay)
  - [ ] Payment plans and debt tracking
- [ ] **Medicare Incentive Programs**
  - [ ] PIP (Practice Incentive Program) tracking
  - [ ] WIP (Workforce Incentive Program) tracking
  - [ ] Automated incentive claims
- [ ] **Basic HL7 v2.x Support**
  - [ ] ORU^R01 parser (pathology results)
  - [ ] Message routing and patient matching

**Deliverable**: Complete billing and prescribing workflows

### Phase 4: Integrations & Conformance
**Duration**: 6-12 months  
**Team**: 3-4 developers (including integration specialist)

**Core Deliverables**:
- [ ] **Services Australia Conformance**
  - [ ] Medicare Online conformance testing
  - [ ] AIR conformance testing
  - [ ] Production certification
  - [ ] Listed on conformance register
  
- [ ] **HI Service Integration**
  - [ ] IHI lookup and verification
  - [ ] Provider verification (HPI-I, HPI-O)
  - [ ] Bulk IHI operations
  - [ ] Conformance certification

- [ ] **My Health Record Integration**
  - [ ] FHIR Gateway v4.0+ implementation
  - [ ] Document upload (event summaries, specialist letters)
  - [ ] Document retrieval
  - [ ] Security conformance testing
  - [ ] Penetration testing by approved assessor
  - [ ] Conformance certification

**NEW - Additional Integrations**:
- [ ] **Secure Messaging (SMD)**
  - [ ] HealthLink integration
  - [ ] Medical Objects integration
  - [ ] SeNT (Secure eReferral Network)
  - [ ] Electronic referral delivery
  - [ ] Results/letters receipt
  
- [ ] **Pathology Laboratory Integration**
  - [ ] Australian Clinical Labs (ACL)
  - [ ] Sonic Healthcare
  - [ ] Healius/Laverty
  - [ ] QML Pathology
  - [ ] Douglass Hanly Moir
  - [ ] HL7 ORU message processing
  - [ ] FHIR DiagnosticReport support

- [ ] **FHIR AU Implementation**
  - [ ] FHIR AU Base profiles
  - [ ] AU Core implementation
  - [ ] SNOMED-CT AU terminology
  - [ ] AMT (Australian Medicines Terminology)
  
- [ ] **CDA Document Support**
  - [ ] CDA R2 parser
  - [ ] Stylesheet rendering
  - [ ] Structured data extraction

**Budget**: $15,000-$30,000 for conformance testing and penetration testing

**Deliverable**: Fully certified, production-ready system

### Phase 5: Advanced Features & Modern Capabilities
**Duration**: 12-18 months (ongoing)  
**Team**: 3-5 developers

**Core Deliverables**:
- [ ] **Advanced Clinical Decision Support**
  - [ ] Therapeutic Guidelines (eTG) integration
  - [ ] RACGP Red Book integration
  - [ ] Chronic disease management modules
  - [ ] Advanced risk calculators
  - [ ] Clinical pathway automation
  
- [ ] **Telehealth Platform**
  - [ ] Video consultation integration
  - [ ] Waiting room functionality
  - [ ] Session recording (with consent)
  - [ ] MBS telehealth billing integration
  - [ ] Privacy-compliant video storage

- [ ] **Patient Portal**
  - [ ] Patient account creation
  - [ ] Online appointment booking
  - [ ] View medical history
  - [ ] Prescription requests
  - [ ] View test results
  - [ ] Secure messaging with practice
  - [ ] Document upload

- [ ] **Mobile Application** (Practitioner)
  - [ ] iOS and Android apps
  - [ ] Schedule viewing
  - [ ] Patient record access (read-only)
  - [ ] Secure messaging
  - [ ] Push notifications
  - [ ] Offline mode with sync

- [ ] **Practice Automation**
  - [ ] SMS/email appointment reminders
  - [ ] Automated recall campaigns
  - [ ] Birthday messages
  - [ ] Patient self-service kiosk
  - [ ] Queue management
  
- [ ] **Enhanced Document Management**
  - [ ] Scanning and OCR
  - [ ] Fax-to-email integration
  - [ ] Version control
  - [ ] E-signature support
  - [ ] Advanced document search

- [ ] **Advanced Reporting & Analytics**
  - [ ] Practice dashboards
  - [ ] KPI tracking
  - [ ] Predictive analytics
  - [ ] Benchmarking against industry
  - [ ] Custom report builder

- [ ] **Multi-Practice Support**
  - [ ] Corporate/group practice management
  - [ ] Multi-location support
  - [ ] Centralized reporting
  - [ ] Role-based access across locations

**Deliverable**: Market-competitive feature set

### Conformance & Certification Timeline

**Parallel to Development**:

| Certification | Timeline | Cost | Required For |
|---------------|----------|------|--------------|
| Services Australia (Medicare, AIR) | 6-12 months | Free (time cost) | Phase 3-4 |
| HI Service | 3-6 months | Free (time cost) | Phase 4 |
| My Health Record | 12-18 months | $5k-$15k | Phase 4 |
| TGA Medical Device (if needed) | 6-24 months | $5k-$50k+ | Phase 5 (CDS) |

**Total Certification Timeline**: Plan for **18-24 months** of conformance processes running in parallel with development.

### Effort Summary

| Phase | Duration | Estimated Effort (person-weeks) |
|-------|----------|--------------------------------|
| Phase 1 | 3-4 months | 12-16 weeks |
| Phase 2 | 4-6 months | 24-32 weeks |
| Phase 3 | 4-6 months | 24-32 weeks |
| Phase 4 | 6-12 months | 36-52 weeks |
| Phase 5 | 12-18 months | 68-96 weeks |
| **Total** | **29-46 months** | **164-228 weeks** |

**Team Size**: Start with 1-2 developers (Phase 1), scale to 3-5 developers (Phase 4-5)

**Budget Estimate** (development + certification):
- Development: $500k-$1.5M (depending on team location and rates)
- Certification: $25k-$50k
- Ongoing subscriptions: $2k-$5k/year (MIMS, secure messaging)
- **Total**: $525k-$1.55M over 3-4 years

---

## Conformance & Certification

### Overview

To operate legally and integrate with Australian government systems, OpenGP **must obtain** several conformance certifications. This section outlines the process, timeline, and costs.

### Services Australia Conformance

**Required For**: Medicare Online, ECLIPSE, AIR claiming

**Certification Process**:

1. **Developer Registration** (1-2 weeks)
   - Create individual PRODA account
   - Register organization in Health Systems Developer Portal
   - Invite staff members
   - Complete identity verification

2. **Test Environment Access** (2-4 weeks)
   - Request test credentials from Services Australia
   - Set up development environment
   - Receive test data and scenarios
   - Configure PRODA test authentication

3. **Development & Internal Testing** (12-20 weeks)
   - Implement Medicare Online web services
   - Implement AIR web services
   - Unit testing
   - Integration testing
   - Security testing
   - Performance testing

4. **Conformance Testing** (8-12 weeks)
   - Submit software for conformance testing
   - Services Australia validation team reviews
   - Fix issues identified in testing
   - Re-submit and re-test until pass
   - Conformance report generated

5. **Production Registration** (2-4 weeks)
   - Apply for production access
   - Conformance certificate issued
   - Listed on Services Australia Conformance Register
   - Receive production credentials
   - Go live

**Timeline**: 25-42 weeks (6-10 months)  
**Cost**: Free (testing services), staff time investment  
**Renewal**: Re-certification required for major version changes

### My Health Record Conformance

**Required For**: Connecting to My Health Record system

**Certification Process**:

1. **Developer Registration** (1 week)
   - Register on Digital Health Agency developer portal
   - Agree to terms and conditions
   - Choose integration method (FHIR Gateway recommended)

2. **Test Environment Setup** (2-4 weeks)
   - Request NASH certificates
   - Set up test environment
   - Access My Health Record test system
   - Configure test patient records

3. **Development** (16-24 weeks)
   - Implement FHIR Gateway API v4.0+
   - Document upload functionality
   - Document retrieval functionality
   - Error handling and retry logic
   - Audit logging

4. **Security Conformance** (4-8 weeks)
   - **Penetration Testing** by ADHA-approved assessor
     - Cost: $5,000-$15,000
     - External security firm
     - Comprehensive security assessment
   - **Vulnerability Assessment**
   - Implement fixes for identified issues
   - Re-test until pass

5. **Functional Conformance Testing** (8-12 weeks)
   - Execute conformance test specification
   - Test all required scenarios
   - Document test results
   - Fix issues
   - Re-test

6. **Conformance Assessment** (4-6 weeks)
   - Submit all test results to ADHA
   - ADHA review and validation
   - Address any concerns raised
   - Conformance certificate issued
   - Listed on Australian Register of Conformity

**Timeline**: 35-55 weeks (8-13 months)  
**Cost**: $5,000-$15,000 (penetration testing)  
**Renewal**: Annual security assessment required

### HI Service Conformance

**Required For**: Using healthcare identifiers (IHI, HPI-I, HPI-O)

**Certification Process**: Similar to Services Australia  
**Timeline**: 12-26 weeks (3-6 months)  
**Cost**: Free (testing services)

### Electronic Prescribing Conformance

**Required For**: E-prescribing functionality

**Certification Process**:
- Implement Electronic Prescribing Conformance Profile v3.0.1
- Integration with prescription exchange provider (eRx, MediSecure)
- Conformance testing with provider
- Certificate issued

**Timeline**: 8-16 weeks (2-4 months)  
**Cost**: Free to $5,000 (depending on provider)

### TGA Medical Device Classification (Conditional)

**Required If**: Software provides diagnostic or therapeutic decision support

**Classification Assessment**:
- **Class I**: Low risk (most practice management software) - Self-assessment
- **Class IIa**: Medium-low risk (clinical decision support) - Conformity assessment
- **Class IIb**: Medium-high risk - External assessment required
- **Class III**: High risk - Full TGA review

**For Class IIa and above**:

1. **Quality Management System** (6-12 months)
   - Implement ISO 13485:2016
   - External certification by NATA-accredited body
   - Cost: $20,000-$50,000

2. **ARTG Registration** (3-6 months)
   - Prepare technical documentation
   - Clinical evidence compilation
   - Risk management documentation
   - Apply to TGA
   - TGA review and approval
   - Cost: $5,000-$20,000 (TGA fees)

**Timeline**: 9-18 months  
**Cost**: $25,000-$70,000  
**Renewal**: Annual ARTG renewal fees

### Conformance Timeline Summary

| Certification | Priority | Timeline | Cost | Start Phase |
|---------------|----------|----------|------|-------------|
| Services Australia | CRITICAL | 6-10 months | Free | Phase 3 |
| HI Service | CRITICAL | 3-6 months | Free | Phase 3 |
| My Health Record | HIGH | 8-13 months | $5k-$15k | Phase 4 |
| E-Prescribing | HIGH | 2-4 months | $0-$5k | Phase 3 |
| TGA Medical Device | MEDIUM | 9-18 months | $25k-$70k | Phase 5 |

**Total Certification Cost**: $30,000-$90,000 over 3-4 years

### Pre-Certification Checklist

Before starting conformance processes:

- [ ] Functional requirements 100% complete
- [ ] Security requirements implemented
- [ ] Audit logging fully functional
- [ ] Integration code complete and tested
- [ ] Internal security assessment passed
- [ ] Test environment access obtained
- [ ] Budget approved for external assessments
- [ ] Project timeline accounts for certification wait times

---

## Open Source Considerations

### License

**GNU Affero General Public License (AGPL-3.0)**

- Ensures modifications shared with community
- Protects against proprietary forks
- Common in healthcare open source

### Quality Assurance

- Automated testing (unit, integration)
- Code review requirements
- CI/CD pipeline
- Security scanning (cargo-audit)

---

## References

- [OAIC Guide to Health Privacy](https://www.oaic.gov.au/privacy/privacy-guidance-for-organisations-and-government-agencies/health-service-providers/guide-to-health-privacy)
- [RACGP Standards 5th Edition](https://www.racgp.org.au/running-a-practice/practice-standards/standards-5th-edition)
- [Australian Digital Health Agency Developer Portal](https://developer.digitalhealth.gov.au/)
- [Ratatui Documentation](https://ratatui.rs/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [HL7 Australia FHIR](https://hl7.org.au/fhir/)

---

*Document Version: 1.1*
*Last Updated: 2026-02-17*

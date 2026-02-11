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
- **Database Flexibility**: Start with SQLite, migrate to PostgreSQL for larger practices

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

- Electronic prescriptions (e-prescribing)
- PBS/RPBS integration
- Authority prescriptions
- Medication interactions checking
- Allergy alerts
- Repeat prescriptions management

### 5. Pathology & Imaging

- Electronic ordering
- Results delivery and acknowledgment
- Abnormal result flagging
- Result trending/graphing

### 6. Billing & Medicare

- Medicare Online claiming
- Bulk billing and private billing
- DVA claiming
- Invoice generation
- Payment processing

### 7. Referrals

- Specialist and allied health referrals
- Referral templates
- Referral tracking

### 8. Recalls & Reminders

- Preventive health recalls
- Chronic disease management
- Immunization reminders

### 9. Reporting & Analytics

- Clinical audit reports
- Practice activity reports
- Accreditation reports

### 10. Multi-Practitioner Support

- User roles and permissions
- Individual practitioner schedules
- Shared patient records

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
│  Data Layer (SQLx - SQLite/PostgreSQL)                   │
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
├── LICENSE
├── migrations/
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── ui/           # TUI components
│   ├── components/   # UI component modules
│   ├── domain/       # Business logic
│   ├── infrastructure/  # Database, crypto, auth
│   └── integrations/    # External APIs
└── tests/
```

---

## Database Design

### Design Principles

1. **Portability**: Use ANSI SQL for SQLite → PostgreSQL migration
2. **Audit Trail**: All clinical data includes audit columns
3. **Soft Deletes**: Never hard delete clinical data
4. **Encryption**: Sensitive fields encrypted at application level

### Core Tables

- **users**: Authentication and authorization
- **practitioners**: Doctors, nurses with HPI-I
- **patients**: Demographics, Medicare, IHI
- **appointments**: Scheduling with status tracking
- **consultations**: Clinical encounters with SOAP notes
- **prescriptions**: Medications with PBS support
- **patient_allergies**: Allergen tracking
- **audit_log**: Append-only audit trail

### SQLite to PostgreSQL Strategy

- Use SQLx with feature flags
- Repository pattern for abstraction
- Compile-time query validation
- Migration tool: sqlx-cli

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
  - Column-level encryption at application layer
  - SQLite database file encryption (SQLCipher) for additional protection
  - PostgreSQL: Use pgcrypto or application-level encryption

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

### Phase 1: Foundation (MVP) - 3-4 months
- Project setup and architecture
- Database schema and migrations
- Authentication system
- Patient management (CRUD)
- Basic TUI framework

### Phase 2: Clinical Core - 3-4 months
- Appointment scheduling
- Consultation/SOAP notes
- Medical history recording
- Allergy management

### Phase 3: Prescribing & Billing - 3-4 months
- Prescription management
- PBS integration
- Medicare claiming
- Invoice generation

### Phase 4: Integrations - 4-6 months
- PRODA authentication
- Medicare Online
- HI Service
- My Health Record

### Phase 5: Advanced Features - Ongoing
- Clinical decision support
- Advanced reporting
- Multi-practice support

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

*Document Version: 1.0*
*Last Updated: 2026-02-11*

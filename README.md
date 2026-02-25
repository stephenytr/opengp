# OpenGP

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

> Open Source General Practice Management Software for Australia

OpenGP is a terminal-based general practice management system built with Rust, designed specifically for Australian healthcare providers. It prioritizes privacy, security, and compliance with Australian healthcare regulations.

## Features

- **Patient Management** - Demographics, Medicare, healthcare identifiers
- **Appointment Scheduling** - Multi-practitioner calendar with reminders
- **Clinical Records** - SOAP notes, medical history, allergies
- **Prescriptions** - E-prescribing with PBS integration
- **Billing** - Medicare claiming, bulk billing, invoicing
- **Security** - Encryption, audit logging, RBAC

## Technology Stack

- **Language**: Rust
- **UI Framework**: Ratatui (Terminal UI)
- **UI Components**: ratatui-textarea
- **Database**: SQLite (PostgreSQL migration path)
- **Async Runtime**: Tokio

## Australian Compliance

OpenGP is designed to comply with:

- Privacy Act 1988 (Australian Privacy Principles)
- My Health Records Act 2012
- Healthcare Identifiers Act 2010
- RACGP Standards for General Practices (5th Edition)

## Getting Started

### Prerequisites

- Rust 1.85 or later
- SQLite 3.x

### Building

```bash
git clone https://gitea.snares-kitchen.ts.net/stephenp/opengp.git
cd opengp
cargo build --release
```

### Running

```bash
cargo run --release
```

## Documentation

📚 **[View Full Documentation Wiki](https://gitea.snares-kitchen.ts.net/stephenp/opengp/wiki)**

The wiki contains comprehensive guides for:
- [Getting Started](https://gitea.snares-kitchen.ts.net/stephenp/opengp/wiki/Getting-Started) - Installation & Setup
- [Quick Start](https://gitea.snares-kitchen.ts.net/stephenp/opengp/wiki/Quick-Start) - Usage Tutorial
- [Keyboard Shortcuts](https://gitea.snares-kitchen.ts.net/stephenp/opengp/wiki/Keyboard-Shortcuts) - Complete Reference
- [Architecture](https://gitea.snares-kitchen.ts.net/stephenp/opengp/wiki/Architecture) - Technical Design
- [Development Guide](https://gitea.snares-kitchen.ts.net/stephenp/opengp/wiki/Development-Guide) - Contributing
- [Compliance Guide](https://gitea.snares-kitchen.ts.net/stephenp/opengp/wiki/Compliance-Guide) - Australian Healthcare

For detailed specifications, see [REQUIREMENTS.md](REQUIREMENTS.md) and [ARCHITECTURE.md](ARCHITECTURE.md).

## Contributing

Contributions are welcome! Please read the [Development Guide](https://gitea.snares-kitchen.ts.net/stephenp/opengp/wiki/Development-Guide) before submitting PRs.

## License

This project is licensed under the GNU Affero General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## Disclaimer

This software is provided as-is and is not a certified medical device. Users are responsible for ensuring compliance with applicable healthcare regulations in their jurisdiction.

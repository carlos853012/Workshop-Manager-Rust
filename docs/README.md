# WorkshopManager Documentation

> Sistema de gestión integral para talleres de motocicletas — Documentación técnica y operativa.

**Version:** 0.1.0 | **Last Updated:** 2026-09-13 | **Status:** Active Development

---

## Quick Navigation

### For End Users
| Document | Description |
|----------|-------------|
| [Getting Started](06-user-guide/getting-started.md) | First-time setup and initial configuration |
| [User Guide](06-user-guide/dashboard.md) | Complete user manual by module |

### For Developers
| Document | Description |
|----------|-------------|
| [Developer Setup](07-developer-guide/setup.md) | Development environment configuration |
| [Project Structure](07-developer-guide/project-structure.md) | Codebase overview and architecture |
| [Coding Conventions](07-developer-guide/conventions.md) | Style guide and patterns |
| [API Reference](02-api-reference/common.md) | Complete REST API documentation |

### For Operations
| Document | Description |
|----------|-------------|
| [Deployment Guide](05-deployment/installation.md) | Installation and configuration |
| [Security Overview](04-security/overview.md) | Security architecture and controls |
| [Monitoring](09-operations/monitoring.md) | Observability and alerting |

### For Management
| Document | Description |
|----------|-------------|
| [Architecture Overview](01-architecture/overview.md) | System design and technology decisions |
| [OWASP Compliance](08-compliance/owasp-top10.md) | Security compliance checklist |
| [Changelog](CHANGELOG.md) | Version history and roadmap |

---

## Documentation Index

### 01 — Architecture
| Document | Description |
|----------|-------------|
| [Overview](01-architecture/overview.md) | High-level architecture, design decisions, quality attributes |
| [System Design](01-architecture/system-design.md) | C4 diagrams, component design, integration patterns |
| [Data Flow](01-architecture/data-flow.md) | Request lifecycle, transaction flows, error propagation |
| [Technology Stack](01-architecture/technology-stack.md) | Dependencies, rationale, version compatibility |

### 02 — API Reference
| Document | Description |
|----------|-------------|
| [Common Patterns](02-api-reference/common.md) | Envelope format, pagination, errors, auth headers |
| [Authentication](02-api-reference/authentication.md) | Login, register, JWT token lifecycle |
| [Products](02-api-reference/products.md) | Inventory CRUD, barcode generation, POS lookup |
| [Sales](02-api-reference/sales.md) | POS transactions, stock deduction, IVA calculation |
| [Repairs](02-api-reference/repairs.md) | Repair orders, parts, status history |
| [Suppliers](02-api-reference/suppliers.md) | Supplier directory CRUD |
| [Reports](02-api-reference/reports.md) | Analytics, client reports, certificates |
| [Users](02-api-reference/users.md) | User management (admin) |
| [Device Keys](02-api-reference/device-keys.md) | Device authentication keys |

### 03 — Database
| Document | Description |
|----------|-------------|
| [Schema](03-database/schema.md) | Complete schema reference (11 tables, 4 enums) |
| [Migrations](03-database/migrations.md) | Migration history and rollback procedures |
| [Indexes](03-database/indexes.md) | Index strategy and performance considerations |
| [ER Diagram](03-database/er-diagram.md) | Entity-Relationship diagram |

### 04 — Security
| Document | Description |
|----------|-------------|
| [Overview](04-security/overview.md) | Defense in depth, threat model, controls |
| [Authentication](04-security/authentication.md) | Argon2id, JWT, rate limiting |
| [Authorization](04-security/authorization.md) | RBAC, middleware stack, data isolation |
| [Encryption](04-security/encryption.md) | AES-256-GCM, TLS, secrets management |
| [Audit](04-security/audit.md) | Audit trail, redaction, retention |
| [Hardening](04-security/hardening.md) | OWASP checklist, server hardening |

### 05 — Deployment
| Document | Description |
|----------|-------------|
| [Prerequisites](05-deployment/prerequisites.md) | System requirements |
| [Installation](05-deployment/installation.md) | Step-by-step setup |
| [Configuration](05-deployment/configuration.md) | Config files reference |
| [Backup & Recovery](05-deployment/backup-recovery.md) | Backup strategy and procedures |
| [Troubleshooting](05-deployment/troubleshooting.md) | Common issues and solutions |

### 06 — User Guide
| Document | Description |
|----------|-------------|
| [Getting Started](06-user-guide/getting-started.md) | First-time setup wizard |
| [Dashboard](06-user-guide/dashboard.md) | KPIs and analytics |
| [Inventory](06-user-guide/inventory.md) | Product management |
| [Point of Sale](06-user-guide/pos.md) | POS workflow |
| [Repairs](06-user-guide/repairs.md) | Repair management |
| [Suppliers](06-user-guide/suppliers.md) | Supplier directory |
| [Reports](06-user-guide/reports.md) | Client reports |
| [Service Certificates](06-user-guide/service-certificates.md) | PDF certificates |
| [Administration](06-user-guide/administration.md) | Users and device keys |

### 07 — Developer Guide
| Document | Description |
|----------|-------------|
| [Setup](07-developer-guide/setup.md) | Development environment |
| [Project Structure](07-developer-guide/project-structure.md) | Codebase overview |
| [Conventions](07-developer-guide/conventions.md) | Coding standards |
| [Testing](07-developer-guide/testing.md) | Test strategy |
| [Component Library](07-developer-guide/component-library.md) | Atomic Design reference |
| [Contributing](07-developer-guide/contributing.md) | Contribution workflow |

### 08 — Compliance
| Document | Description |
|----------|-------------|
| [OWASP Top 10](08-compliance/owasp-top10.md) | Security compliance checklist |
| [Data Protection](08-compliance/data-protection.md) | GDPR and Ley 19.628 compliance |
| [Coding Standards](08-compliance/coding-standards.md) | Rust and security coding standards |

### 09 — Operations
| Document | Description |
|----------|-------------|
| [Monitoring](09-operations/monitoring.md) | Observability and alerting |
| [Incident Response](09-operations/incident-response.md) | Incident playbook |
| [Maintenance](09-operations/maintenance.md) | Routine procedures |

### Reference
| Document | Description |
|----------|-------------|
| [Changelog](CHANGELOG.md) | Version history |
| [Glossary](GLOSSARY.md) | Domain terminology |
| [Acronyms](ACRONYMS.md) | Abbreviations |

---

## Document Standards

This documentation follows these international standards:

- **ISO/IEC 27001** — Information security management
- **OWASP Top 10 (2021)** — Web application security
- **RFC 7519** — JSON Web Token
- **NIST SP 800-63B** — Digital identity guidelines
- **GDPR** — Data protection (EU)
- **Ley 19.628** — Protección de datos personales (Chile)
- **C4 Model** — Software architecture documentation
- **Keep a Changelog** — Changelog format

---

## Maintenance

Documentation is maintained alongside code changes. All PRs must include documentation updates for:
- New API endpoints
- Schema changes
- Security-relevant changes
- Configuration changes
- New features or modules

**Documentation Owner:** Engineering Team
**Review Cycle:** Quarterly
**Last Full Review:** 2026-09-13

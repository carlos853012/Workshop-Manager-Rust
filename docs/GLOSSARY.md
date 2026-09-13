# Glossary — WorkshopManager

**Version:** 0.1.0
**Last Updated:** 2026-09-13
**Audience:** All users and developers

---

## Workshop Management Terms

| Term | Definition |
|------|------------|
| **Workshop** | A motorcycle repair and maintenance business. The primary entity in WorkshopManager. |
| **Product** | An item sold by the workshop (e.g., parts, accessories, oil). |
| **Sale** | A transaction where products are sold to a customer. |
| **Sale Item** | An individual line item within a sale, including quantity and price. |
| **Repair** | A service order for motorcycle maintenance or repair. |
| **Repair Status** | The current state of a repair: Pending, InProgress, Completed, or Cancelled. |
| **Supplier** | A vendor that provides products to the workshop. |
| **Customer** | An individual who purchases products or services from the workshop. |
| **Point of Sale (POS)** | Interface for processing sales transactions quickly. |
| **Dashboard** | Main screen showing key business metrics and summaries. |
| **Inventory** | Collection of products available for sale or use in repairs. |

---

## Financial Terms

| Term | Definition |
|------|------------|
| **CLP** | Chilean Peso — the official currency of Chile. All prices are in CLP. |
| **IVA** | Impuesto al Valor Agregado — Chilean Value Added Tax (19%). |
| **Subtotal** | Total before tax calculation. |
| **Taxable** | Amount subject to IVA calculation. |
| **Total** | Final amount including IVA. |
| **Invoice (Factura)** | Official tax document recording a sale. |
| **Quote (Presupuesto)** | Estimated cost for products/services before acceptance. |
| **Payment Method** | How a sale is paid: Cash, Card, or Transfer. |
| **Margin** | Difference between cost price and selling price. |
| **Markup** | Percentage added to cost to determine selling price. |

---

## Technical Terms

| Term | Definition |
|------|------------|
| **JWT** | JSON Web Token — a compact, URL-safe means of representing claims between two parties. Used for authentication. |
| **HS256** | HMAC-SHA256 — symmetric signing algorithm used for JWT tokens. |
| **Argon2id** | Memory-hard password hashing algorithm, recommended by OWASP. |
| **AES-256-GCM** | Advanced Encryption Standard with 256-bit key in Galois/Counter Mode — authenticated encryption for data at rest. |
| **TLS** | Transport Layer Security — cryptographic protocol for secure communications. Version 1.3 is used. |
| **RBAC** | Role-Based Access Control — access management based on user roles. |
| **API** | Application Programming Interface — set of rules for software communication. |
| **CORS** | Cross-Origin Resource Sharing — mechanism for restricted resources on web pages. |
| **HSTS** | HTTP Strict Transport Security — security policy mechanism for web apps. |
| **CSPRNG** | Cryptographically Secure Pseudo-Random Number Generator. |
| **Database Pool** | Collection of reusable database connections for performance. |
| **Migration** | Versioned database schema changes applied sequentially. |
| **Middleware** | Software that runs between request and response processing. |
| **Endpoint** | A specific URL where an API can be accessed. |
| **HTTP Status Code** | Three-digit code indicating request result (e.g., 200 OK, 401 Unauthorized). |

---

## Chilean-Specific Terms

| Term | Definition |
|------|------------|
| **Patente** | Vehicle license plate in Chile. Used to identify motorcycles. |
| **PPU** | Placa Patente Única — unique vehicle license plate number. |
| **Factura** | Official tax invoice required by Chilean law. |
| **Boleta** | Simplified receipt for smaller transactions. |
| **RUT** | Rol Único Tributario — Chilean tax identification number. |
| **SII** | Servicio de Impuestos Internos — Chilean Internal Tax Service. |
| **Giro** | Business activity classification for tax purposes. |
| **Factura Electrónica** | Electronic invoice compliant with SII regulations. |
| **Token** | Digital certificate for signing electronic invoices. |

---

## Component Terms (Atomic Design)

| Term | Definition |
|------|------------|
| **Atom** | Smallest UI component (Button, Input, Badge, Icon, Spinner). |
| **Molecule** | Combination of atoms forming a functional unit (Card, Modal, FormGroup). |
| **Organism** | Complex UI component combining molecules (Header, DataTable, ConnectionSettings). |
| **Page** | Full screen component combining organisms (Home, Products, Sales). |
| **Token** | Design variable (color, spacing, typography) stored in TOML files. |
| **Theme** | Collection of tokens defining visual appearance (light/dark). |
| **Component Library** | Reusable UI components following Atomic Design principles. |

---

## Security Terms

| Term | Definition |
|------|------------|
| **Authentication** | Verifying user identity (login process). |
| **Authorization** | Determining user permissions after authentication. |
| **Encryption** | Converting data into coded form for security. |
| **Hashing** | One-way transformation of data for verification. |
| **Salt** | Random data added to password before hashing. |
| **Token** | Digital credential for authentication (JWT). |
| **Secret** | Cryptographic key or password used for security. |
| **Certificate** | Digital document proving ownership of a public key. |
| **Audit Trail** | Log of all system activities for security and compliance. |
| **Soft Delete** | Marking data as deleted without physically removing it. |

---

## Database Terms

| Term | Definition |
|------|------------|
| **PostgreSQL** | Open-source relational database system used by WorkshopManager. |
| **Schema** | Database structure definition (tables, columns, relationships). |
| **Query** | Request for data from the database. |
| **Transaction** | Group of operations that succeed or fail together. |
| **Index** | Data structure improving query performance. |
| **Pool** | Collection of reusable database connections. |
| **Migration** | Versioned schema change applied to database. |
| **VACUUM** | PostgreSQL operation to reclaim storage and update statistics. |
| **ANALYZE** | PostgreSQL operation to update table statistics for query optimizer. |

---

## Deployment Terms

| Term | Definition |
|------|------------|
| **Embedded Database** | PostgreSQL running within the application process. |
| **Self-Signed Certificate** | TLS certificate not issued by a Certificate Authority. |
| **Backup** | Copy of data for disaster recovery. |
| **Retention** | How long backups are kept (7 days for WorkshopManager). |
| **Rollback** | Reverting to a previous version after failed upgrade. |
| **Health Check** | Endpoint to verify service availability (`GET /health`). |
| **Graceful Shutdown** | Clean shutdown process that completes pending operations. |

---

## Cross-References

| Document | Description |
|----------|-------------|
| [ACRONYMS.md](./ACRONYMS.md) | All abbreviations used in documentation |
| [API Reference](./02-api-reference/common.md) | Common API types and responses |
| [Architecture Overview](./01-architecture/overview.md) | System design and components |

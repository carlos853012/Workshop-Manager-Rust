# Changelog — WorkshopManager

All notable changes to WorkshopManager will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Planned
- DPAPI (Windows) / Keychain (macOS) secrets storage
- Token revocation mechanism
- `workshop_id` scoping for device keys
- `cargo audit` in CI pipeline
- Password history enforcement
- CSP and Permissions-Policy security headers
- Automated crypto key rotation
- Online validation server deployment

### Added (since 0.1.0)
- **Licensing system**: Ed25519-signed, hardware-bound licenses with 5 tiers
- **license-tool**: CLI for generating, verifying, and migrating licenses
- **Hardware fingerprinting**: CPU + Motherboard + Disk SHA-256 hash
- **Viewer limits**: Configurable per license tier (Trial:1, Base:2, Reports:5, Advanced:10, API:999)
- **License migration**: Transfer licenses between machines with transfer count limits
- **Trial mode**: 7-day trial license on first run
- **Tray icon enhancements**: Copy API Key, Copy Device Key, license status display
- **Sale cancellation endpoint**: `POST /api/sales/:id/cancel`
- **Audit log endpoint**: `GET /api/audit` (admin only)
- **Backup restore script**: `scripts/restore.ps1`
- **User manual**: Complete Spanish documentation for clients
- **Configurable IVA rate**: `[tax] iva_rate` in server.toml
- **Configurable icon colors**: `[icon] bg` and `[icon] fg` in server.toml

---

## [0.1.0] - 2026-09-13

### Added

**Core Features:**
- Product management (CRUD operations)
- Sales management with point-of-sale (POS) interface
- Repair order tracking with status workflow
- Supplier management
- Customer management
- Dashboard with key metrics and charts
- Reports generation

**Authentication & Security:**
- JWT authentication with HS256 signing
- Argon2id password hashing (64MB memory, 3 iterations)
- AES-256-GCM encryption for data at rest
- TLS 1.3 transport encryption
- Role-based access control (Admin, Mechanic, Seller)
- Rate limiting on login attempts (5 per 300 seconds)
- API key authentication
- Device key authentication with SHA-256 hashing
- Admin self-protection rules

**Infrastructure:**
- Embedded PostgreSQL database
- Automatic database migrations (8 migration files)
- Automatic backup scheduler (24-hour interval, 7-day retention)
- Self-signed TLS certificate generation
- Health check endpoint (`GET /health`)
- Structured logging with `tracing-subscriber`
- Graceful shutdown handling

**Multi-tenancy:**
- `workshop_id` isolation for all data
- Multi-workshop support via JWT claims

**Audit & Compliance:**
- Complete audit trail for all CRUD operations
- Credential redaction in audit logs
- Sensitive field masking

**Frontend (Viewer):**
- Dioxus 0.6 Desktop application
- 13 pages: Home, Login, Setup, Products, Sales, POS, Repairs, Suppliers, Reports, Users, DeviceKeys
- Atomic Design component system (atoms, molecules, organisms)
- Theme system with light/dark mode (TOML tokens)
- Responsive data tables with sorting and filtering
- Modal dialogs with confirmation workflows
- Form validation with error display
- API client with typed endpoints
- Internationalization support (i18n)
- Connection settings management

**Developer Experience:**
- Cargo workspace with 3 crates
- Constitutional coding rules (no `unsafe`, no `unwrap()`)
- Comprehensive documentation (42+ files)
- CI/CD ready with linting and testing
- Version bump script

### Security
- All passwords hashed with Argon2id before storage
- JWT tokens expire after 8 hours
- Secrets encrypted with AES-256-GCM
- TLS 1.3 for all client-server communication
- CORS restricted to localhost
- Security headers: HSTS, X-Content-Type-Options, X-Frame-Options
- Request body size limit: 10MB
- SQL injection prevented via parameterized queries
- Audit logging with credential redaction

---

## [0.2.0] - Planned

### Planned

**Security Hardening:**
- Migrate secrets to DPAPI (Windows) / Keychain (macOS)
- Implement token revocation mechanism
- Add `workshop_id` scoping to device keys
- Add `cargo audit` to CI pipeline
- Implement password history (last N passwords)
- Add CSP and Permissions-Policy security headers
- Implement automated key rotation for crypto key

**Features:**
- Enhanced reporting with export capabilities
- Barcode generation for products
- Service certificate generation
- Advanced search and filtering
- Batch operations
- Data import/export

**Performance:**
- Query optimization
- Connection pool tuning
- Caching layer
- Pagination improvements

**Developer Experience:**
- Integration test suite
- API documentation with OpenAPI/Swagger
- Performance benchmarks
- Load testing scripts

---

## Versioning Strategy

### Semantic Versioning

- **Major (X.0.0)**: Breaking changes, major feature additions
- **Minor (0.X.0)**: New features, backward compatible
- **Patch (0.0.X)**: Bug fixes, security patches

### Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run `.\scripts\bump.ps1 X.Y.Z`
4. Create git tag
5. Push to repository

### Release Channels

- **Stable**: Production-ready releases
- **Beta**: Pre-release testing
- **Nightly**: Development builds

---

## Support

### Version Support

| Version | Support Level | EOL Date |
|---------|---------------|----------|
| 0.1.x | Active | TBD |
| 0.2.x | Planned | TBD |

### Upgrade Path

Always upgrade to the latest minor version before moving to the next major version.

```bash
# Check current version
cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'

# Upgrade to latest
git pull
cargo build --workspace
```

---

## References

- [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
- [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
- [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/)

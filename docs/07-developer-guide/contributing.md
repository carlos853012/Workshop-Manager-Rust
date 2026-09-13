# Contributing Guide

This document describes the contribution workflow for WorkshopManager.

---

## Branch Naming

```
feat/short-description      # New features
fix/issue-description       # Bug fixes
refactor/module-name        # Code refactoring
docs/update-readme          # Documentation only
test/add-auth-tests         # Test additions
release/v0.2.0              # Release branches
```

---

## Commit Message Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

| Type | When to Use |
|------|-------------|
| `feat` | New feature or capability |
| `fix` | Bug fix |
| `refactor` | Code restructuring without behavior change |
| `docs` | Documentation changes |
| `test` | Adding or updating tests |
| `chore` | Build, CI, tooling changes |
| `perf` | Performance improvements |

### Scopes

| Scope | Crate/Module |
|-------|-------------|
| `auth` | `inventory-server/src/auth.rs` |
| `products` | `inventory-server/src/routes/products.rs` |
| `sales` | `inventory-server/src/routes/sales.rs` |
| `repairs` | `inventory-server/src/routes/repairs.rs` |
| `common` | `inventory-common` |
| `viewer` | `inventory-viewer` |
| `api` | `inventory-viewer/src/api.rs` |

### Examples

```
feat(pos): add barcode scanning support
fix(auth): prevent token refresh race condition
refactor(server): extract pagination to shared module
docs(api): document repair status transitions
test(crypto): add AES-GCM roundtrip tests
```

---

## Pull Request Process

### Before Creating a PR

1. ✅ `cargo fmt --all --check` passes
2. ✅ `cargo clippy --workspace -- -D warnings` passes
3. ✅ `cargo test --workspace` passes
4. ✅ `cargo build --workspace` succeeds
5. ✅ No secrets or keys committed
6. ✅ Database migrations are backward-compatible

### PR Template

```markdown
## Description
[What does this PR do?]

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Refactoring
- [ ] Documentation
- [ ] Tests

## Testing
- [ ] Unit tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Code follows project conventions
- [ ] No `unsafe`, `unwrap()`, or `panic!()` added
- [ ] SQL queries are parameterized
- [ ] CSS uses design tokens
- [ ] i18n for new UI strings
- [ ] Documentation updated if needed
```

### PR Title Format

Same as commit messages: `<type>(<scope>): <description>`

---

## Code Review Checklist

### Security

- [ ] No hardcoded secrets or credentials
- [ ] All inputs validated before DB insertion
- [ ] SQL queries parameterized (no string concatenation)
- [ ] `workshop_id` filter on all queries (multi-tenancy)
- [ ] Rate limiting on write endpoints
- [ ] Error messages sanitized (no DB details exposed)

### Code Quality

- [ ] No `unsafe` blocks without justification
- [ ] No `unwrap()` or `expect()` outside tests
- [ ] Functions are small and single-purpose
- [ ] Error handling uses `AppError` enum
- [ ] No dead code or unused imports

### Frontend

- [ ] CSS uses design tokens (no hardcoded values)
- [ ] Tables wrapped in `div.data-table-wrapper`
- [ ] Modals use `Modal` molecule with footer
- [ ] Cancel buttons use `ButtonVariant::Ghost`
- [ ] Responsive rules for `@media (max-width: 768px)`
- [ ] New strings added to `i18n.rs`

### Testing

- [ ] Tests cover happy path
- [ ] Tests cover error cases
- [ ] Tests cover edge cases
- [ ] No test interdependencies

---

## Testing Requirements

### Required

- All new features must include tests
- Bug fixes must include a regression test
- Tests must pass in CI before merge

### Test Commands

```powershell
cargo test --workspace                     # All tests
cargo clippy --workspace -- -D warnings    # Lint check
cargo fmt --all --check                    # Format check
```

---

## Documentation Requirements

### Code Documentation

- Public functions: doc comments (`///`)
- Complex algorithms: inline comments explaining logic
- Safety invariants: documented before `unsafe` blocks

### API Changes

- Update `docs/02-api-reference/` with new/changed endpoints
- Include request/response examples

### Frontend Changes

- Update `docs/06-user-guide/` if UI changes affect users
- Update `docs/07-developer-guide/component-library.md` for new components

---

## Release Process

### Version Bumping

```powershell
# Automated (recommended)
.\scripts\bump.ps1 0.2.0

# What it does:
# 1. Updates [workspace.package] version in Cargo.toml
# 2. Builds the workspace
# 3. Creates a git commit
# 4. Creates a git tag
```

### Manual Release Steps

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` if exists
3. Run full test suite
4. Build release binaries
5. Create git tag
6. Push with tags

```powershell
git push && git push --tags
```

### Release Checklist

- [ ] Version bumped in `Cargo.toml`
- [ ] All tests passing
- [ ] Clippy clean
- [ ] Formatting clean
- [ ] No known regressions
- [ ] Changelog updated
- [ ] Git tag created
- [ ] CI pipeline passes

---

## Dependency Management

### Adding Dependencies

Before adding any crate:

1. Check if it solves a real problem
2. Verify it's actively maintained
3. Check license compatibility
4. Evaluate size/compile time impact
5. Discuss in PR description

```toml
# Add to appropriate Cargo.toml
[dependencies]
new-crate = { version = "1.0", features = ["needed-feature"] }
```

### Updating Dependencies

```powershell
cargo update                    # Update all
cargo update -p specific-crate # Update one
```

---

## Development Workflow

### Feature Development

1. Create branch: `git checkout -b feat/feature-name`
2. Implement changes
3. Add tests
4. Run quality checks
5. Create PR with description
6. Address review feedback
7. Merge when approved

### Bug Fix

1. Create branch: `git checkout -b fix/bug-description`
2. Write failing test that reproduces bug
3. Fix the bug
4. Verify test passes
5. Create PR

### Hotfix

1. Create branch from `main`: `git checkout -b fix/critical-issue main`
2. Minimal fix
3. Add regression test
4. Fast-track review
5. Merge and tag release

# pforge Release Process

**Last Updated**: 2025-10-03
**Version**: 0.1.0

---

## Table of Contents

1. [Overview](#overview)
2. [Release Schedule](#release-schedule)
3. [Release Process](#release-process)
4. [Versioning](#versioning)
5. [Quality Gates](#quality-gates)
6. [Post-Release](#post-release)
7. [Hotfix Releases](#hotfix-releases)

---

## Overview

pforge follows [Semantic Versioning 2.0.0](https://semver.org/) and uses automated release workflows for consistency and reliability.

### Release Artifacts

Each release produces:
- **Binaries**: Linux (x86_64, musl), macOS (x86_64, ARM64), Windows (x86_64)
- **Checksums**: SHA256 for all binaries
- **Crates**: Published to crates.io
- **Documentation**: Versioned API docs on docs.rs
- **Changelog**: Auto-generated from commits

---

## Release Schedule

### Regular Releases

| Release Type | Frequency | Example | Breaking Changes |
|--------------|-----------|---------|------------------|
| **Major** | As needed | 1.0.0 → 2.0.0 | ✅ Yes |
| **Minor** | Monthly | 1.0.0 → 1.1.0 | ❌ No |
| **Patch** | As needed | 1.0.0 → 1.0.1 | ❌ No |

### Pre-1.0 Releases

During 0.x.x phase:
- **Minor** releases (0.1.0 → 0.2.0) may include breaking changes
- **Patch** releases (0.1.0 → 0.1.1) are backwards compatible
- Aim for 1.0.0 when API is stable

---

## Release Process

### Method 1: Automated Script (Recommended)

**Step 1**: Run the release script

```bash
./scripts/release.sh <version>
```

Example:
```bash
./scripts/release.sh 0.2.0
```

The script will:
1. Validate version format
2. Check working directory is clean
3. Update all Cargo.toml files
4. Update Cargo.lock
5. Run full test suite
6. Run quality gates (clippy, fmt)
7. Create release commit
8. Create git tag

**Step 2**: Review and push

```bash
# Review the commit
git show HEAD

# Push commit and tag
git push origin main
git push origin v0.2.0
```

**Step 3**: GitHub Actions takes over

The release workflow automatically:
1. Runs pre-release quality gates
2. Builds binaries for all platforms
3. Generates changelog from commits
4. Creates GitHub release with binaries
5. Publishes crates to crates.io

### Method 2: Manual Release

**Step 1**: Update versions manually

```bash
# Workspace Cargo.toml
vim Cargo.toml  # Update version

# All crate Cargo.toml files
vim crates/pforge-cli/Cargo.toml
vim crates/pforge-runtime/Cargo.toml
# ... (all 6 crates)

# Update lock file
cargo check
```

**Step 2**: Run quality gates

```bash
# Run all tests
cargo test --all --release

# Check clippy
cargo clippy --all-targets --all-features -- -D warnings

# Check formatting
cargo fmt --all -- --check

# Security audit
cargo audit
```

**Step 3**: Commit and tag

```bash
git add -A
git commit -m "chore: bump version to 0.2.0"
git tag -a v0.2.0 -m "Release v0.2.0"
```

**Step 4**: Push

```bash
git push origin main
git push origin v0.2.0
```

### Method 3: GitHub UI (Manual Trigger)

1. Go to **Actions** → **Release** workflow
2. Click **Run workflow**
3. Enter version (e.g., `0.2.0`)
4. Click **Run workflow**

This triggers the full release process without pushing a tag first.

---

## Versioning

### Semantic Versioning

Given version **MAJOR.MINOR.PATCH**:

1. **MAJOR**: Incompatible API changes
2. **MINOR**: Backwards-compatible functionality
3. **PATCH**: Backwards-compatible bug fixes

### Examples

**Patch Release** (0.1.0 → 0.1.1):
- Bug fixes
- Performance improvements
- Documentation updates
- No API changes

**Minor Release** (0.1.0 → 0.2.0):
- New features
- Deprecations (with warnings)
- Internal refactoring
- API additions (backwards-compatible)

**Major Release** (0.9.0 → 1.0.0):
- Breaking API changes
- Removed deprecated features
- Architectural changes

### Version Constraints

**Cargo.toml**:
```toml
[dependencies]
pforge-runtime = "0.1"      # ^0.1.0 (any 0.1.x)
pforge-runtime = "0.1.5"    # ^0.1.5 (>= 0.1.5, < 0.2.0)
pforge-runtime = "~0.1.5"   # >= 0.1.5, < 0.2.0
pforge-runtime = "=0.1.5"   # Exactly 0.1.5
```

---

## Quality Gates

All releases must pass:

### 1. Test Suite (Required)

```bash
cargo test --all --release
```

- Unit tests
- Integration tests (54 tests)
- Property-based tests (120K cases)
- Doc tests

**Failure = Release Blocked**

### 2. Linting (Required)

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

- Zero clippy warnings
- All lints enforced

**Failure = Release Blocked**

### 3. Formatting (Required)

```bash
cargo fmt --all -- --check
```

- Consistent formatting
- rustfmt config enforced

**Failure = Release Blocked**

### 4. Security Audit (Required)

```bash
cargo audit
```

- Zero critical vulnerabilities
- Zero high-severity vulnerabilities

**Failure = Release Blocked**

### 5. Coverage (Recommended)

```bash
cargo tarpaulin --out Html
```

- Target: ≥80% line coverage
- Informational (doesn't block)

### 6. Benchmarks (Recommended)

```bash
cargo bench --package pforge-runtime
```

- Verify no performance regressions
- Informational (doesn't block)

---

## Post-Release

### Verification

After release workflow completes:

1. **Check GitHub Release**
   - Navigate to: https://github.com/paiml/pforge/releases
   - Verify binaries uploaded (5 platforms + checksums)
   - Review auto-generated changelog

2. **Check crates.io**
   - Visit: https://crates.io/crates/pforge-cli
   - Verify new version published
   - Check all 6 crates published successfully

3. **Check docs.rs**
   - Visit: https://docs.rs/pforge-runtime
   - Verify docs built for new version

4. **Test Installation**
   ```bash
   cargo install pforge-cli
   pforge --version  # Should show new version
   ```

### Announcement

1. **GitHub Release Notes**
   - Auto-generated changelog is sufficient
   - Optionally add release highlights manually

2. **Social Media** (Optional)
   - Twitter/X announcement
   - Reddit (r/rust)
   - Discord community

3. **Documentation Updates**
   - Update README badges if needed
   - Update version references in docs

### Monitor

Track for 24-48 hours:
- GitHub Issues (bug reports)
- crates.io download stats
- docs.rs build status

---

## Hotfix Releases

For critical bugs in production:

### 1. Create Hotfix Branch

```bash
git checkout -b hotfix/0.1.1 v0.1.0
```

### 2. Fix the Bug

```bash
# Make minimal changes
vim src/critical_bug.rs

# Test the fix
cargo test
```

### 3. Bump Patch Version

```bash
./scripts/release.sh 0.1.1
```

### 4. Merge to Main

```bash
# Create PR or merge directly (emergency only)
git checkout main
git merge hotfix/0.1.1

# Push
git push origin main
git push origin v0.1.1
```

### 5. Backport if Needed

If main has diverged significantly:
```bash
git checkout main
git cherry-pick <hotfix-commit-sha>
```

---

## Troubleshooting

### Release Workflow Failed

**Symptoms**: GitHub Actions release workflow shows errors

**Common Causes**:
1. Quality gates failed (tests, clippy, audit)
2. Binary build failed (platform-specific issue)
3. crates.io publish failed (version already exists)

**Solutions**:
1. **Fix quality gates**: Address test/clippy failures locally
2. **Check build logs**: Platform-specific dependencies might be missing
3. **Version conflict**: Bump to next version, don't reuse

### Crates Not Publishing

**Symptoms**: publish-crates job fails

**Causes**:
- Missing `CARGO_TOKEN` secret
- Version already published
- Dependency not available yet

**Solutions**:
```bash
# Verify token is set in GitHub Secrets
# Settings → Secrets → CARGO_TOKEN

# Check crates.io manually
# Dependency publishing order: macro → config → runtime → codegen → bridge → cli

# Wait 30 seconds between publishes (already in workflow)
```

### Binary Downloads Failing

**Symptoms**: Release created but no binaries attached

**Causes**:
- Build job failed
- Asset upload failed
- Timeout

**Solutions**:
- Check build-release job logs
- Re-run failed jobs in GitHub UI
- Manually build and upload if needed

### Version Mismatch

**Symptoms**: Installed version doesn't match release

**Causes**:
- Cargo cache
- crates.io index delay

**Solutions**:
```bash
# Clear cargo cache
cargo clean
rm -rf ~/.cargo/registry/cache
rm -rf ~/.cargo/git

# Update index
cargo search pforge-cli

# Force specific version
cargo install pforge-cli --version 0.2.0 --force
```

---

## Rollback

If a release has critical issues:

### 1. Yank from crates.io

```bash
cargo yank --vers 0.2.0 pforge-cli
cargo yank --vers 0.2.0 pforge-runtime
# ... (all crates)
```

**Note**: Yanking prevents new downloads but doesn't break existing users

### 2. Delete GitHub Release

1. Go to https://github.com/paiml/pforge/releases
2. Click release to delete
3. Click **Delete release**
4. Keep the tag (don't delete)

### 3. Mark as Pre-release (Alternative)

Instead of deleting:
1. Edit release
2. Check **Set as a pre-release**
3. Add warning to description

### 4. Create Fix Release

```bash
# Bump to next patch version
./scripts/release.sh 0.2.1

# Include fix and explanation in commit
```

---

## Checklist

Before releasing, ensure:

- [ ] All tests pass (`cargo test --all --release`)
- [ ] No clippy warnings (`cargo clippy --all-targets --all-features -- -D warnings`)
- [ ] Formatting correct (`cargo fmt --all -- --check`)
- [ ] No security issues (`cargo audit`)
- [ ] Version bumped in all Cargo.toml files
- [ ] Cargo.lock updated (`cargo check`)
- [ ] CHANGELOG.md updated (optional, auto-generated)
- [ ] Working directory clean (`git status`)
- [ ] On main branch (`git branch`)

After releasing, verify:

- [ ] GitHub release created
- [ ] Binaries uploaded (5 platforms)
- [ ] Checksums uploaded
- [ ] Crates published to crates.io (all 6)
- [ ] Docs built on docs.rs
- [ ] Installation works (`cargo install pforge-cli`)
- [ ] Version correct (`pforge --version`)

---

## References

- [Semantic Versioning](https://semver.org/)
- [Cargo Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github)

---

**Last Updated**: 2025-10-03
**Maintained by**: pforge team

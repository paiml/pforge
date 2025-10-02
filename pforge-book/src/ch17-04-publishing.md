# Publishing Process

This chapter covers the actual mechanics of publishing crates to crates.io, including authentication, dry runs, the publication workflow, verification, and troubleshooting. We'll use pforge's real publishing experience with five interconnected crates.

## Prerequisites

Before publishing, ensure:

1. **crates.io account**: Sign up at https://crates.io using GitHub
2. **API token**: Generate at https://crates.io/me
3. **Email verification**: Verify your email address
4. **Preparation complete**: Metadata, documentation, tests (Chapters 17-01 through 17-03)

## Authentication

### Getting Your API Token

1. Visit https://crates.io/me
2. Click "New Token"
3. Name it (e.g., "pforge-publishing")
4. Set scope: "Publish new crates and update existing ones"
5. Click "Create"
6. Copy the token (you won't see it again!)

### Storing the Token

```bash
cargo login
```

Paste your token when prompted. This stores it in `~/.cargo/credentials.toml`:

```toml
[registry]
token = "your-api-token-here"
```

**Security**:
- Never commit this file to git
- Keep permissions restrictive: `chmod 600 ~/.cargo/credentials.toml`
- Regenerate if compromised

### CI/CD Authentication

For automated publishing in CI:

```yaml
# .github/workflows/publish.yml
env:
  CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

Add token as GitHub secret at: Repository Settings → Secrets → Actions

## Dry Run: Testing Without Publishing

Always dry run first. This simulates publication without actually publishing.

### Running Dry Run

```bash
cd crates/pforge-config
cargo publish --dry-run
```

Expected output:

```
   Packaging pforge-config v0.1.0 (/home/user/pforge/crates/pforge-config)
   Verifying pforge-config v0.1.0 (/home/user/pforge/crates/pforge-config)
   Compiling pforge-config v0.1.0 (/home/user/pforge/target/package/pforge-config-0.1.0)
    Finished dev [unoptimized + debuginfo] target(s) in 2.34s
```

No errors = ready to publish.

### What Dry Run Checks

1. **Packaging**: Creates `.crate` file with included files
2. **Manifest validation**: Checks Cargo.toml metadata
3. **Dependency resolution**: Verifies all dependencies available
4. **Compilation**: Builds the packaged crate from scratch
5. **Tests**: Runs all tests in the packaged crate

### Common Dry Run Errors

#### Missing Metadata

```
error: manifest has no description, license, or license-file
```

**Fix**: Add to `Cargo.toml`:

```toml
description = "Your description"
license = "MIT"
```

#### Missing Dependencies

```
error: no matching package named `pforge-config` found
```

**Fix**: Ensure dependency is published to crates.io first, or add version:

```toml
pforge-config = { path = "../pforge-config", version = "0.1.0" }
```

#### Package Too Large

```
error: package size exceeds 10 MB limit
```

**Fix**: Use `exclude` or `include` to reduce size:

```toml
exclude = ["benches/data/*", "tests/fixtures/*"]
```

## Publishing: Dependency Order

For multi-crate workspaces, publish in dependency order.

### pforge Publishing Order

```
1. pforge-config (no dependencies)
2. pforge-macro (no dependencies)
   ↓
3. pforge-runtime (depends on config)
4. pforge-codegen (depends on config)
   ↓
5. pforge-cli (depends on all)
```

**Rule**: Publish dependencies before dependents.

### Day 1: Foundation Crates

#### Step 1: Publish pforge-config

```bash
cd crates/pforge-config
cargo publish
```

Output:

```
    Updating crates.io index
   Packaging pforge-config v0.1.0 (/home/user/pforge/crates/pforge-config)
   Verifying pforge-config v0.1.0 (/home/user/pforge/crates/pforge-config)
   Compiling pforge-config v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 3.21s
   Uploading pforge-config v0.1.0 (/home/user/pforge/crates/pforge-config)
```

**Success indicators**:
- "Uploading..." message appears
- No errors
- Process completes

#### Step 2: Verify on crates.io

Wait 1-2 minutes, then visit:

https://crates.io/crates/pforge-config

Verify:
- Version shows as 0.1.0
- Description is correct
- Repository link works
- README renders

#### Step 3: Publish pforge-macro (Parallel)

Can publish immediately since it has no pforge dependencies:

```bash
cd ../pforge-macro
cargo publish
```

#### Step 4: Rate Limiting Pause

**Wait 10-15 minutes** before publishing more crates to avoid rate limiting.

### Day 1 (Continued): Dependent Crates

#### Step 5: Publish pforge-runtime

After waiting and verifying config is live:

```bash
cd ../pforge-runtime
cargo publish
```

If config isn't available yet:

```
error: no matching package named `pforge-config` found
```

**Fix**: Wait longer. crates.io indexing takes 1-2 minutes.

#### Step 6: Publish pforge-codegen (Parallel Option)

Since both runtime and codegen only depend on config:

```bash
cd ../pforge-codegen
cargo publish
```

### Day 2: Final Crate

#### Step 7: Wait and Verify

Wait until:
- pforge-runtime is visible on crates.io
- pforge-codegen is visible on crates.io
- docs.rs has built docs for both

#### Step 8: Publish pforge-cli

```bash
cd ../pforge-cli
cargo publish
```

This is the most complex crate - depends on all others.

**Critical**: Ensure `include` has templates:

```toml
include = [
    "src/**/*",
    "templates/**/*",
    "Cargo.toml",
]
```

## Handling Publishing Errors

### Error: Too Many Requests

```
error: failed to publish to crates.io

Caused by:
  the remote server responded with an error: too many crates published too quickly
```

**Cause**: Rate limiting (prevents spam)

**Fix**:
- Wait 10-15 minutes
- Retry with `cargo publish`
- Consider spreading across multiple days

### Error: Crate Name Taken

```
error: crate name `pforge` is already taken
```

**Cause**: Someone else owns this name

**Fix**:
- Choose different name
- Request name transfer if abandoned (email help@crates.io)
- Use scoped name like `your-org-pforge`

### Error: Version Already Published

```
error: crate version `0.1.0` is already uploaded
```

**Cause**: You (or someone else) already published this version

**Fix**:
- Bump version: `0.1.0` → `0.1.1`
- Update `Cargo.toml`
- Run `cargo update -w`
- Publish new version

**Note**: You cannot delete or replace published versions.

### Error: Missing Dependency

```
error: no matching package named `pforge-config` found
location searched: registry `crates-io`
required by package `pforge-runtime v0.1.0`
```

**Cause**: Dependency not yet on crates.io

**Fix**:
- Ensure dependency is published first
- Wait for crates.io indexing (1-2 minutes)
- Verify dependency is visible at `https://crates.io/crates/dependency-name`

### Error: Dirty Working Directory

```
error: 3 files in the working directory contain changes that were not yet committed
```

**Cause**: Uncommitted changes in git

**Options**:

Option 1: Commit changes first (recommended)

```bash
git add .
git commit -m "Prepare for publication"
cargo publish
```

Option 2: Force publish (use cautiously)

```bash
cargo publish --allow-dirty
```

**Warning**: `--allow-dirty` bypasses safety checks. Only use if you know what you're doing.

### Error: Network Timeout

```
error: failed to connect to crates.io
```

**Cause**: Network issues or crates.io downtime

**Fix**:
- Check internet connection
- Check crates.io status: https://status.rust-lang.org
- Retry after a few minutes
- Use different network if persistent

## Verification After Publishing

After each publication, verify it worked correctly.

### 1. Check crates.io Listing

Visit `https://crates.io/crates/your-crate-name`

Verify:
- Version is correct
- Description appears
- Keywords are visible
- Categories are correct
- Links work (repository, documentation, homepage)
- README renders properly
- License is displayed

### 2. Check docs.rs Build

Visit `https://docs.rs/your-crate-name`

Initial visit shows:

```
Building documentation...
This may take a few minutes.
```

After build completes (5-10 minutes):

Verify:
- Documentation built successfully
- All modules are present
- Examples render correctly
- Intra-doc links work
- No build warnings shown

If build fails, check build log at `https://docs.rs/crate/your-crate-name/0.1.0/builds`

### 3. Test Installation

On a clean machine or Docker container:

```bash
# Install CLI
cargo install pforge-cli

# Verify version
pforge --version

# Test functionality
pforge new test-project
cd test-project
cargo build
```

This ensures published crate actually works for users.

### 4. Test as Dependency

Create test project:

```bash
cargo new test-pforge-config
cd test-pforge-config
```

Add to `Cargo.toml`:

```toml
[dependencies]
pforge-config = "0.1.0"
```

```bash
cargo build
```

Verifies:
- Crate is downloadable
- Dependencies resolve
- Compilation succeeds

## Using --allow-dirty Flag

The `--allow-dirty` flag bypasses git cleanliness checks.

### When to Use

**Safe scenarios**:
- Automated CI/CD pipelines (working directory is ephemeral)
- Documentation-only changes (already committed elsewhere)
- Version bump commits (version updated but not committed yet)

**Unsafe scenarios**:
- Uncommitted code changes
- Experimental features not in git
- Local-only patches

### Example: CI/CD Publishing

```yaml
# .github/workflows/publish.yml
name: Publish

on:
  push:
    tags:
      - 'v*'

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Publish pforge-config
        run: |
          cd crates/pforge-config
          cargo publish --allow-dirty
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

      - name: Wait for crates.io
        run: sleep 60

      - name: Publish pforge-runtime
        run: |
          cd crates/pforge-runtime
          cargo publish --allow-dirty
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

`--allow-dirty` is needed because CI checkout might not be clean.

## Post-Publication Tasks

### 1. Tag the Release

```bash
git tag -a v0.1.0 -m "Release version 0.1.0"
git push origin v0.1.0
```

### 2. Create GitHub Release

Visit: https://github.com/your-org/your-repo/releases/new

- Tag: v0.1.0
- Title: pforge 0.1.0
- Description: Copy from CHANGELOG.md

### 3. Update Documentation

If you have separate docs site:
- Update version numbers
- Add release notes
- Update installation instructions

### 4. Announce Release

Channels to consider:
- GitHub Discussions/Issues
- Reddit: r/rust
- Twitter/X
- Discord/Slack communities
- Blog post

**Template announcement**:

```
pforge 0.1.0 released!

Zero-boilerplate MCP server framework with EXTREME TDD.

Install: cargo install pforge-cli

Changes:
- Initial release
- Native, CLI, and Pipeline tool types
- Quality gates with PMAT integration
- <1μs dispatch, <100ms cold start

Docs: https://docs.rs/pforge-runtime
Repo: https://github.com/paiml/pforge
```

### 5. Monitor for Issues

After release, watch:
- GitHub issues
- crates.io downloads
- docs.rs build status
- Community feedback

Be ready to publish a patch (0.1.1) if critical bugs appear.

## Publishing Checklist

Use this checklist for each publication:

### Pre-Publication
- [ ] All tests pass: `cargo test --all`
- [ ] Quality gates pass: `make quality-gate`
- [ ] Documentation builds: `cargo doc --no-deps`
- [ ] Dry run succeeds: `cargo publish --dry-run`
- [ ] Version bumped in `Cargo.toml`
- [ ] CHANGELOG.md updated
- [ ] Git committed: `git status` clean
- [ ] Dependencies published (if any)

### Publication
- [ ] Run: `cargo publish`
- [ ] No errors during upload
- [ ] "Uploading..." message appears
- [ ] Process completes successfully

### Verification
- [ ] crates.io listing appears
- [ ] Version number correct
- [ ] Metadata correct (description, keywords, license)
- [ ] README renders correctly
- [ ] Links work (repository, homepage, docs)
- [ ] docs.rs build starts
- [ ] docs.rs build succeeds (wait 5-10 min)
- [ ] Test installation: `cargo install crate-name`
- [ ] Test as dependency in new project

### Post-Publication
- [ ] Git tag created: `git tag -a vX.Y.Z`
- [ ] Tag pushed: `git push origin vX.Y.Z`
- [ ] GitHub release created
- [ ] Documentation updated
- [ ] Announce release
- [ ] Monitor for issues

## Troubleshooting Guide

### Problem: Publication Hangs

**Symptoms**: `cargo publish` freezes during upload

**Causes**:
- Large package size
- Slow network
- crates.io performance

**Solutions**:
- Wait patiently (can take 5+ minutes for large crates)
- Check package size: `ls -lh target/package/*.crate`
- Reduce size with `exclude` if >5MB
- Try different network

### Problem: docs.rs Build Fails

**Symptoms**: docs.rs shows "Build failed"

**Causes**:
- Missing dependencies
- Feature flags required
- Platform-specific code without guards
- Doc test failures

**Solutions**:
- View build log at `https://docs.rs/crate/name/version/builds`
- Fix errors locally: `cargo doc --no-deps`
- Add `[package.metadata.docs.rs]` configuration
- Ensure doc tests pass: `cargo test --doc`

### Problem: Can't Find Published Crate

**Symptoms**: `cargo install` fails with "could not find"

**Causes**:
- crates.io indexing delay
- Typo in crate name
- Version not specified correctly

**Solutions**:
- Wait 1-2 minutes for indexing
- Check spelling: `https://crates.io/crates/exact-name`
- Force index update: `cargo search your-crate`
- Clear cargo cache: `rm -rf ~/.cargo/registry/index/*`

### Problem: Wrong Version Published

**Symptoms**: Realized you published 0.1.0 instead of 0.2.0

**Solutions**:
- **Cannot unpublish**
- Option 1: Yank wrong version: `cargo yank --version 0.1.0`
- Option 2: Publish correct version: `0.2.0`
- Option 3: If 0.1.0 has bugs, yank and publish 0.1.1

## Complete Publishing Script

Automate the full publishing workflow:

```bash
#!/bin/bash
# scripts/publish-all.sh

set -e

CRATES=("pforge-config" "pforge-macro" "pforge-runtime" "pforge-codegen" "pforge-cli")
WAIT_TIME=120  # 2 minutes between publications

echo "Starting publication workflow..."

# Pre-flight checks
echo "Running pre-flight checks..."
cargo test --all
cargo clippy --all -- -D warnings
cargo doc --no-deps --all

# Publish each crate
for crate in "${CRATES[@]}"; do
    echo ""
    echo "========================================  "
    echo "Publishing: $crate"
    echo "========================================"

    cd "crates/$crate"

    # Dry run first
    echo "Dry run..."
    cargo publish --dry-run

    # Confirm
    read -p "Proceed with publication? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Skipped $crate"
        cd ../..
        continue
    fi

    # Publish
    cargo publish

    cd ../..

    # Wait before next (except for last crate)
    if [ "$crate" != "${CRATES[-1]}" ]; then
        echo "Waiting $WAIT_TIME seconds before next publication..."
        sleep $WAIT_TIME
    fi
done

echo ""
echo "All crates published successfully!"
echo "Don't forget to:"
echo "  1. Create git tag: git tag -a vX.Y.Z"
echo "  2. Push tag: git push origin vX.Y.Z"
echo "  3. Create GitHub release"
echo "  4. Verify on crates.io"
echo "  5. Check docs.rs builds"
```

Run with:

```bash
./scripts/publish-all.sh
```

## Summary

Publishing to crates.io involves:

1. **Authentication**: Get API token, store with `cargo login`
2. **Dry run**: Test with `cargo publish --dry-run`
3. **Dependency order**: Publish dependencies first
4. **Rate limiting**: Wait 10-15 minutes between publications
5. **Verification**: Check crates.io, docs.rs, test installation
6. **Post-publication**: Tag, release, announce

**pforge publishing experience**:
- Five crates published over two days
- Foundation crates first (config, macro)
- Then dependent crates (runtime, codegen)
- Finally CLI with all dependencies
- Hit rate limiting - spaced publications
- Caught template inclusion issue in dry run
- All verified before announcing

**Key lessons**:
- Dry run is essential
- Wait for crates.io indexing between dependent crates
- Verify each publication before continuing
- Can't unpublish - only yank
- Automation helps but manual verification required

Publishing is irreversible. Take your time, use checklists, verify everything.

---

**Previous**: [Documentation](ch17-03-documentation.md)

**Next**: [CI/CD Pipeline](ch18-00-cicd.md)

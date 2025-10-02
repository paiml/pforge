# Chapter 18: CI/CD with GitHub Actions

Continuous Integration and Continuous Deployment automate quality enforcement, testing, and releases for pforge projects. This chapter covers GitHub Actions workflows for testing, quality gates, performance tracking, and automated releases.

## CI/CD Philosophy

**Key Principles**:
1. **Fast Feedback**: Fail fast on quality violations
2. **Comprehensive Coverage**: Test on multiple platforms
3. **Quality First**: No compromises on quality gates
4. **Automated Releases**: One-click deployments
5. **Performance Tracking**: Continuous benchmarking

## Basic CI Workflow

From `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test:
    name: Test Suite
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
          components: rustfmt, clippy

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-git-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo build
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: cargo test --all --verbose

      - name: Run integration tests
        run: cargo test --package pforge-integration-tests --verbose

  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - run: cargo clippy --all-targets --all-features -- -D warnings

  build:
    name: Build
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Build
        run: cargo build --release --verbose

      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: pforge-${{ matrix.os }}
          path: |
            target/release/pforge
            target/release/pforge.exe

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cargo tarpaulin --out Xml --all-features --workspace

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml
          fail_ci_if_error: false

  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Run cargo-audit
        run: |
          cargo install cargo-audit
          cargo audit

  docs:
    name: Documentation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Build documentation
        run: cargo doc --no-deps --all-features

      - name: Check doc tests
        run: cargo test --doc
```

**Key Features**:
- Multi-platform testing (Linux, macOS, Windows)
- Multi-version testing (stable, beta)
- Caching for faster builds
- Parallel job execution
- Comprehensive coverage

## Quality Gates Workflow

```yaml
name: Quality Gates

on:
  pull_request:
  push:
    branches: [main]

jobs:
  quality:
    name: Quality Enforcement
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Check formatting
        run: cargo fmt --all -- --check
        continue-on-error: false

      - name: Run Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings
        continue-on-error: false

      - name: Run tests with coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Json --all-features --workspace

      - name: Check coverage threshold
        run: |
          COVERAGE=$(jq '.files | map(.coverage) | add / length' cobertura.json)
          echo "Coverage: $COVERAGE%"
          if (( $(echo "$COVERAGE < 80" | bc -l) )); then
            echo "Coverage below 80% threshold"
            exit 1
          fi

      - name: Check cyclomatic complexity
        run: |
          cargo install cargo-geiger
          cargo geiger --forbid-unsafe

      - name: Security audit
        run: |
          cargo install cargo-audit
          cargo audit --deny warnings

      - name: Check dependencies
        run: |
          cargo install cargo-deny
          cargo deny check

  mutation-testing:
    name: Mutation Testing
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Run cargo-mutants
        run: |
          cargo install cargo-mutants
          cargo mutants --check --minimum-test-timeout=10

      - name: Check mutation score
        run: |
          SCORE=$(grep "caught" mutants.out | awk '{print $2}')
          echo "Mutation score: $SCORE%"
          if (( $(echo "$SCORE < 90" | bc -l) )); then
            echo "Mutation score below 90% threshold"
            exit 1
          fi
```

## Performance Benchmarking Workflow

```yaml
name: Performance Benchmarks

on:
  push:
    branches: [main]
  pull_request:

jobs:
  benchmark:
    name: Run Benchmarks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable

      - name: Run benchmarks
        run: cargo bench --bench dispatch_benchmark -- --save-baseline pr-${{ github.event.number }}

      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'criterion'
          output-file-path: target/criterion/dispatch_benchmark/base/estimates.json
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true
          alert-threshold: '110%'
          comment-on-alert: true
          fail-on-alert: true
          alert-comment-cc-users: '@maintainers'

      - name: Compare with baseline
        run: |
          cargo bench --bench dispatch_benchmark -- --baseline pr-${{ github.event.number }}

  load-test:
    name: Load Testing
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Build release
        run: cargo build --release

      - name: Start server
        run: |
          ./target/release/pforge serve &
          echo $! > server.pid
          sleep 5

      - name: Run load test
        run: |
          cargo test --test load_test --release -- --nocapture

      - name: Stop server
        run: kill $(cat server.pid)

  performance-regression:
    name: Performance Regression Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: dtolnay/rust-toolchain@stable

      - name: Run SLA tests
        run: |
          cargo test --test performance_sla --release -- --nocapture

      - name: Check dispatch latency
        run: |
          cargo run --release --example benchmark_dispatch | tee results.txt
          LATENCY=$(grep "Average latency" results.txt | awk '{print $3}')
          if (( $(echo "$LATENCY > 1.0" | bc -l) )); then
            echo "Dispatch latency $LATENCY μs exceeds 1μs SLA"
            exit 1
          fi
```

## Release Workflow

From `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

env:
  CARGO_TERM_COLOR: always

jobs:
  create-release:
    name: Create Release
    runs-on: ubuntu-latest
    outputs:
      upload_url: ${{ steps.create_release.outputs.upload_url }}
    steps:
      - name: Create Release
        id: create_release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: ${{ github.ref }}
          release_name: Release ${{ github.ref }}
          draft: false
          prerelease: false

  build-release:
    name: Build Release
    needs: create-release
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            asset_name: pforge-linux-amd64
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            asset_name: pforge-linux-amd64-musl
          - os: macos-latest
            target: x86_64-apple-darwin
            asset_name: pforge-macos-amd64
          - os: macos-latest
            target: aarch64-apple-darwin
            asset_name: pforge-macos-arm64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            asset_name: pforge-windows-amd64.exe

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Prepare artifact
        shell: bash
        run: |
          if [ "${{ matrix.os }}" = "windows-latest" ]; then
            cp target/${{ matrix.target }}/release/pforge.exe ${{ matrix.asset_name }}
          else
            cp target/${{ matrix.target }}/release/pforge ${{ matrix.asset_name }}
            chmod +x ${{ matrix.asset_name }}
          fi

      - name: Upload Release Asset
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ needs.create-release.outputs.upload_url }}
          asset_path: ./${{ matrix.asset_name }}
          asset_name: ${{ matrix.asset_name }}
          asset_content_type: application/octet-stream

  publish-crate:
    name: Publish to crates.io
    runs-on: ubuntu-latest
    needs: build-release
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Publish pforge-config
        run: cd crates/pforge-config && cargo publish --token ${{ secrets.CARGO_TOKEN }}
        continue-on-error: true

      - name: Wait for crates.io
        run: sleep 30

      - name: Publish pforge-runtime
        run: cd crates/pforge-runtime && cargo publish --token ${{ secrets.CARGO_TOKEN }}
        continue-on-error: true

      - name: Wait for crates.io
        run: sleep 30

      - name: Publish pforge-codegen
        run: cd crates/pforge-codegen && cargo publish --token ${{ secrets.CARGO_TOKEN }}
        continue-on-error: true

      - name: Wait for crates.io
        run: sleep 30

      - name: Publish pforge-cli
        run: cd crates/pforge-cli && cargo publish --token ${{ secrets.CARGO_TOKEN }}
        continue-on-error: true

  publish-docker:
    name: Publish Docker Image
    runs-on: ubuntu-latest
    needs: build-release
    steps:
      - uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v2

      - name: Login to GitHub Container Registry
        uses: docker/login-action@v2
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v4
        with:
          images: ghcr.io/${{ github.repository }}

      - name: Build and push
        uses: docker/build-push-action@v4
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

## Documentation Deployment

```yaml
name: Deploy Documentation

on:
  push:
    branches: [main]

jobs:
  deploy-docs:
    name: Deploy Documentation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable

      - name: Build API documentation
        run: cargo doc --no-deps --all-features

      - name: Install mdBook
        run: |
          mkdir -p ~/bin
          curl -sSL https://github.com/rust-lang/mdBook/releases/download/v0.4.35/mdbook-v0.4.35-x86_64-unknown-linux-gnu.tar.gz | tar -xz --directory=~/bin
          echo "$HOME/bin" >> $GITHUB_PATH

      - name: Build book
        run: |
          cd pforge-book
          mdbook build

      - name: Combine docs
        run: |
          mkdir -p deploy/api
          mkdir -p deploy/book
          cp -r target/doc/* deploy/api/
          cp -r pforge-book/book/* deploy/book/
          echo '<html><head><meta http-equiv="refresh" content="0;url=book/index.html"></head></html>' > deploy/index.html

      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./deploy
          cname: pforge.dev
```

## Pre-Commit Hooks

```yaml
# .github/workflows/pre-commit.yml
name: Pre-commit

on:
  pull_request:

jobs:
  pre-commit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v4

      - name: Install pre-commit
        run: pip install pre-commit

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Run pre-commit
        run: pre-commit run --all-files
```

```.pre-commit-config.yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.4.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
      - id: check-toml

  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt --all -- --check
        language: system
        types: [rust]
        pass_filenames: false

      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy --all-targets --all-features -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false

      - id: cargo-test
        name: cargo test
        entry: cargo test --all
        language: system
        types: [rust]
        pass_filenames: false
```

## Docker Support

```dockerfile
# Dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build dependencies (cached layer)
RUN cargo build --release --bin pforge && rm -rf target/release/deps/pforge*

# Copy source code
COPY . .

# Build application
RUN cargo build --release --bin pforge

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/pforge /usr/local/bin/pforge

EXPOSE 3000

ENTRYPOINT ["pforge"]
CMD ["serve"]
```

```yaml
# docker-compose.yml
version: '3.8'

services:
  pforge:
    build: .
    ports:
      - "3000:3000"
    volumes:
      - ./forge.yaml:/app/forge.yaml:ro
    environment:
      - RUST_LOG=info
    restart: unless-stopped

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
    restart: unless-stopped

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    volumes:
      - grafana-data:/var/lib/grafana
    restart: unless-stopped

volumes:
  grafana-data:
```

## Continuous Deployment

```yaml
name: Deploy to Production

on:
  release:
    types: [published]

jobs:
  deploy:
    name: Deploy
    runs-on: ubuntu-latest
    environment: production
    steps:
      - uses: actions/checkout@v4

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v2
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: us-east-1

      - name: Login to Amazon ECR
        id: login-ecr
        uses: aws-actions/amazon-ecr-login@v1

      - name: Build, tag, and push image to Amazon ECR
        env:
          ECR_REGISTRY: ${{ steps.login-ecr.outputs.registry }}
          ECR_REPOSITORY: pforge
          IMAGE_TAG: ${{ github.ref_name }}
        run: |
          docker build -t $ECR_REGISTRY/$ECR_REPOSITORY:$IMAGE_TAG .
          docker push $ECR_REGISTRY/$ECR_REPOSITORY:$IMAGE_TAG

      - name: Deploy to ECS
        run: |
          aws ecs update-service \
            --cluster pforge-cluster \
            --service pforge-service \
            --force-new-deployment
```

## Monitoring and Alerting

```yaml
# .github/workflows/health-check.yml
name: Health Check

on:
  schedule:
    - cron: '*/15 * * * *'  # Every 15 minutes

jobs:
  health-check:
    runs-on: ubuntu-latest
    steps:
      - name: Check production endpoint
        run: |
          STATUS=$(curl -s -o /dev/null -w "%{http_code}" https://api.pforge.dev/health)
          if [ $STATUS -ne 200 ]; then
            echo "Health check failed with status $STATUS"
            exit 1
          fi

      - name: Send alert on failure
        if: failure()
        uses: dawidd6/action-send-mail@v3
        with:
          server_address: smtp.gmail.com
          server_port: 465
          username: ${{ secrets.MAIL_USERNAME }}
          password: ${{ secrets.MAIL_PASSWORD }}
          subject: Production Health Check Failed
          body: The health check for https://api.pforge.dev failed
          to: alerts@pforge.dev
```

## Best Practices

### 1. Fast CI Feedback

**Optimize with parallelism**:
```yaml
jobs:
  quick-checks:
    runs-on: ubuntu-latest
    steps:
      - run: cargo fmt --check & cargo clippy & cargo test --lib
```

**Use matrix strategies**:
```yaml
strategy:
  matrix:
    rust: [stable, beta, nightly]
  fail-fast: false  # Continue other jobs on failure
```

### 2. Caching Strategy

```yaml
- name: Cache everything
  uses: Swatinem/rust-cache@v2
  with:
    shared-key: "ci"
    cache-on-failure: true
```

### 3. Branch Protection Rules

Configure in GitHub Settings → Branches:

- Require pull request reviews (1+ approvals)
- Require status checks to pass:
  - fmt
  - clippy
  - test
  - quality gates
  - benchmarks
- Require branches to be up to date
- Require linear history
- Include administrators

### 4. Automated Dependency Updates

```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 5
    reviewers:
      - "maintainers"
```

### 5. Security Scanning

```yaml
- name: Run Snyk security scan
  uses: snyk/actions/rust@master
  env:
    SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}
  with:
    args: --severity-threshold=high
```

## Summary

Effective CI/CD for pforge:

1. **Multi-platform testing**: Linux, macOS, Windows
2. **Quality enforcement**: Format, lint, test, coverage
3. **Performance tracking**: Continuous benchmarking
4. **Automated releases**: Tag-based deployments
5. **Security audits**: Dependency scanning
6. **Documentation deployment**: Auto-publish docs

**Complete CI/CD pipeline**:
- Push → CI checks → Quality gates → Benchmarks
- Tag → Release → Build → Publish → Deploy
- Schedule → Health checks → Alerts

**Next chapter**: Language bridges for Python and Go integration.

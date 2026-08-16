# pforge - Declarative MCP Server Framework
# Pragmatic AI Labs
# https://paiml.com

.PHONY: all test test-all test-fast test-doc test-property build clean install coverage coverage-summary coverage-html quality-gate help

# Default target
# Property-test budget. Pinned rather than left to proptest's default so a
# green run means the same amount of searching on every machine, and so CI can
# turn it up without editing recipes (CB-126-D / CB-127-B).
PROPTEST_CASES ?= 256
QUICKCHECK_TESTS ?= 256

all: test

# Run all tests (unit, integration, doctests)
test: test-doc
	@echo "🧪 Running all tests..."
	@PROPTEST_CASES=$(PROPTEST_CASES) QUICKCHECK_TESTS=$(QUICKCHECK_TESTS) \
		cargo test --all
	@echo "✅ All tests passed!"

# Run documentation tests
test-doc:
	@echo "📚 Running documentation tests..."
	@PROPTEST_CASES=$(PROPTEST_CASES) QUICKCHECK_TESTS=$(QUICKCHECK_TESTS) \
		cargo test --doc
	@echo "✅ Doctests passed!"

# Run property-based tests (PFORGE-3002)
test-property:
	@echo "🔀 Running property-based tests..."
	@PROPTEST_CASES=$(PROPTEST_CASES) QUICKCHECK_TESTS=$(QUICKCHECK_TESTS) \
		cargo test --test property --release -- --test-threads=1
	@echo "✅ Property tests passed!"

# Run all tests including integration tests
test-all:
	@echo "🧪 Running all tests (including integration)..."
	@PROPTEST_CASES=$(PROPTEST_CASES) QUICKCHECK_TESTS=$(QUICKCHECK_TESTS) \
		cargo test --all --all-features
	@echo "✅ All tests passed!"

# Fast test run (no coverage)
test-fast:
	@echo "⚡ Running fast tests..."
	@PROPTEST_CASES=$(PROPTEST_CASES) QUICKCHECK_TESTS=$(QUICKCHECK_TESTS) \
		cargo test --lib --quiet
	@echo "✅ Fast tests passed!"

# Build all crates
build:
	@echo "🔨 Building all crates..."
	@cargo build --all
	@echo "✅ Build complete!"

# Build release
build-release:
	@echo "🚀 Building release..."
	@cargo build --all --release
	@echo "✅ Release build complete!"

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean
	@rm -rf target/
	@rm -f lcov.info coverage.json
	@echo "✅ Clean complete!"

# Install pforge CLI
install:
	@echo "📦 Installing pforge CLI..."
	@cargo install --path crates/pforge-cli
	@echo "✅ Installation complete!"

# Code coverage with llvm-cov (two-phase production pattern)
# Note: Temporarily moves ~/.cargo/config.toml to avoid mold linker interference
coverage:
	@echo "📊 Running comprehensive test coverage analysis..."
	@echo "🔍 Checking for cargo-llvm-cov..."
	@which cargo-llvm-cov > /dev/null 2>&1 || (echo "📦 Installing cargo-llvm-cov..." && cargo install cargo-llvm-cov --locked)
	@mkdir -p target/coverage
	@echo "⚙️  Temporarily disabling global cargo config (mold breaks coverage)..."
	@test -f ~/.cargo/config.toml && mv ~/.cargo/config.toml ~/.cargo/config.toml.cov-backup || true
	@echo "🧪 Phase 1: Running tests with instrumentation (no report)..."
	@PROPTEST_CASES=$(PROPTEST_CASES) QUICKCHECK_TESTS=$(QUICKCHECK_TESTS) \
		cargo llvm-cov --no-report test --lib --all-features --workspace
	@echo "📊 Phase 2: Generating coverage reports..."
	@cargo llvm-cov report --html --output-dir target/coverage/html
	@cargo llvm-cov report --lcov --output-path target/coverage/lcov.info
	@echo "⚙️  Restoring global cargo config..."
	@test -f ~/.cargo/config.toml.cov-backup && mv ~/.cargo/config.toml.cov-backup ~/.cargo/config.toml || true
	@echo ""
	@echo "📊 Coverage Summary:"
	@echo "=================="
	@cargo llvm-cov report --summary-only
	@echo ""
	@echo "💡 COVERAGE INSIGHTS:"
	@echo "- HTML report: target/coverage/html/index.html"
	@echo "- LCOV file: target/coverage/lcov.info"
	@echo "- Open HTML: make coverage-open"
	@echo ""

# Coverage summary only (run after 'make coverage')
coverage-summary:
	@cargo llvm-cov report --summary-only 2>/dev/null || echo "Run 'make coverage' first"

# Open HTML coverage report in browser
coverage-open:
	@if [ -f target/coverage/html/index.html ]; then \
		xdg-open target/coverage/html/index.html 2>/dev/null || \
		open target/coverage/html/index.html 2>/dev/null || \
		echo "Please open: target/coverage/html/index.html"; \
	else \
		echo "❌ Run 'make coverage' first to generate the HTML report"; \
	fi

# Format code
format:
	@echo "📝 Formatting code..."
	@cargo fmt --all
	@echo "✅ Formatting complete!"

# Lint code
lint:
	@echo "🔍 Linting code..."
	@cargo clippy --all-targets --all-features -- -D warnings
	@echo "✅ Linting complete!"

# Quality gate (format, lint, test, coverage, PMAT)
quality-gate: format lint test coverage
	@echo ""
	@echo "🔬 Running PMAT quality checks..."
	@echo ""
	@echo "  1. Complexity Analysis (max: 20)..."
	@pmat analyze complexity --max-cyclomatic 20 --format summary || (echo "❌ Complexity check failed" && exit 1)
	@echo ""
	@echo "  2. SATD Detection (technical debt)..."
	@pmat analyze satd --format summary || true
	@echo ""
	@echo "  3. Technical Debt Grade (TDG)..."
	@pmat tdg . || (echo "❌ TDG check failed" && exit 1)
	@echo ""
	@echo "  4. Dead Code Analysis..."
	@pmat analyze dead-code --format summary || true
	@echo ""
	@echo "✅ All quality gates passed!"
	@echo ""
	@echo "📊 Final Metrics:"
	@$(MAKE) coverage-summary
	@echo ""

# Development watch mode
watch:
	@echo "👀 Starting watch mode..."
	@cargo watch -x 'test --lib --quiet' -x 'clippy --quiet'

# Help target
help:
	@echo "pforge Makefile Commands:"
	@echo ""
	@echo "Testing:"
	@echo "  make test          - Run all tests (unit + integration + doctests)"
	@echo "  make test-doc      - Run documentation tests only"
	@echo "  make test-property - Run property-based tests (Phase 3)"
	@echo "  make test-all      - Run all tests including integration"
	@echo "  make test-fast     - Run fast tests without coverage"
	@echo ""
	@echo "Building:"
	@echo "  make build         - Build all crates"
	@echo "  make build-release - Build release version"
	@echo "  make clean         - Clean build artifacts"
	@echo ""
	@echo "Coverage:"
	@echo "  make coverage         - Generate coverage report (HTML + LCOV)"
	@echo "  make coverage-summary - Show coverage summary"
	@echo "  make coverage-open    - Open HTML coverage report in browser"
	@echo ""
	@echo "Quality:"
	@echo "  make format        - Format code"
	@echo "  make lint          - Lint code"
	@echo "  make quality-gate  - Run all quality checks"
	@echo ""
	@echo "Development:"
	@echo "  make watch         - Watch mode (test + clippy)"
	@echo "  make install       - Install pforge CLI"
	@echo "  make help          - Show this help"

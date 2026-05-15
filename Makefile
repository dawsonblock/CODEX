# CODEX-main 36 Makefile
# Common development tasks
# Usage: make <target>

.PHONY: help build test test-all proof fmt lint clean validate setup pre-commit coverage

RUST_DIR := global-workspace-runtime-rs
CODEX_SCRIPTS := scripts
VENV := .venv

# Default target: show help
help:
	@echo "╔════════════════════════════════════════════════════════╗"
	@echo "║     CODEX-main 36 Development Tasks                   ║"
	@echo "╚════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "Build & Compile:"
	@echo "  make build          Build Rust workspace (debug)"
	@echo "  make build-release  Build Rust workspace (optimized)"
	@echo "  make fmt            Format all code (Rust + Python)"
	@echo "  make lint           Run clippy linter"
	@echo ""
	@echo "Testing:"
	@echo "  make test           Run unit tests"
	@echo "  make test-all       Run all tests (unit + integration)"
	@echo "  make test-oracle    Run oracle guard tests"
	@echo "  make coverage       Generate code coverage report"
	@echo ""
	@echo "Proof & Validation:"
	@echo "  make proof          Generate proof artifacts"
	@echo "  make proof-strict   Generate proof with strict mode"
	@echo "  make validate       Run full validation suite"
	@echo ""
	@echo "Development:"
	@echo "  make pre-commit     Run pre-commit checks locally"
	@echo "  make setup          Setup development environment"
	@echo "  make clean          Clean build artifacts"
	@echo ""

# ============================================================================
# BUILD TARGETS
# ============================================================================

build:
	@echo "📦 Building CODEX-main 36 (debug)..."
	cd $(RUST_DIR) && cargo build --all --verbose
	@echo "✅ Build complete"

build-release:
	@echo "🚀 Building CODEX-main 36 (release)..."
	cd $(RUST_DIR) && cargo build --release --all
	@echo "✅ Release build complete"

# ============================================================================
# FORMAT & LINT TARGETS
# ============================================================================

fmt:
	@echo "🎨 Formatting code..."
	cargo fmt --all
	cd $(RUST_DIR) && cargo fmt --all
	@echo "✅ Formatting complete"

lint:
	@echo "🔍 Running clippy linter..."
	cd $(RUST_DIR) && cargo clippy --workspace --all-targets --all-features -- -D warnings
	@echo "✅ Lint passed (no warnings)"

fmt-check:
	@echo "🔍 Checking format (no changes)..."
	cargo fmt --all -- --check
	cd $(RUST_DIR) && cargo fmt --all -- --check
	@echo "✅ Format check passed"

# ============================================================================
# TEST TARGETS
# ============================================================================

test:
	@echo "🧪 Running unit tests..."
	cd $(RUST_DIR) && cargo test --lib --all
	@echo "✅ Unit tests passed"

test-all:
	@echo "🧪 Running all tests (unit + integration)..."
	cd $(RUST_DIR) && cargo test --workspace --all-targets --all-features
	@echo "✅ Test command completed"

test-oracle:
	@echo "🛡️  Running oracle guard tests..."
	cd $(RUST_DIR) && cargo test --workspace -- oracle
	@echo "✅ Oracle tests passed"

coverage:
	@echo "📊 Generating code coverage report..."
	@if command -v cargo-tarpaulin &> /dev/null; then \
		cd $(RUST_DIR) && cargo tarpaulin --workspace --out Html --timeout 600 \
			--exclude-files prove.rs symbolic.rs --output-dir ../coverage; \
		echo "✅ Coverage report generated: coverage/tarpaulin-report.html"; \
	else \
		echo "ℹ️  cargo-tarpaulin not installed. Install with: cargo install cargo-tarpaulin"; \
		exit 1; \
	fi

# ============================================================================
# PROOF TARGETS
# ============================================================================

proof:
	@echo "📜 Generating proof artifacts..."
	cd $(RUST_DIR) && cargo run -p runtime-cli -- proof --long-horizon --nl \
		--out ../artifacts/proof/current
	@echo "✅ Proof generation complete"

proof-strict:
	@echo "📜 Generating proof artifacts (strict mode)..."
	cd $(RUST_DIR) && cargo run -p runtime-cli -- proof --strict --long-horizon --nl \
		--out ../artifacts/proof/current
	@echo "✅ Proof generation complete (strict)"

# ============================================================================
# VALIDATION TARGETS
# ============================================================================

validate:
	@echo "✅ Running validation suite..."
	@bash scripts/validate_codex_36.sh
	@echo "✅ All validations passed"

pre-commit:
	@echo "🔒 Running pre-commit checks..."
	@echo "  1️⃣  Format checking..."
	@cd $(RUST_DIR) && cargo fmt --all -- --check || { echo "❌ Format check failed"; exit 1; }
	@echo "  2️⃣  Linting..."
	@cd $(RUST_DIR) && cargo clippy --workspace --all-targets -- -D warnings || { echo "❌ Lint failed"; exit 1; }
	@echo "  3️⃣  Unit tests..."
	@cd $(RUST_DIR) && cargo test --lib --all --quiet || { echo "❌ Tests failed"; exit 1; }
	@echo "  4️⃣  Proof validation..."
	@cd $(RUST_DIR) && cargo run -p runtime-cli -- proof --strict --long-horizon --nl \
		--out ../artifacts/proof/current 2>/dev/null | grep -q "overall_status.*pass" || { echo "❌ Proof failed"; exit 1; }
	@echo "✅ All pre-commit checks passed!"

# ============================================================================
# SETUP & MAINTENANCE TARGETS
# ============================================================================

setup:
	@echo "🔧 Setting up development environment..."
	@echo "  Installing pre-commit hook..."
	@if [ ! -d .git/hooks ]; then mkdir -p .git/hooks; fi
	@cp scripts/pre-commit-hook.sh .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit
	@echo "  ✅ Pre-commit hook installed"
	@echo ""
	@echo "  Installing Rust tools..."
	@rustup component add rustfmt clippy
	@echo "  ✅ Rust tools installed"
	@echo ""
	@echo "✅ Development environment ready!"
	@echo "   Run 'make help' for available commands"

clean:
	@echo "🧹 Cleaning build artifacts..."
	cd $(RUST_DIR) && cargo clean
	rm -rf coverage/
	rm -rf artifacts/proof/current/
	@find . -name "__pycache__" -type d -exec rm -rf {} + 2>/dev/null || true
	@find . -name "*.pyc" -delete 2>/dev/null || true
	@echo "✅ Cleanup complete"

# ============================================================================
# COMBINED WORKFLOWS
# ============================================================================

check: fmt-check lint test
	@echo "✅ Code quality checks passed"

ci: clean check proof validate
	@echo "✅ Full CI pipeline passed locally"

all: clean build test proof validate
	@echo "✅ Complete build + test + proof pipeline passed"

watch:
	@echo "👀 Watching for changes (requires cargo-watch)..."
	@cd $(RUST_DIR) && cargo watch -x "build --all"

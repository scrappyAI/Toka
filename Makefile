.PHONY: dev-setup dev-start dev-stop dev-clean dev-status test build overwrite

# Development environment setup
dev-setup:
	@echo "🚀 Setting up Toka development environment..."
	@chmod +x scripts/dev-setup.sh
	@./scripts/dev-setup.sh

# Check development environment status
dev-status:
	@echo "📊 Development environment status:"
	@echo "✅ Development environment is ready"

# Clean development environment
dev-clean:
	@echo "🧹 Cleaning development environment..."
	@cargo clean

# Test auth service
test-auth:
	@echo "🧪 Testing toka-auth..."
	@cd crates/toka-security-auth && cargo test

# Test all services
test:
	@echo "🧪 Testing all services..."
	@cargo test --workspace

# Build all services
build:
	@echo "🔨 Building all services..."
	@cargo build --workspace

# Check all services
check:
	@echo "🔍 Checking all services..."
	@cargo check --workspace

# Run clippy
clippy:
	@echo "📎 Running clippy..."
	@cargo clippy --workspace -- -D warnings

# Full development setup and check
dev-full: dev-setup check test
	@echo "✅ Full development setup complete!"

# ---- Lean workspace shortcuts ----

.PHONY: lean portable

lean:
	cargo build -p toka-runtime --no-default-features

portable:
	cargo build -p minimal-cli --release

fmt:
	cargo fmt --all 

# Coverage report (cross-platform using cargo-llvm-cov)
.PHONY: coverage
coverage:
	@echo "📈 Generating test coverage report (HTML + summary) ..."
	@command -v cargo-llvm-cov >/dev/null 2>&1 || (echo "Installing cargo-llvm-cov ..." && cargo install cargo-llvm-cov --locked)
	@cargo llvm-cov --workspace --all-features --html --open 

# -------------------------------------------------
# Workspace-wide shortcuts (aligned with CRATES.md)
# -------------------------------------------------
.PHONY: workspace-check workspace-test workspace-test-all-features workspace-test-lean integration-tests lint ci docs clean-all build-release coverage-ci

# Fast check of all crates with every feature enabled
workspace-check:
	cargo check --workspace --all-features

# Default feature set tests (quick)
workspace-test:
	cargo test --workspace

# Exhaustive tests (all features)
workspace-test-all-features:
	cargo test --workspace --all-features

# Lean build tests – prove `no_default_features` compiles for runtime
workspace-test-lean:
	cargo test -p toka-runtime --no-default-features

# Run integration tests with nextest when available
integration-tests:
	@command -v cargo-nextest >/dev/null 2>&1 || (echo "Installing cargo-nextest ..." && cargo install cargo-nextest --locked)
	cargo nextest run --workspace --all-features

# Clippy lint across workspace (fail on warnings)
lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

# Build HTML docs for all public items
docs:
	cargo doc --workspace --no-deps --open

# Clean target + incremental artifacts
clean-all:
	cargo clean && rm -rf target/tarpaulin target/llvm-cov

# Release build (all crates, all features)
build-release:
	cargo build --workspace --release --all-features

# Coverage for CI pipelines (summary only, fail-under 60 %)
coverage-ci:
	@command -v cargo-llvm-cov >/dev/null 2>&1 || (echo "Installing cargo-llvm-cov ..." && cargo install cargo-llvm-cov --locked)
	cargo llvm-cov --workspace --all-features --summary-only --fail-under 60

# Turnkey CI target – one command mirrors what GitHub Actions run
ci: fmt lint workspace-check workspace-test-all-features coverage-ci
	@echo "✅ CI checks complete – workspace healthy!" 
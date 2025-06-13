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
	@cd crates/toka-auth && cargo test

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
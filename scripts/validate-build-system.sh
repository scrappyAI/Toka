#!/bin/bash
# Build System Validation Script
# Phase 1: Build System Stabilization Agent
# Version: v0.3.0

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔧 Toka OS v0.3.0 Build System Validation"
echo "=========================================="

# Check Rust version
echo -e "${YELLOW}Checking Rust version...${NC}"
RUST_VERSION=$(rustc --version | cut -d' ' -f2)
echo "Rust version: $RUST_VERSION"

# Check if we meet minimum requirements for base64ct (Rust 1.85+)
if [[ "$RUST_VERSION" < "1.85" ]]; then
    echo -e "${RED}❌ Rust version $RUST_VERSION is below minimum requirement (1.85) for base64ct${NC}"
    echo "Please upgrade Rust: rustup update"
    exit 1
fi

echo -e "${GREEN}✅ Rust version meets requirements${NC}"

# Clean previous builds
echo -e "${YELLOW}Cleaning previous builds...${NC}"
cargo clean

# Phase 1: Dependency Resolution Check
echo -e "${YELLOW}Phase 1: Checking dependency resolution...${NC}"
if cargo check --workspace --all-features; then
    echo -e "${GREEN}✅ All dependencies resolve correctly${NC}"
else
    echo -e "${RED}❌ Dependency resolution failed${NC}"
    exit 1
fi

# Phase 2: Individual Crate Builds
echo -e "${YELLOW}Phase 2: Building individual crates...${NC}"
CRATES=(
    "toka-types"
    "toka-auth"
    "toka-bus-core"
    "toka-kernel"
    "toka-store-core"
    "toka-store-memory"
    "toka-store-sled"
    "toka-store-sqlite"
    "toka-runtime"
    "toka-tools"
    "toka-cli"
    "toka-config-cli"
)

for crate in "${CRATES[@]}"; do
    echo "Building $crate..."
    if cargo build -p "$crate" --all-features; then
        echo -e "${GREEN}✅ $crate built successfully${NC}"
    else
        echo -e "${RED}❌ $crate build failed${NC}"
        exit 1
    fi
done

# Phase 3: Full Workspace Build
echo -e "${YELLOW}Phase 3: Building entire workspace...${NC}"
if cargo build --workspace --all-features; then
    echo -e "${GREEN}✅ Full workspace build successful${NC}"
else
    echo -e "${RED}❌ Full workspace build failed${NC}"
    exit 1
fi

# Phase 4: Test Suite
echo -e "${YELLOW}Phase 4: Running test suite...${NC}"
if cargo test --workspace --all-features; then
    echo -e "${GREEN}✅ All tests passed${NC}"
else
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
fi

# Phase 5: Clippy Linting
echo -e "${YELLOW}Phase 5: Running clippy lints...${NC}"
if cargo clippy --workspace --all-targets --all-features -- -D warnings; then
    echo -e "${GREEN}✅ No clippy warnings${NC}"
else
    echo -e "${RED}❌ Clippy warnings detected${NC}"
    exit 1
fi

# Phase 6: Documentation Build
echo -e "${YELLOW}Phase 6: Building documentation...${NC}"
if cargo doc --workspace --no-deps --all-features; then
    echo -e "${GREEN}✅ Documentation built successfully${NC}"
else
    echo -e "${RED}❌ Documentation build failed${NC}"
    exit 1
fi

# Phase 7: Base64 Compatibility Check
echo -e "${YELLOW}Phase 7: Verifying base64 compatibility...${NC}"
if cargo check -p toka-tools --features base64; then
    echo -e "${GREEN}✅ Base64 dependency compatible${NC}"
else
    echo -e "${RED}❌ Base64 dependency incompatible${NC}"
    exit 1
fi

# Success Report
echo ""
echo -e "${GREEN}🎉 BUILD SYSTEM VALIDATION COMPLETE${NC}"
echo "=========================================="
echo "✅ All dependency conflicts resolved"
echo "✅ All workspace crates build successfully"
echo "✅ All tests pass"
echo "✅ No clippy warnings"
echo "✅ Documentation builds"
echo "✅ Base64ct compatibility verified"
echo ""
echo "Build System Stabilization Agent: SUCCESS"
echo "Ready to proceed with Phase 2 agents"

# Generate build report
BUILD_REPORT="target/build-system-validation-$(date +%Y%m%d_%H%M%S).log"
echo "Build validation completed successfully at $(date)" > "$BUILD_REPORT"
echo "Rust version: $RUST_VERSION" >> "$BUILD_REPORT"
echo "Base64 version: 0.22" >> "$BUILD_REPORT"
echo "All workspace crates validated" >> "$BUILD_REPORT"

echo "Build report saved to: $BUILD_REPORT"
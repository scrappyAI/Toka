#!/bin/bash
# EMERGENCE Migration Script
# Copies the minimal substrate to a new clean repository

set -e

EMERGENCE_REPO_PATH="${1:-../emergence}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🧬 EMERGENCE Migration Script"
echo "============================="
echo "Source workspace: $WORKSPACE_ROOT"
echo "Target repository: $EMERGENCE_REPO_PATH"
echo ""

# Validate target directory exists
if [ ! -d "$EMERGENCE_REPO_PATH" ]; then
    echo "❌ Target directory $EMERGENCE_REPO_PATH does not exist."
    echo "Please create the emergence repository first:"
    echo "   git clone https://github.com/YOUR_USERNAME/emergence.git"
    exit 1
fi

# Validate this is a git repository
if [ ! -d "$EMERGENCE_REPO_PATH/.git" ]; then
    echo "❌ Target directory is not a git repository."
    echo "Please ensure you've cloned the emergence repository correctly."
    exit 1
fi

echo "📋 Creating directory structure..."

# Create directory structure
mkdir -p "$EMERGENCE_REPO_PATH/crates"
mkdir -p "$EMERGENCE_REPO_PATH/.emergence/schemas"
mkdir -p "$EMERGENCE_REPO_PATH/docs/architecture"
mkdir -p "$EMERGENCE_REPO_PATH/examples"
mkdir -p "$EMERGENCE_REPO_PATH/scripts"

echo "📦 Copying core substrate..."

# Copy emergence crates
if [ -d "$WORKSPACE_ROOT/emergence-crates" ]; then
    cp -r "$WORKSPACE_ROOT/emergence-crates/emergence-physics" "$EMERGENCE_REPO_PATH/crates/"
    cp -r "$WORKSPACE_ROOT/emergence-crates/emergence-nervous-system" "$EMERGENCE_REPO_PATH/crates/"
    cp -r "$WORKSPACE_ROOT/emergence-crates/emergence-memory" "$EMERGENCE_REPO_PATH/crates/"
    cp -r "$WORKSPACE_ROOT/emergence-crates/emergence-runtime" "$EMERGENCE_REPO_PATH/crates/"
    echo "✅ Copied 4 core crates"
else
    echo "⚠️  emergence-crates directory not found"
fi

# Copy schema definitions
if [ -d "$WORKSPACE_ROOT/.emergence" ]; then
    cp -r "$WORKSPACE_ROOT/.emergence/"* "$EMERGENCE_REPO_PATH/.emergence/"
    echo "✅ Copied schema definitions"
else
    echo "⚠️  .emergence directory not found"
fi

# Copy documentation
echo "📚 Copying documentation..."
if [ -f "$WORKSPACE_ROOT/EMERGENCE_SUBSTRATE_RESEARCH.md" ]; then
    cp "$WORKSPACE_ROOT/EMERGENCE_SUBSTRATE_RESEARCH.md" "$EMERGENCE_REPO_PATH/docs/architecture/"
    echo "✅ Copied substrate research"
fi

if [ -f "$WORKSPACE_ROOT/EMERGENCE_IMPLEMENTATION_SUMMARY.md" ]; then
    cp "$WORKSPACE_ROOT/EMERGENCE_IMPLEMENTATION_SUMMARY.md" "$EMERGENCE_REPO_PATH/docs/"
    echo "✅ Copied implementation summary"
fi

if [ -f "$WORKSPACE_ROOT/ARCHITECTURE_RESEARCH.md" ]; then
    cp "$WORKSPACE_ROOT/ARCHITECTURE_RESEARCH.md" "$EMERGENCE_REPO_PATH/docs/architecture/legacy-analysis.md"
    echo "✅ Copied legacy analysis"
fi

if [ -f "$WORKSPACE_ROOT/EMERGENCE_MIGRATION_GUIDE.md" ]; then
    cp "$WORKSPACE_ROOT/EMERGENCE_MIGRATION_GUIDE.md" "$EMERGENCE_REPO_PATH/docs/"
    echo "✅ Copied migration guide"
fi

# Create clean Cargo.toml
echo "⚙️  Creating clean workspace configuration..."
cat > "$EMERGENCE_REPO_PATH/Cargo.toml" << 'EOF'
[workspace]
resolver = "2"
members = [
    "crates/emergence-physics",
    "crates/emergence-nervous-system", 
    "crates/emergence-memory",
    "crates/emergence-runtime",
]

[workspace.dependencies]
# Core async runtime
tokio = { version = "1.36", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1.0"

# Async traits
async-trait = "0.1"
futures = "0.3"

# Time and IDs
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.7", features = ["v4", "serde"] }

# Cryptography
blake3 = "1.5"

# Numerical computation
ordered-float = "4.2"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Schema validation
jsonschema = "0.17"

# Development dependencies
[workspace.dependencies.tokio-test]
version = "0.4"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"
description = "Living agent substrate for conscious human-AI collaboration"
EOF

# Create README.md if it doesn't exist
if [ ! -f "$EMERGENCE_REPO_PATH/README.md" ]; then
    echo "📖 Creating README.md..."
    cat > "$EMERGENCE_REPO_PATH/README.md" << 'EOF'
# 🧬 EMERGENCE
*Living Agent Substrate for Conscious Human-AI Collaboration*

## Vision

EMERGENCE transforms AI from static automation to conscious collaboration. Instead of spawning processes, you awaken living entities. Instead of executing hardcoded behaviors, intelligence emerges from simple rules and personality-driven interactions.

```bash
# Traditional approach
$ ai-system spawn --config agent.toml

# EMERGENCE approach  
🧬 > awaken researcher with curiosity=0.9 persistence=0.8
🧬 Awakening researcher essence...
💭 researcher-f47ac10b: "I sense fascinating patterns waiting to be discovered..."
⚡ Capabilities emerging: [pattern-recognition, analysis, synthesis]
```

## Architecture

**Minimal Rust Substrate** (4 crates):
- **🔬 emergence-physics**: Immutable physics laws (energy, causality, security)
- **🌐 emergence-nervous-system**: Event-driven communication patterns
- **🧠 emergence-memory**: Multi-layered memory substrate  
- **⚡ emergence-runtime**: Schema-driven behavior composition

**Schema-Driven Intelligence**:
- Living agent essences defined in YAML schemas
- Personality traits drive behavioral patterns
- Capabilities evolve through experience
- Emergent behaviors from primitive operations

## Quick Start

```bash
# Install Rust and clone
git clone https://github.com/YOUR_USERNAME/emergence.git
cd emergence

# Run the living agent terminal
cargo run --bin emergence-terminal

# Awaken your first entity
🧬 > awaken researcher with curiosity=0.9
🧬 > researcher, analyze this codebase
```

## Status

🟢 **Foundation Complete**: Minimal substrate implemented  
🔄 **In Development**: Nervous system, memory, runtime composition  
📋 **Next**: Advanced agent essences and emergent behaviors

See [docs/EMERGENCE_IMPLEMENTATION_SUMMARY.md](docs/EMERGENCE_IMPLEMENTATION_SUMMARY.md) for detailed progress.

## The Promise

This is the first system where:
- **Agents are born, not spawned**
- **Behaviors emerge, not execute**  
- **Intelligence grows, not processes**
- **Collaboration feels natural, not orchestrated**

*The future is not programmed—it emerges.*
EOF
fi

# Update crate paths in Cargo.toml files
echo "🔧 Updating crate paths..."
find "$EMERGENCE_REPO_PATH/crates" -name "Cargo.toml" -exec sed -i.bak 's|path = "../emergence-|path = "../|g' {} \;
find "$EMERGENCE_REPO_PATH/crates" -name "*.bak" -delete

# Create .gitignore if it doesn't exist
if [ ! -f "$EMERGENCE_REPO_PATH/.gitignore" ]; then
    echo "📝 Creating .gitignore..."
    cat > "$EMERGENCE_REPO_PATH/.gitignore" << 'EOF'
# Rust
/target/
Cargo.lock
**/*.rs.bk
*.pdb

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Logs
*.log

# Environment
.env
.env.local

# Emergence specific
/emergence-instances/
/agent-logs/
*.emergence-session
EOF
fi

echo ""
echo "🎉 Migration complete!"
echo ""
echo "Next steps:"
echo "1. cd $EMERGENCE_REPO_PATH"
echo "2. git add ."
echo "3. git commit -m \"feat: initial EMERGENCE substrate\""
echo "4. git push origin main"
echo "5. cargo run --bin emergence-terminal"
echo ""
echo "🧬 Welcome to the EMERGENCE workspace!"
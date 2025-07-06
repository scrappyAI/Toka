# Toka Dependency Graph Visualization System

A comprehensive dependency analysis and visualization tool for the Toka modular agent runtime, built using agent spec schemas and parallel processing to create accurate dependency graphs that match the actual structure of the system.

## Generated Visualizations

This system has analyzed the Toka codebase and generated the following visualizations:

### 📊 System-Wide Dependency Graph
- **File**: `dependency_graphs/system_dependency_graph.(png|svg)`
- **Description**: Complete overview of all 23 crates in the workspace showing their internal dependencies
- **Features**: Color-coded by category (core, storage, runtime, etc.) with legend

### 🎯 Individual Crate Dependency Graphs
- **Location**: `dependency_graphs/individual_crates/`
- **Count**: 20+ individual graphs
- **Description**: Detailed dependency view for each crate showing:
  - Workspace dependencies (blue arrows)
  - Important external dependencies (gray dashed arrows)
  - Category-based color coding

### 🤖 Agent Composition Graph
- **File**: `dependency_graphs/agent_composition_graph.(png|svg)`
- **Description**: Visualization of the 4 configured agents based on agent specifications:
  - **code-analyst** (code-analysis domain)
  - **test-orchestrator** (testing domain) 
  - **security-auditor** (security domain)
  - **performance-optimizer** (performance domain)

### 🏗️ Layered Architecture Graph
- **File**: `dependency_graphs/layered_architecture.(png|svg)`
- **Description**: Architectural layers showing:
  - **Applications Layer**: CLI tools
  - **Agent Layer**: Runtime and orchestration
  - **Runtime Layer**: Core runtime and LLM gateway
  - **Core Layer**: Kernel, bus, types, auth
  - **Storage Layer**: Various storage implementations
  - **Consensus Layer**: Raft implementation
  - **Security Layer**: Capability management
  - **Utilities Layer**: Tools and performance monitoring

### 📋 Analysis Report
- **File**: `dependency_graphs/dependency_analysis_report.md`
- **Content**: Comprehensive analysis including:
  - 23 crates analyzed
  - 4 agent specifications
  - 23 internal dependencies
  - Breakdown by categories and detailed crate information

## Analysis Results

### Crate Categories
- **Core**: 3 crates (kernel, bus-core, capability-core)
- **Storage**: 6 crates (memory, sled, sqlite, semantic, core)
- **Runtime**: 2 crates (runtime, agent-runtime)
- **Tools**: 3 crates (cli, config-cli, tools)
- **Security**: 7 crates (various capability implementations)
- **LLM**: 1 crate (llm-gateway)
- **Orchestration**: 1 crate (orchestration)

### Key Dependencies
The system identified critical dependency relationships:
- **toka-runtime** has the most dependencies (8 workspace deps)
- **toka-orchestration** connects multiple layers (6 workspace deps)
- **toka-agent-runtime** integrates with core systems (5 workspace deps)

## Usage

### Running the Visualizer

```bash
# Activate virtual environment
source venv/bin/activate

# Generate all graphs
python dependency_graph_visualizer.py --all

# Generate specific graph types
python dependency_graph_visualizer.py --individual    # Individual crate graphs
python dependency_graph_visualizer.py --agents        # Agent composition
python dependency_graph_visualizer.py --layered       # Layered architecture

# Custom output directory
python dependency_graph_visualizer.py --all --output-dir custom_graphs
```

### Command Line Options

- `--workspace PATH`: Specify workspace path (default: current directory)
- `--output-dir DIR`: Output directory for graphs (default: dependency_graphs)
- `--individual`: Generate individual crate dependency graphs
- `--agents`: Generate agent composition graph
- `--layered`: Generate layered architecture graph
- `--all`: Generate all graph types

## Features

### Parallel Processing
- **Multi-threaded Analysis**: Uses ThreadPoolExecutor with 8 workers for efficient crate analysis
- **Async I/O**: Utilizes aiofiles for non-blocking file operations
- **Concurrent Graph Generation**: Generates multiple visualizations in parallel

### Agent Spec Integration
- **Schema Compliance**: Based on agent-spec-schema.yaml v1.0.0
- **Agent Configuration**: Reads from config/agents.toml
- **Domain Classification**: Categorizes agents by domain (testing, security, performance, etc.)
- **Priority Visualization**: Shows agent priorities visually

### Intelligent Categorization
Automatically categorizes crates based on:
- Name patterns (storage, auth, security, runtime, etc.)
- Description analysis
- Dependency relationships
- Domain-specific patterns

### Output Formats
- **PNG**: High-resolution raster images for viewing/sharing
- **SVG**: Vector graphics for embedding in documentation
- **Markdown**: Comprehensive analysis reports

## Architecture Insights

### Layer Dependencies
The visualization reveals the clean layered architecture:
1. **Applications** depend on **Core** and **Tools**
2. **Agent Layer** orchestrates **Runtime** components
3. **Storage Layer** provides multiple backend options
4. **Security Layer** integrates across all levels

### Dependency Health
- Most crates have focused, minimal dependencies
- Clear separation of concerns across layers
- No circular dependencies detected
- External dependencies are well-managed

## System Requirements

### Dependencies
```
graphviz>=0.20.1      # Graph visualization
toml>=0.10.2          # TOML configuration parsing
aiofiles>=23.2.1      # Async file operations
PyYAML>=6.0.1         # YAML schema parsing
```

### System Packages
```bash
# Ubuntu/Debian
sudo apt install python3-pip python3-venv python3-graphviz graphviz

# Or install via package manager
pip install -r requirements.txt
```

## Files Generated

### Main Outputs
- `system_dependency_graph.png/.svg` - Complete system overview
- `agent_composition_graph.png/.svg` - Agent relationships
- `layered_architecture.png/.svg` - Architectural layers
- `dependency_analysis_report.md` - Detailed analysis

### Individual Crate Graphs
Each crate gets its own visualization showing:
- Direct workspace dependencies
- Key external dependencies
- Category-based styling
- Version information

## Technical Implementation

### Core Components

1. **DependencyAnalyzer**: Analyzes Cargo.toml files and agent configurations
2. **GraphVisualizer**: Generates visual representations using Graphviz
3. **Parallel Processing**: Efficient analysis using concurrent futures
4. **Schema Integration**: Validates against agent specification schema

### Graph Types

1. **System Graph**: Overall dependency network
2. **Individual Graphs**: Per-crate dependency views
3. **Agent Graph**: Agent composition and relationships
4. **Architecture Graph**: Layered system view

### Customization

The visualizer supports:
- Custom color schemes per category
- Flexible output formats
- Configurable graph layouts
- Extensible categorization rules

## Next Steps

This visualization system provides the foundation for:
- **Dependency Management**: Track and optimize dependencies
- **Architecture Reviews**: Assess system design and evolution
- **Agent Composition**: Plan and validate agent configurations
- **Documentation**: Auto-generate architectural documentation
- **CI/CD Integration**: Automated dependency analysis

The graphs accurately reflect the current structure and can be regenerated as the codebase evolves, providing ongoing architectural insights and dependency tracking.
# Toka Control Flow Graph Visualizer

A comprehensive control flow analysis tool for the Toka codebase that creates detailed visualizations of execution patterns, async flows, and system-wide control flow interactions.

## Overview

The Control Flow Graph Visualizer complements the existing dependency graph tool by focusing on **execution flow** rather than static dependencies. It provides insights into:

- **Function-level control flow**: How code executes within individual functions
- **System-level execution patterns**: How components coordinate and interact during runtime
- **Async flow analysis**: Understanding complex async/await patterns and coordination
- **Error handling flows**: Tracking error propagation and handling patterns
- **State machine visualizations**: Identifying and visualizing state transitions
- **Complexity analysis**: Measuring and visualizing code complexity metrics

## Features

### 🔍 Function-Level Analysis
- **Control Flow Graphs (CFGs)** for individual functions
- **Rust-specific patterns**: async/await, Result handling, state machines
- **Complexity metrics**: Cyclomatic complexity, async complexity, error handling complexity
- **Node classification**: Entry/exit points, conditions, loops, async points, error handlers

### 🏗️ System-Level Analysis
- **Component interaction flows** showing runtime coordination
- **Async coordination patterns** across the system
- **Error propagation chains** through multiple components
- **Orchestration sequence analysis** for complex workflows

### 📊 Visualization Types
1. **Individual Function CFGs**: Detailed flow within specific functions
2. **System Flow Graphs**: High-level component interactions
3. **Async Coordination Graphs**: Async pattern relationships
4. **Complexity Heatmaps**: Visual complexity analysis across the codebase

### 🎯 Smart Pattern Recognition
- **Async patterns**: spawn_and_await, fire_and_forget, sequential_async
- **Error patterns**: Result handling, error propagation, unwrap chains
- **State patterns**: State machine transitions, state-based logic
- **Concurrency patterns**: Arc/Mutex usage, channel communication

## Installation & Setup

### Prerequisites
```bash
# Install Python dependencies
pip install graphviz aiofiles

# Install Graphviz system package
# Ubuntu/Debian:
sudo apt-get install graphviz

# macOS:
brew install graphviz

# Windows:
# Download from https://graphviz.org/download/
```

### Make the tool executable
```bash
chmod +x control_flow_graph_visualizer.py
```

## Usage

### Basic Usage

```bash
# Analyze entire workspace and generate all visualizations
./control_flow_graph_visualizer.py --all

# Generate system-wide control flow
./control_flow_graph_visualizer.py --system

# Analyze async coordination patterns
./control_flow_graph_visualizer.py --async

# Generate complexity heatmap
./control_flow_graph_visualizer.py --complexity
```

### Function-Specific Analysis

```bash
# Generate CFG for a specific function
./control_flow_graph_visualizer.py --function "submit"

# Analyze the kernel's main execution flow
./control_flow_graph_visualizer.py --function "handle_schedule_task"

# Examine orchestration coordination
./control_flow_graph_visualizer.py --function "run_orchestration"
```

### Advanced Options

```bash
# Specify custom workspace and output directory
./control_flow_graph_visualizer.py --workspace /path/to/toka --output-dir /path/to/output

# Generate specific visualization types
./control_flow_graph_visualizer.py --system --async --complexity
```

## Output Files

The tool generates several types of output files:

### Visualization Files
- `system_control_flow.{png,svg}` - System-wide component interactions
- `async_coordination.{png,svg}` - Async pattern visualizations  
- `complexity_heatmap.{png,svg}` - Function complexity analysis
- `cfg_<function_name>.{png,svg}` - Individual function control flow graphs

### Reports
- `control_flow_analysis_report.md` - Comprehensive analysis summary with:
  - Complexity statistics
  - Most complex functions
  - Async pattern distribution
  - Component interaction summary

## Understanding the Visualizations

### Node Types & Colors

| Node Type | Color | Description |
|-----------|--------|-------------|
| Entry | Green | Function entry point |
| Exit | Red | Function exit point |
| Statement | Light Blue | Regular code statements |
| Condition | Yellow | if/match conditional statements |
| Loop | Light Orange | for/while/loop constructs |
| Async Point | Light Purple | General async operations |
| Await Point | Medium Purple | Specific .await points |
| Spawn Point | Dark Purple | tokio::spawn/thread spawn |
| Error Handler | Light Red | Error handling (?, unwrap, etc.) |
| State Transition | Light Green | State machine transitions |
| Function Call | Light Blue | Function/method calls |
| Return Point | Light Orange | Return statements |

### Edge Types & Colors

| Edge Type | Color | Style | Description |
|-----------|--------|-------|-------------|
| Control | Black | Solid | Normal control flow |
| Async | Purple | Dashed | Async operations |
| Error | Red | Solid | Error handling/propagation |
| Data | Blue | Solid | Data flow dependencies |

### Complexity Metrics

The tool computes several complexity metrics:

- **Cyclomatic Complexity**: Traditional complexity based on decision points
- **Async Complexity**: Number of async points (await/spawn)
- **Error Handling Complexity**: Number of error handling constructs
- **Total Nodes/Edges**: Graph size metrics

## Integration with Dependency Graph Tool

The control flow graph tool is designed to complement the existing dependency graph visualizer:

### Dependency Graph Shows:
- Static relationships between crates
- Import/export dependencies
- Module structure
- Build-time relationships

### Control Flow Graph Shows:
- Runtime execution patterns
- Dynamic interactions
- Async coordination
- Error flow patterns
- Execution complexity

### Combined Analysis Workflow:
1. **Start with dependency graphs** to understand static structure
2. **Use control flow graphs** to understand runtime behavior
3. **Cross-reference** to identify architectural patterns
4. **Optimize** based on complexity and flow analysis

## Key Use Cases

### 🔧 Development & Debugging
- **Understanding complex async flows** in the orchestration system
- **Tracing error propagation** through multiple components
- **Identifying bottlenecks** in execution patterns
- **Debugging state machine logic** in the kernel

### 📈 Code Quality & Maintenance  
- **Complexity analysis** to identify refactoring candidates
- **Pattern identification** for consistency across the codebase
- **Architecture validation** to ensure proper separation of concerns
- **Performance optimization** by understanding execution hotspots

### 🎓 Onboarding & Documentation
- **Visual learning** of system execution patterns
- **Understanding async coordination** in distributed systems
- **Grasping error handling strategies** across components
- **Learning architectural patterns** through visualization

## Example Analysis Scenarios

### Scenario 1: Analyzing Kernel Execution Flow
```bash
# Generate kernel function CFGs
./control_flow_graph_visualizer.py --function "submit"
./control_flow_graph_visualizer.py --function "handle_schedule_task" 
./control_flow_graph_visualizer.py --function "handle_spawn_agent"

# View kernel complexity
./control_flow_graph_visualizer.py --complexity
```

### Scenario 2: Understanding Orchestration Patterns
```bash
# Analyze orchestration coordination
./control_flow_graph_visualizer.py --function "run_orchestration"
./control_flow_graph_visualizer.py --async

# View system-wide interactions
./control_flow_graph_visualizer.py --system
```

### Scenario 3: Error Handling Analysis
```bash
# Generate all graphs to see error patterns
./control_flow_graph_visualizer.py --all

# Focus on functions with high error complexity
# (Check the complexity heatmap for candidates)
```

## Advanced Features

### Custom Pattern Recognition
The tool recognizes Toka-specific patterns:
- **Agent lifecycle management** in the runtime
- **Message passing patterns** in the bus system
- **State transitions** in kernel state machines
- **Orchestration sequences** for parallel agent spawning

### Smart Filtering
- **Component-based filtering** to focus on specific subsystems
- **Complexity thresholds** to highlight problem areas
- **Pattern-based filtering** to analyze specific async patterns

### Performance Optimizations
- **Parallel analysis** of multiple files
- **Incremental updates** for large codebases
- **Memory-efficient** graph construction
- **Fast pattern matching** with compiled regexes

## Troubleshooting

### Common Issues

**Graphviz not found:**
```bash
# Install Graphviz and ensure it's in PATH
which dot  # Should return a path
```

**Large graphs:**
```bash
# Use specific function analysis instead of full system
./control_flow_graph_visualizer.py --function "specific_function"
```

**Memory issues:**
```bash
# Reduce parallel processing
# Edit the ThreadPoolExecutor max_workers parameter in the code
```

### Performance Tips

1. **Start with specific functions** before generating full system graphs
2. **Use the report first** to identify interesting functions
3. **Filter by complexity** to focus on problem areas
4. **Generate SVG files** for better scalability in browsers

## Contributing

### Adding New Pattern Recognition
1. Extend `RustPatternAnalyzer` with new regex patterns
2. Add corresponding node types to `FlowNodeType`
3. Update visualization colors and styles
4. Test with representative code samples

### Adding New Visualization Types
1. Create new methods in `ControlFlowVisualizer`
2. Add command-line options in `main()`
3. Update documentation and examples
4. Ensure proper error handling

## Future Enhancements

- **Interactive web interface** for exploring graphs
- **Real-time analysis** during development
- **Integration with profiling data** for execution frequency
- **Machine learning** for pattern classification
- **Performance correlation** with actual runtime metrics
- **Integration with IDE** for inline flow visualization

## Related Tools

- **Dependency Graph Visualizer**: `dependency_graph_visualizer.py`
- **Raft Analysis**: `raft_analysis.py`
- **Performance Monitoring**: Various monitoring scripts
- **Test Analysis**: Integration and property-based test tools

---

**Generated by Toka Control Flow Graph Visualizer v1.0**  
*Bringing clarity to complex execution patterns* 
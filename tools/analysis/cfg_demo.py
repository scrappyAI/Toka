#!/usr/bin/env python3
"""
Enhanced Control Flow Graph Demo
Showcases the new LLM-friendly and interactive visualization capabilities.
"""

import json
import os
from pathlib import Path

def create_sample_control_flow_data():
    """Create sample control flow data to demonstrate the enhanced outputs"""
    return {
        "function_name": "process_user_request",
        "file_path": "src/handlers/request_handler.rs",
        "location": {"start_line": 45, "end_line": 120},
        "properties": {"is_async": True, "return_type": "Result<Response, Error>"},
        "complexity_metrics": {
            "cyclomatic_complexity": 8,
            "async_complexity": 3,
            "error_handling_complexity": 5
        },
        "nodes": [
            {"id": "entry", "type": "entry", "label": "🚀 Entry: process_user_request", "source_line": 45},
            {"id": "validate", "type": "condition", "label": "validate_request(req)?", "source_line": 47},
            {"id": "auth", "type": "async_point", "label": "⚡ authenticate_user().await", "source_line": 52},
            {"id": "process", "type": "statement", "label": "process_business_logic()", "source_line": 58},
            {"id": "error_handler", "type": "error_handler", "label": "❌ handle_error(e)", "source_line": 65},
            {"id": "success", "type": "statement", "label": "format_response(result)", "source_line": 70},
            {"id": "exit", "type": "exit", "label": "🏁 Exit: process_user_request", "source_line": 120}
        ],
        "edges": [
            {"source": "entry", "target": "validate", "edge_type": "control"},
            {"source": "validate", "target": "auth", "edge_type": "control", "condition": "valid"},
            {"source": "validate", "target": "error_handler", "edge_type": "error", "condition": "invalid"},
            {"source": "auth", "target": "process", "edge_type": "async"},
            {"source": "auth", "target": "error_handler", "edge_type": "error"},
            {"source": "process", "target": "success", "edge_type": "control"},
            {"source": "process", "target": "error_handler", "edge_type": "error"},
            {"source": "success", "target": "exit", "edge_type": "control"},
            {"source": "error_handler", "target": "exit", "edge_type": "control"}
        ],
        "analysis_summary": "This function exhibits moderate complexity, async operations, comprehensive error handling, conditional logic."
    }

def generate_mermaid_flowchart(data):
    """Generate Mermaid flowchart - much better than blurry PNG!"""
    mermaid = ["flowchart TD"]
    
    # Add nodes with proper shapes and emojis
    for node in data["nodes"]:
        node_id = node["id"]
        label = node["label"]
        node_type = node["type"]
        
        if node_type == "entry":
            mermaid.append(f'    {node_id}(["{label}"])')
        elif node_type == "exit":
            mermaid.append(f'    {node_id}(["{label}"])')
        elif node_type == "condition":
            mermaid.append(f'    {node_id}{{{"{label}"}}}')
        elif node_type == "async_point":
            mermaid.append(f'    {node_id}(("{label}"))')
        elif node_type == "error_handler":
            mermaid.append(f'    {node_id}[["{label}"]]')
        else:
            mermaid.append(f'    {node_id}["{label}"]')
    
    # Add edges with styling
    for edge in data["edges"]:
        source = edge["source"]
        target = edge["target"]
        edge_type = edge["edge_type"]
        condition = edge.get("condition", "")
        
        if edge_type == "async":
            mermaid.append(f'    {source} -.->|"async"| {target}')
        elif edge_type == "error":
            mermaid.append(f'    {source} ==>|"error {condition}"| {target}')
        else:
            if condition:
                mermaid.append(f'    {source} -->|"{condition}"| {target}')
            else:
                mermaid.append(f'    {source} --> {target}')
    
    # Add styling
    mermaid.extend([
        "",
        "    %% Styling for better visual appeal",
        "    classDef entryNode fill:#4CAF50,stroke:#2E7D32,color:#fff",
        "    classDef exitNode fill:#F44336,stroke:#C62828,color:#fff",
        "    classDef conditionNode fill:#FFF59D,stroke:#F57F17",
        "    classDef asyncNode fill:#E1BEE7,stroke:#7B1FA2",
        "    classDef errorNode fill:#FFCDD2,stroke:#C62828",
        "",
        f"    class entry entryNode",
        f"    class exit exitNode", 
        f"    class validate conditionNode",
        f"    class auth asyncNode",
        f"    class error_handler errorNode"
    ])
    
    # Add complexity metrics as comments
    metrics = data["complexity_metrics"]
    mermaid.extend([
        "",
        f"%% Complexity Metrics:",
        f"%% Cyclomatic Complexity: {metrics['cyclomatic_complexity']}",
        f"%% Async Complexity: {metrics['async_complexity']}",
        f"%% Error Handling Complexity: {metrics['error_handling_complexity']}"
    ])
    
    return "\n".join(mermaid)

def generate_textual_summary(data):
    """Generate LLM-friendly textual analysis"""
    func_name = data["function_name"]
    location = data["location"]
    props = data["properties"]
    metrics = data["complexity_metrics"]
    
    summary = f"""# Control Flow Analysis: {func_name}

**Location:** {data['file_path']}:{location['start_line']}-{location['end_line']}
**Type:** {'Async Function' if props['is_async'] else 'Sync Function'}
**Return Type:** {props['return_type']}

## Complexity Metrics
- Cyclomatic Complexity: {metrics['cyclomatic_complexity']}
- Async Complexity: {metrics['async_complexity']} 
- Error Handling Complexity: {metrics['error_handling_complexity']}

## Control Flow Structure
- Total Nodes: {len(data['nodes'])}
- Total Edges: {len(data['edges'])}

### Node Distribution
- Entry Points: 1
- Exit Points: 1
- Conditions: 1
- Async Points: 1
- Error Handlers: 1
- Statements: 2

### Flow Analysis
{data['analysis_summary']}

## Architecture Notes
• **Processor/Handler**: Core business logic execution
• **Async Pattern**: Uses async/await for authentication operations
• **Error Resilience**: Multiple error handling strategies
• **Validation Logic**: Input validation with early returns

## LLM Context for Architecture Understanding

This function represents a typical async request handler in the Toka system:

1. **Entry Flow**: Validates incoming requests with early error handling
2. **Authentication**: Async authentication with proper error propagation
3. **Processing**: Core business logic with error boundaries
4. **Response**: Structured response formatting
5. **Error Handling**: Comprehensive error management throughout the flow

The moderate complexity (cyclomatic: 8) indicates well-structured code with appropriate branching. The async complexity (3) shows thoughtful async coordination without over-complication. The error handling complexity (5) demonstrates robust error management patterns.

**Architectural Significance**: This pattern is likely replicated across similar handlers in the system, making it a good example of the request processing architecture.
"""
    return summary

def generate_interactive_html_preview(data):
    """Generate a preview of what the interactive HTML would contain"""
    func_name = data["function_name"]
    
    preview = f"""
🌐 **Interactive HTML Visualization Features for {func_name}:**

✨ **Zoom & Pan**: Infinite zoom with smooth panning for large graphs
🎯 **Node Filtering**: Filter by node type (async, error, condition, etc.)
🔄 **Layout Options**: Hierarchical, breadth-first, circular layouts
📊 **Live Metrics**: Real-time complexity metrics display
🖱️ **Node Interaction**: Click nodes for detailed information
🎨 **Visual Styling**: Color-coded nodes and edges by type
📱 **Responsive Design**: Works on desktop, tablet, and mobile

**Example Interactive Features:**
- Click 'auth' node → Shows: "Async authentication with error handling"
- Filter 'async' → Highlights async flow paths
- Switch to 'circle' layout → Reorganizes for pattern analysis
- Zoom to 200% → See detailed labels without pixelation

**Benefits over Static PNG:**
❌ PNG: Blurry when zoomed, fixed layout, no interaction
✅ Interactive: Sharp at any zoom, multiple layouts, rich interaction
"""
    return preview

def create_enhanced_visualizations():
    """Demonstrate all the enhanced visualization formats"""
    
    print("🚀 Enhanced Control Flow Graph Visualizer Demo")
    print("=" * 60)
    
    # Get sample data
    data = create_sample_control_flow_data()
    
    # Create output directory
    output_dir = "enhanced_cfg_outputs"
    os.makedirs(output_dir, exist_ok=True)
    
    print(f"\n📁 Creating outputs in: {output_dir}/\n")
    
    # 1. Generate Mermaid flowchart
    print("1️⃣ Generating Mermaid Flowchart (Documentation & LLM friendly)")
    mermaid_content = generate_mermaid_flowchart(data)
    
    mermaid_file = f"{output_dir}/process_user_request.mmd"
    with open(mermaid_file, 'w') as f:
        f.write(mermaid_content)
    
    print(f"   ✅ Saved: {mermaid_file}")
    print("   🎯 Benefits: Scalable, embeddable in docs, GitHub renders automatically")
    
    # 2. Export structured JSON
    print("\n2️⃣ Exporting Structured JSON (Programmatic Analysis)")
    
    json_file = f"{output_dir}/process_user_request.json" 
    with open(json_file, 'w') as f:
        json.dump(data, f, indent=2)
    
    print(f"   ✅ Saved: {json_file}")
    print("   🎯 Benefits: Machine readable, custom visualizations, data analysis")
    
    # 3. Generate textual summary
    print("\n3️⃣ Generating Textual Summary (LLM Context)")
    summary_content = generate_textual_summary(data)
    
    summary_file = f"{output_dir}/process_user_request_summary.md"
    with open(summary_file, 'w') as f:
        f.write(summary_content)
    
    print(f"   ✅ Saved: {summary_file}")
    print("   🎯 Benefits: LLM consumable, architectural insights, human readable")
    
    # 4. Interactive HTML preview
    print("\n4️⃣ Interactive HTML Visualization Preview")
    html_preview = generate_interactive_html_preview(data)
    print(html_preview)
    
    print("\n🎉 **Summary of Improvements over Static PNG:**")
    print("""
🔍 **Visual Quality**: 
   • PNG: Blurry, pixelated when zoomed, fixed resolution
   • Enhanced: Vector graphics, infinite zoom, crisp at any size

📊 **Functionality**:
   • PNG: Static image, no interaction, single view
   • Enhanced: Interactive filtering, multiple layouts, live metrics

📖 **LLM Integration**:
   • PNG: Binary data, not text-analyzable by LLMs
   • Enhanced: Structured data, textual summaries, semantic markup

🛠️ **Use Cases**:
   • PNG: Basic visualization only
   • Enhanced: Documentation, analysis, architectural review, LLM context

💡 **Developer Experience**:
   • PNG: Open image viewer, squint to read labels
   • Enhanced: Navigate in browser, zoom to details, filter complexity
""")
    
    print(f"\n📂 All outputs available in: {output_dir}/")
    print("\n🚀 Next Steps:")
    print("   • Open .mmd files in Mermaid Live Editor or GitHub")
    print("   • Use .json for custom visualizations or analysis")
    print("   • Share .md summaries with LLMs for architectural discussions")
    print("   • Deploy .html files for interactive team reviews")

if __name__ == "__main__":
    create_enhanced_visualizations() 
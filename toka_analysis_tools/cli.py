#!/usr/bin/env python3
"""
CLI entry point for Toka Analysis Tools

This provides a command-line interface for running the analysis tools
without requiring MCP integration.
"""

import asyncio
import argparse
import sys
import logging
from pathlib import Path
from typing import List, Optional

from .config import AnalysisConfig
from .tool_registry import register_toka_tools
from .control_flow import ControlFlowAnalyzer, ControlFlowVisualizer
from .dependency_graph import DependencyAnalyzer, DependencyVisualizer

logger = logging.getLogger(__name__)

def setup_logging(level: str = "INFO"):
    """Setup logging configuration"""
    logging.basicConfig(
        level=getattr(logging, level.upper()),
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )

async def control_flow_command(args: argparse.Namespace) -> None:
    """Execute control flow analysis command"""
    config = AnalysisConfig()
    config.workspace_path = args.workspace
    config.output_dir = args.output_dir
    
    # Create output directories
    config.create_output_dirs()
    
    # Initialize analyzer
    analyzer = ControlFlowAnalyzer(config.workspace_path, config)
    await analyzer.analyze_workspace()
    
    # Initialize visualizer
    visualizer = ControlFlowVisualizer(analyzer)
    
    output_base = Path(config.output_dir) / "control_flow"
    
    if args.function:
        # Analyze specific function
        if args.function not in analyzer.function_flows:
            print(f"❌ Function '{args.function}' not found")
            sys.exit(1)
        
        output_path = output_base / args.function
        
        print(f"🔍 Analyzing function: {args.function}")
        
        if not args.formats or "mermaid" in args.formats:
            mermaid_content = visualizer.generate_mermaid_flowchart(args.function, str(output_path))
            print(f"✅ Mermaid flowchart: {output_path}.mmd")
        
        if not args.formats or "json" in args.formats:
            json_data = visualizer.export_function_json(args.function, str(output_path))
            print(f"✅ JSON export: {output_path}.json")
        
        if not args.formats or "summary" in args.formats:
            summary = visualizer.generate_textual_summary(args.function, str(output_path))
            print(f"✅ Summary: {output_path}_summary.md")
        
        if not args.formats or "interactive" in args.formats:
            html_content = visualizer.generate_interactive_html(args.function, str(output_path))
            print(f"✅ Interactive: {output_path}_interactive.html")
    
    else:
        # System-wide analysis
        print(f"🔍 Analyzing {len(analyzer.function_flows)} functions across workspace")
        
        if not args.formats or "system" in args.formats:
            visualizer.generate_system_flow_graph(str(output_base / "system_flow"))
            print(f"✅ System flow: {output_base}/system_flow")
        
        if not args.formats or "complexity" in args.formats:
            visualizer.generate_complexity_heatmap(str(output_base / "complexity_heatmap"))
            print(f"✅ Complexity heatmap: {output_base}/complexity_heatmap")

async def dependency_command(args: argparse.Namespace) -> None:
    """Execute dependency analysis command"""
    config = AnalysisConfig()
    config.workspace_path = args.workspace
    config.output_dir = args.output_dir
    
    # Create output directories
    config.create_output_dirs()
    
    # Initialize analyzer
    analyzer = DependencyAnalyzer(config.workspace_path, config)
    await analyzer.analyze_workspace()
    
    # Initialize visualizer
    visualizer = DependencyVisualizer(analyzer)
    
    output_base = Path(config.output_dir) / "dependency_graphs"
    
    print(f"🔍 Analyzing {len(analyzer.crates)} crates and {len(analyzer.agent_specs)} agents")
    
    if not args.formats or "mermaid" in args.formats:
        mermaid_content = visualizer.generate_mermaid_graph(str(output_base / "dependencies"))
        print(f"✅ Mermaid graph: {output_base}/dependencies.mmd")
    
    if not args.formats or "json" in args.formats:
        json_data = visualizer.export_json(str(output_base / "dependencies"))
        print(f"✅ JSON export: {output_base}/dependencies.json")
    
    if not args.formats or "summary" in args.formats:
        summary = visualizer.generate_textual_summary(str(output_base / "dependencies"))
        print(f"✅ Summary: {output_base}/dependencies_summary.md")
    
    if not args.formats or "interactive" in args.formats:
        html_content = visualizer.generate_interactive_html(str(output_base / "dependencies"))
        print(f"✅ Interactive: {output_base}/dependencies_interactive.html")
    
    if not args.formats or "system" in args.formats:
        result = visualizer.generate_system_graph(str(output_base / "system_dependencies"))
        print(f"✅ System graph: {result}")

async def combined_command(args: argparse.Namespace) -> None:
    """Execute combined analysis command"""
    config = AnalysisConfig()
    config.workspace_path = args.workspace
    config.output_dir = args.output_dir
    
    print("🚀 Running combined analysis...")
    
    # Run control flow analysis
    args.formats = ["mermaid", "json", "system"]
    await control_flow_command(args)
    
    # Run dependency analysis  
    args.formats = ["mermaid", "json", "summary", "interactive"]
    await dependency_command(args)
    
    print("✅ Combined analysis complete!")

async def mcp_server_command(args: argparse.Namespace) -> None:
    """Run MCP server command"""
    try:
        from .mcp_server import TokaAnalysisServer
    except ImportError:
        print("❌ MCP server dependencies not available. Install with: pip install mcp")
        sys.exit(1)
    
    config = AnalysisConfig()
    if args.config:
        config = AnalysisConfig.from_file(args.config)
    else:
        config = AnalysisConfig.from_env()
    
    if args.workspace:
        config.workspace_path = args.workspace
    
    server = TokaAnalysisServer(config)
    await server.initialize()
    
    print(f"🚀 Starting Toka Analysis MCP Server")
    print(f"   Workspace: {config.workspace_path}")
    print(f"   Tools: {len(server.registry.tools) if server.registry else 0}")
    
    if args.stdio:
        await server.run_stdio()
    else:
        print("❌ Only stdio transport is currently supported")
        sys.exit(1)

async def list_tools_command(args: argparse.Namespace) -> None:
    """List available tools"""
    config = AnalysisConfig()
    registry = register_toka_tools(config)
    
    print("📋 Available Analysis Tools:\n")
    
    for name, spec in registry.tools.items():
        print(f"🔧 {name}")
        print(f"   Description: {spec.description}")
        print(f"   Capabilities: {', '.join(spec.capabilities)}")
        print(f"   Dependencies: {', '.join(spec.dependencies)}")
        if spec.resources:
            resources = ', '.join(f"{k}={v}" for k, v in spec.resources.items())
            print(f"   Resources: {resources}")
        print()

def create_parser() -> argparse.ArgumentParser:
    """Create the argument parser"""
    parser = argparse.ArgumentParser(
        description="Toka Analysis Tools - Code analysis and visualization",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Analyze control flow for a specific function
  toka-analysis control-flow --function my_function --formats mermaid json

  # Analyze dependencies with all formats
  toka-analysis dependency --formats mermaid json summary interactive

  # Run combined analysis
  toka-analysis combined

  # Start MCP server for Cursor integration
  toka-analysis mcp-server --stdio

  # List available tools
  toka-analysis list-tools
"""
    )
    
    parser.add_argument("--workspace", default=".", help="Workspace path")
    parser.add_argument("--output-dir", default="analysis_output", help="Output directory")
    parser.add_argument("--log-level", default="INFO", help="Log level")
    parser.add_argument("--config", help="Configuration file path")
    
    subparsers = parser.add_subparsers(dest="command", help="Commands")
    
    # Control flow analysis
    cf_parser = subparsers.add_parser("control-flow", help="Analyze control flow")
    cf_parser.add_argument("--function", help="Specific function to analyze")
    cf_parser.add_argument("--formats", nargs="+", 
                          choices=["mermaid", "json", "summary", "interactive", "system", "complexity"],
                          help="Output formats")
    
    # Dependency analysis
    dep_parser = subparsers.add_parser("dependency", help="Analyze dependencies")
    dep_parser.add_argument("--formats", nargs="+",
                           choices=["mermaid", "json", "summary", "interactive", "system"],
                           help="Output formats")
    
    # Combined analysis
    subparsers.add_parser("combined", help="Run both control flow and dependency analysis")
    
    # MCP server
    mcp_parser = subparsers.add_parser("mcp-server", help="Run MCP server")
    mcp_parser.add_argument("--stdio", action="store_true", default=True, 
                           help="Use stdio transport")
    mcp_parser.add_argument("--port", type=int, help="HTTP port (not yet implemented)")
    
    # List tools
    subparsers.add_parser("list-tools", help="List available tools")
    
    return parser

async def main():
    """Main CLI entry point"""
    parser = create_parser()
    args = parser.parse_args()
    
    # Setup logging
    setup_logging(args.log_level)
    
    if not args.command:
        parser.print_help()
        sys.exit(1)
    
    try:
        if args.command == "control-flow":
            await control_flow_command(args)
        elif args.command == "dependency":
            await dependency_command(args)
        elif args.command == "combined":
            await combined_command(args)
        elif args.command == "mcp-server":
            await mcp_server_command(args)
        elif args.command == "list-tools":
            await list_tools_command(args)
        else:
            print(f"❌ Unknown command: {args.command}")
            sys.exit(1)
    
    except KeyboardInterrupt:
        print("\n👋 Analysis interrupted by user")
    except Exception as e:
        logger.error(f"Analysis failed: {e}")
        print(f"❌ Analysis failed: {e}")
        sys.exit(1)

def cli_main():
    """Synchronous entry point for setuptools"""
    asyncio.run(main())

if __name__ == "__main__":
    cli_main()
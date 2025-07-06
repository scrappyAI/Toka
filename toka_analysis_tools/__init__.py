"""
Toka Analysis Tools Package

This package provides code analysis and visualization tools for the Toka agentic OS:
- Control Flow Graph Analysis
- Dependency Graph Analysis  
- Interactive Visualizations
- Mermaid/GitHub Integration
- MCP Server Integration
"""

from .control_flow import ControlFlowAnalyzer, ControlFlowVisualizer
from .dependency_graph import DependencyAnalyzer, DependencyVisualizer
from .mcp_server import TokaAnalysisServer
from .tool_registry import register_toka_tools
from .config import AnalysisConfig

__version__ = "0.2.1"
__all__ = [
    "ControlFlowAnalyzer",
    "ControlFlowVisualizer", 
    "DependencyAnalyzer",
    "DependencyVisualizer",
    "TokaAnalysisServer",
    "register_toka_tools",
    "AnalysisConfig",
]
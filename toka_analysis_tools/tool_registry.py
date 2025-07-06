"""
Tool registry for integrating analysis tools with the Toka system
"""

import asyncio
from dataclasses import asdict, dataclass
from typing import Any, Dict, List, Optional, Callable, Union
import logging

from .config import AnalysisConfig
from .control_flow import ControlFlowAnalyzer, ControlFlowVisualizer  
from .dependency_graph import DependencyAnalyzer, DependencyVisualizer

logger = logging.getLogger(__name__)

@dataclass
class TokaToolSpec:
    """Specification for a Toka tool"""
    name: str
    version: str
    description: str
    capabilities: List[str]
    dependencies: List[str]
    handler: Callable
    config_schema: Optional[Dict] = None
    resources: Optional[Dict] = None

class TokaToolRegistry:
    """Registry for Toka analysis tools"""
    
    def __init__(self, config: AnalysisConfig):
        self.config = config
        self.tools: Dict[str, TokaToolSpec] = {}
        self.active_analyzers: Dict[str, Any] = {}
        self._setup_logging()
    
    def _setup_logging(self) -> None:
        """Setup logging for the registry"""
        self.config.setup_logging()
        
    def register_tool(self, tool_spec: TokaToolSpec) -> None:
        """Register a tool with the system"""
        logger.info(f"Registering tool: {tool_spec.name} v{tool_spec.version}")
        
        # Validate tool specification
        if not tool_spec.name:
            raise ValueError("Tool name cannot be empty")
        
        if not tool_spec.handler:
            raise ValueError("Tool handler cannot be None")
        
        if not callable(tool_spec.handler):
            raise ValueError("Tool handler must be callable")
        
        # Register the tool
        self.tools[tool_spec.name] = tool_spec
        logger.info(f"Successfully registered tool: {tool_spec.name}")
    
    def get_tool(self, name: str) -> Optional[TokaToolSpec]:
        """Get a tool by name"""
        return self.tools.get(name)
    
    def list_tools(self) -> List[str]:
        """List all registered tools"""
        return list(self.tools.keys())
    
    def get_tool_capabilities(self, name: str) -> List[str]:
        """Get capabilities of a specific tool"""
        tool = self.get_tool(name)
        return tool.capabilities if tool else []
    
    async def execute_tool(self, name: str, **kwargs) -> Dict[str, Any]:
        """Execute a tool with given parameters"""
        tool = self.get_tool(name)
        if not tool:
            raise ValueError(f"Tool '{name}' not found")
        
        logger.info(f"Executing tool: {name}")
        
        try:
            # Add configuration to kwargs
            kwargs.setdefault("config", self.config)
            
            # Execute the tool
            if asyncio.iscoroutinefunction(tool.handler):
                result = await tool.handler(**kwargs)
            else:
                result = tool.handler(**kwargs)
            
            logger.info(f"Tool {name} executed successfully")
            return {
                "success": True,
                "tool": name,
                "result": result,
                "timestamp": asyncio.get_event_loop().time()
            }
            
        except Exception as e:
            logger.error(f"Tool {name} execution failed: {e}")
            return {
                "success": False,
                "tool": name,
                "error": str(e),
                "timestamp": asyncio.get_event_loop().time()
            }
    
    def to_agent_spec(self) -> Dict[str, Any]:
        """Convert registry to agent specification format"""
        return {
            "name": "analysis-tools",
            "version": self.config.tool_version,
            "domain": "code-analysis",
            "priority": "medium",
            "capabilities": {
                "primary": [
                    "control-flow-analysis",
                    "dependency-analysis", 
                    "code-visualization",
                    "interactive-graphs"
                ],
                "secondary": [
                    "mermaid-generation",
                    "performance-metrics",
                    "architectural-insights"
                ]
            },
            "tools": {
                name: {
                    "description": spec.description,
                    "capabilities": spec.capabilities,
                    "dependencies": spec.dependencies,
                    "resources": spec.resources or {}
                }
                for name, spec in self.tools.items()
            },
            "config": self.config.to_dict(),
            "objectives": [
                {
                    "description": "Analyze code structure and dependencies",
                    "deliverable": "Analysis reports and visualizations",
                    "validation": "Generated outputs are accurate and complete"
                }
            ]
        }

async def control_flow_analysis_handler(
    workspace_path: Optional[str] = None,
    function_name: Optional[str] = None,
    output_formats: Optional[List[str]] = None,
    config: Optional[AnalysisConfig] = None,
    **kwargs
) -> Dict[str, Any]:
    """Handler for control flow analysis"""
    if config is None:
        config = AnalysisConfig()
    
    if workspace_path:
        config.workspace_path = workspace_path
    
    if output_formats is None:
        output_formats = ["mermaid", "json", "summary"]
    
    # Initialize analyzer
    analyzer = ControlFlowAnalyzer(config.workspace_path)
    await analyzer.analyze_workspace()
    
    # Initialize visualizer
    visualizer = ControlFlowVisualizer(analyzer)
    
    results = {
        "workspace_path": config.workspace_path,
        "total_functions": len(analyzer.function_flows),
        "output_formats": output_formats,
        "outputs": {}
    }
    
    if function_name:
        # Analyze specific function
        if function_name not in analyzer.function_flows:
            return {"error": f"Function '{function_name}' not found"}
        
        # Generate requested formats
        if "mermaid" in output_formats:
            mermaid_content = visualizer.generate_mermaid_flowchart(
                function_name, 
                f"{config.output_dir}/control_flow/{function_name}"
            )
            results["outputs"]["mermaid"] = mermaid_content
        
        if "json" in output_formats:
            json_data = visualizer.export_function_json(
                function_name,
                f"{config.output_dir}/control_flow/{function_name}"
            )
            results["outputs"]["json"] = json_data
        
        if "summary" in output_formats:
            summary = visualizer.generate_textual_summary(
                function_name,
                f"{config.output_dir}/control_flow/{function_name}"
            )
            results["outputs"]["summary"] = summary
        
        if "interactive" in output_formats:
            html_content = visualizer.generate_interactive_html(
                function_name,
                f"{config.output_dir}/control_flow/{function_name}"
            )
            results["outputs"]["interactive"] = html_content
    
    else:
        # System-wide analysis
        if "system" in output_formats:
            visualizer.generate_system_flow_graph(
                f"{config.output_dir}/control_flow/system_flow"
            )
            results["outputs"]["system"] = "System flow graph generated"
        
        if "complexity" in output_formats:
            visualizer.generate_complexity_heatmap(
                f"{config.output_dir}/control_flow/complexity_heatmap"
            )
            results["outputs"]["complexity"] = "Complexity heatmap generated"
    
    return results

async def dependency_analysis_handler(
    workspace_path: Optional[str] = None,
    output_formats: Optional[List[str]] = None,
    config: Optional[AnalysisConfig] = None,
    **kwargs
) -> Dict[str, Any]:
    """Handler for dependency analysis"""
    if config is None:
        config = AnalysisConfig()
    
    if workspace_path:
        config.workspace_path = workspace_path
    
    if output_formats is None:
        output_formats = ["system", "individual"]
    
    # Initialize analyzer
    analyzer = DependencyAnalyzer(config.workspace_path)
    await analyzer.analyze_workspace()
    
    # Initialize visualizer
    visualizer = DependencyVisualizer(analyzer)
    
    results = {
        "workspace_path": config.workspace_path,
        "total_crates": len(analyzer.crates),
        "total_agents": len(analyzer.agent_specs),
        "output_formats": output_formats,
        "outputs": {}
    }
    
    # Generate requested formats
    if "system" in output_formats:
        visualizer.generate_system_graph(
            f"{config.output_dir}/dependency_graphs/system_dependencies"
        )
        results["outputs"]["system"] = "System dependency graph generated"
    
    if "individual" in output_formats:
        visualizer.generate_individual_crate_graphs(
            f"{config.output_dir}/dependency_graphs/individual_crates"
        )
        results["outputs"]["individual"] = "Individual crate graphs generated"
    
    if "agents" in output_formats:
        visualizer.generate_agent_composition_graph(
            f"{config.output_dir}/dependency_graphs/agent_composition"
        )
        results["outputs"]["agents"] = "Agent composition graph generated"
    
    if "layered" in output_formats:
        visualizer.generate_layered_architecture_graph(
            f"{config.output_dir}/dependency_graphs/layered_architecture"
        )
        results["outputs"]["layered"] = "Layered architecture graph generated"
    
    return results

async def combined_analysis_handler(
    workspace_path: Optional[str] = None,
    config: Optional[AnalysisConfig] = None,
    **kwargs
) -> Dict[str, Any]:
    """Handler for combined analysis"""
    if config is None:
        config = AnalysisConfig()
    
    if workspace_path:
        config.workspace_path = workspace_path
    
    # Run both analyses
    control_flow_results = await control_flow_analysis_handler(
        workspace_path=workspace_path,
        output_formats=["mermaid", "json", "system"],
        config=config
    )
    
    dependency_results = await dependency_analysis_handler(
        workspace_path=workspace_path,
        output_formats=["system", "agents", "layered"],
        config=config
    )
    
    return {
        "control_flow": control_flow_results,
        "dependency": dependency_results,
        "combined_summary": {
            "total_functions": control_flow_results.get("total_functions", 0),
            "total_crates": dependency_results.get("total_crates", 0),
            "total_agents": dependency_results.get("total_agents", 0),
        }
    }

def register_toka_tools(config: Optional[AnalysisConfig] = None) -> TokaToolRegistry:
    """Register all analysis tools with the Toka system"""
    if config is None:
        config = AnalysisConfig()
    
    registry = TokaToolRegistry(config)
    
    # Register control flow analysis tool
    registry.register_tool(TokaToolSpec(
        name="control-flow-analysis",
        version=config.tool_version,
        description="Analyze control flow patterns in Rust code",
        capabilities=[
            "rust-ast-analysis",
            "async-pattern-detection",
            "complexity-metrics",
            "mermaid-generation",
            "interactive-visualization"
        ],
        dependencies=["graphviz", "rust-analyzer"],
        handler=control_flow_analysis_handler,
        config_schema={
            "workspace_path": {"type": "string", "required": False},
            "function_name": {"type": "string", "required": False},
            "output_formats": {"type": "array", "required": False}
        },
        resources={
            "memory": config.resource_limits.get("max_memory", "512MB"),
            "cpu": config.resource_limits.get("max_cpu", "0.5"),
            "timeout": config.resource_limits.get("timeout", "1800")
        }
    ))
    
    # Register dependency analysis tool
    registry.register_tool(TokaToolSpec(
        name="dependency-analysis",
        version=config.tool_version,
        description="Analyze dependency relationships in Rust workspace",
        capabilities=[
            "cargo-manifest-parsing",
            "dependency-graph-generation",
            "agent-composition-analysis",
            "layered-architecture-visualization"
        ],
        dependencies=["graphviz", "cargo"],
        handler=dependency_analysis_handler,
        config_schema={
            "workspace_path": {"type": "string", "required": False},
            "output_formats": {"type": "array", "required": False}
        },
        resources={
            "memory": config.resource_limits.get("max_memory", "512MB"),
            "cpu": config.resource_limits.get("max_cpu", "0.5"),
            "timeout": config.resource_limits.get("timeout", "1800")
        }
    ))
    
    # Register combined analysis tool
    registry.register_tool(TokaToolSpec(
        name="combined-analysis",
        version=config.tool_version,
        description="Comprehensive code and dependency analysis",
        capabilities=[
            "full-workspace-analysis",
            "multi-format-output",
            "architectural-insights",
            "performance-metrics"
        ],
        dependencies=["graphviz", "rust-analyzer", "cargo"],
        handler=combined_analysis_handler,
        config_schema={
            "workspace_path": {"type": "string", "required": False}
        },
        resources={
            "memory": "1GB",
            "cpu": "1.0",
            "timeout": "3600"
        }
    ))
    
    logger.info(f"Registered {len(registry.tools)} analysis tools")
    return registry
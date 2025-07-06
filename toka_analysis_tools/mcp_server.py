"""
MCP (Model Context Protocol) Server for Toka Analysis Tools

This module provides an MCP server that exposes the analysis tools
to MCP clients like Cursor, allowing seamless integration with
development environments.
"""

import asyncio
import json
from typing import Any, Dict, List, Optional, Sequence
from dataclasses import asdict
import logging

try:
    from mcp import ClientSession, StdioServerParameters
    from mcp.server import Server
    from mcp.types import (
        Tool, 
        TextContent, 
        CallToolRequestParams,
        ListToolsRequestParams,
        GetPromptRequestParams,
        ListPromptsRequestParams,
        Prompt,
        PromptMessage,
        Resource,
        ListResourcesRequestParams,
        ReadResourceRequestParams
    )
    MCP_AVAILABLE = True
except ImportError:
    MCP_AVAILABLE = False
    
    # Fallback types for when MCP is not available
    class Tool:
        def __init__(self, name: str, description: str, inputSchema: Dict[str, Any]):
            self.name = name
            self.description = description
            self.inputSchema = inputSchema
    
    class TextContent:
        def __init__(self, type: str, text: str):
            self.type = type
            self.text = text

from .config import AnalysisConfig
from .tool_registry import register_toka_tools, TokaToolRegistry

logger = logging.getLogger(__name__)

class TokaAnalysisServer:
    """MCP Server for Toka Analysis Tools"""
    
    def __init__(self, config: Optional[AnalysisConfig] = None):
        self.config = config or AnalysisConfig()
        self.registry: Optional[TokaToolRegistry] = None
        self.server: Optional[Server] = None
        
        if not MCP_AVAILABLE:
            logger.warning("MCP library not available. Server will run in fallback mode.")
    
    async def initialize(self) -> None:
        """Initialize the server and tool registry"""
        # Register tools
        self.registry = register_toka_tools(self.config)
        
        if MCP_AVAILABLE:
            # Create MCP server
            self.server = Server("toka-analysis-server")
            
            # Register MCP handlers
            self._register_mcp_handlers()
            
            logger.info(f"MCP server initialized with {len(self.registry.tools)} tools")
        else:
            logger.info(f"Fallback server initialized with {len(self.registry.tools)} tools")
    
    def _register_mcp_handlers(self) -> None:
        """Register MCP request handlers"""
        if not self.server or not MCP_AVAILABLE:
            return
        
        @self.server.list_tools()
        async def list_tools(params: ListToolsRequestParams) -> List[Tool]:
            """List available tools"""
            tools = []
            
                                      if self.registry:
                 for tool_name, tool_spec in self.registry.tools.items():
                     # Convert tool spec to MCP Tool format
                     input_schema = {
                         "type": "object",
                         "properties": tool_spec.config_schema or {},
                         "required": []
                     }
                     
                     # Add common parameters
                     input_schema["properties"].update({
                         "workspace_path": {
                             "type": "string",
                             "description": "Path to the workspace to analyze",
                             "default": self.config.workspace_path
                         },
                         "output_formats": {
                             "type": "array",
                             "items": {"type": "string"},
                             "description": "Output formats to generate",
                             "default": ["mermaid", "json", "summary"]
                         }
                     })
                     
                     tool = Tool(
                         name=tool_name,
                         description=tool_spec.description,
                         inputSchema=input_schema
                     )
                     tools.append(tool)
            
            return tools
        
        @self.server.call_tool()
        async def call_tool(params: CallToolRequestParams) -> Sequence[TextContent]:
            """Execute a tool"""
            tool_name = params.name
            arguments = params.arguments or {}
            
            logger.info(f"Executing tool: {tool_name} with args: {arguments}")
            
            try:
                # Execute the tool
                if self.registry:
                    result = await self.registry.execute_tool(tool_name, **arguments)
                else:
                    raise RuntimeError("Tool registry not initialized")
                
                if result["success"]:
                    # Format successful result
                    response_text = self._format_tool_result(tool_name, result["result"])
                else:
                    # Format error result
                    response_text = f"Error executing {tool_name}: {result['error']}"
                
                return [TextContent(type="text", text=response_text)]
                
            except Exception as e:
                logger.error(f"Tool execution error: {e}")
                error_text = f"Failed to execute {tool_name}: {str(e)}"
                return [TextContent(type="text", text=error_text)]
        
        @self.server.list_prompts()
        async def list_prompts(params: ListPromptsRequestParams) -> List[Prompt]:
            """List available prompts for analysis guidance"""
            prompts = [
                Prompt(
                    name="analyze-architecture",
                    description="Analyze the overall system architecture",
                    arguments=[
                        {
                            "name": "focus_area",
                            "description": "Specific area to focus on (e.g., 'storage', 'runtime', 'agents')",
                            "required": False
                        }
                    ]
                ),
                Prompt(
                    name="analyze-function",
                    description="Analyze a specific function's control flow",
                    arguments=[
                        {
                            "name": "function_name",
                            "description": "Name of the function to analyze",
                            "required": True
                        }
                    ]
                ),
                Prompt(
                    name="dependency-review",
                    description="Review dependency structure and identify potential issues",
                    arguments=[
                        {
                            "name": "category",
                            "description": "Crate category to focus on",
                            "required": False
                        }
                    ]
                )
            ]
            return prompts
        
        @self.server.get_prompt()
        async def get_prompt(params: GetPromptRequestParams) -> PromptMessage:
            """Get a specific analysis prompt"""
            prompt_name = params.name
            arguments = params.arguments or {}
            
            if prompt_name == "analyze-architecture":
                focus_area = arguments.get("focus_area", "overall")
                prompt_text = self._generate_architecture_prompt(focus_area)
            elif prompt_name == "analyze-function":
                function_name = arguments.get("function_name", "")
                prompt_text = self._generate_function_prompt(function_name)
            elif prompt_name == "dependency-review":
                category = arguments.get("category", "all")
                prompt_text = self._generate_dependency_prompt(category)
            else:
                prompt_text = f"Unknown prompt: {prompt_name}"
            
            return PromptMessage(
                role="user",
                content=TextContent(type="text", text=prompt_text)
            )
        
        @self.server.list_resources()
        async def list_resources(params: ListResourcesRequestParams) -> List[Resource]:
            """List available analysis resources"""
            resources = []
            
            # Add configuration resource
            resources.append(Resource(
                uri="toka://config",
                name="Analysis Configuration",
                description="Current analysis tool configuration",
                mimeType="application/json"
            ))
            
            # Add tool registry resource
            resources.append(Resource(
                uri="toka://tools",
                name="Tool Registry",
                description="Available analysis tools and their capabilities",
                mimeType="application/json"
            ))
            
            return resources
        
        @self.server.read_resource()
        async def read_resource(params: ReadResourceRequestParams) -> str:
            """Read a specific resource"""
            uri = params.uri
            
            if uri == "toka://config":
                return json.dumps(self.config.to_dict(), indent=2)
            elif uri == "toka://tools":
                tools_info = {}
                                 if self.registry:
                     for name, spec in self.registry.tools.items():
                    tools_info[name] = {
                        "description": spec.description,
                        "capabilities": spec.capabilities,
                        "dependencies": spec.dependencies,
                        "resources": spec.resources
                    }
                return json.dumps(tools_info, indent=2)
            else:
                return f"Unknown resource: {uri}"
    
    def _format_tool_result(self, tool_name: str, result: Dict[str, Any]) -> str:
        """Format tool execution result for MCP response"""
        if tool_name == "control-flow-analysis":
            return self._format_control_flow_result(result)
        elif tool_name == "dependency-analysis":
            return self._format_dependency_result(result)
        elif tool_name == "combined-analysis":
            return self._format_combined_result(result)
        else:
            return json.dumps(result, indent=2)
    
    def _format_control_flow_result(self, result: Dict[str, Any]) -> str:
        """Format control flow analysis result"""
        lines = [
            f"# Control Flow Analysis Results",
            f"",
            f"**Workspace:** {result.get('workspace_path', 'Unknown')}",
            f"**Functions Analyzed:** {result.get('total_functions', 0)}",
            f"**Output Formats:** {', '.join(result.get('output_formats', []))}",
            f"",
        ]
        
        outputs = result.get('outputs', {})
        
        if 'mermaid' in outputs:
            lines.extend([
                "## Mermaid Flowchart",
                "```mermaid",
                outputs['mermaid'],
                "```",
                ""
            ])
        
        if 'summary' in outputs:
            lines.extend([
                "## Analysis Summary",
                outputs['summary'],
                ""
            ])
        
        if 'json' in outputs:
            lines.extend([
                "## Structured Data",
                "```json",
                json.dumps(outputs['json'], indent=2),
                "```",
                ""
            ])
        
        return "\n".join(lines)
    
    def _format_dependency_result(self, result: Dict[str, Any]) -> str:
        """Format dependency analysis result"""
        lines = [
            f"# Dependency Analysis Results",
            f"",
            f"**Workspace:** {result.get('workspace_path', 'Unknown')}",
            f"**Crates Analyzed:** {result.get('total_crates', 0)}",
            f"**Agents Found:** {result.get('total_agents', 0)}",
            f"",
        ]
        
        outputs = result.get('outputs', {})
        for output_type, message in outputs.items():
            lines.extend([
                f"## {output_type.title()} Output",
                message,
                ""
            ])
        
        return "\n".join(lines)
    
    def _format_combined_result(self, result: Dict[str, Any]) -> str:
        """Format combined analysis result"""
        lines = [
            f"# Combined Analysis Results",
            f"",
        ]
        
        summary = result.get('combined_summary', {})
        lines.extend([
            f"**Functions:** {summary.get('total_functions', 0)}",
            f"**Crates:** {summary.get('total_crates', 0)}",
            f"**Agents:** {summary.get('total_agents', 0)}",
            f"",
        ])
        
        if 'control_flow' in result:
            lines.extend([
                "## Control Flow Analysis",
                self._format_control_flow_result(result['control_flow']),
                ""
            ])
        
        if 'dependency' in result:
            lines.extend([
                "## Dependency Analysis", 
                self._format_dependency_result(result['dependency']),
                ""
            ])
        
        return "\n".join(lines)
    
    def _generate_architecture_prompt(self, focus_area: str) -> str:
        """Generate architecture analysis prompt"""
        return f"""
Analyze the Toka system architecture with focus on: {focus_area}

Please run the combined analysis tools to understand:

1. **System Structure**: How are the components organized?
2. **Dependency Patterns**: What are the key dependencies between crates?
3. **Agent Orchestration**: How do agents interact with the system?
4. **Control Flow**: What are the main execution patterns?

Use these tools:
- `combined-analysis` for overall system analysis
- `dependency-analysis` with formats: ["system", "agents", "layered"]
- `control-flow-analysis` with formats: ["system", "complexity"]

Focus particularly on the {focus_area} aspect and provide architectural insights.
"""
    
    def _generate_function_prompt(self, function_name: str) -> str:
        """Generate function analysis prompt"""
        return f"""
Analyze the function: {function_name}

Please use the control flow analysis tool to understand:

1. **Control Flow**: What is the execution path through this function?
2. **Complexity**: How complex is this function?
3. **Async Patterns**: Does it use async/await? How?
4. **Error Handling**: How are errors managed?

Use this tool:
- `control-flow-analysis` with function_name: "{function_name}" and formats: ["mermaid", "json", "summary", "interactive"]

Provide insights on the function's design and any potential improvements.
"""
    
    def _generate_dependency_prompt(self, category: str) -> str:
        """Generate dependency review prompt"""
        return f"""
Review dependency structure for category: {category}

Please analyze the dependency patterns to identify:

1. **Circular Dependencies**: Are there any problematic cycles?
2. **Layering Violations**: Do dependencies respect architectural layers?
3. **Coupling**: Which crates are highly coupled?
4. **Abstraction**: Are abstractions properly used?

Use this tool:
- `dependency-analysis` with formats: ["system", "individual", "layered"]

Focus on the {category} category and suggest improvements for better modularity.
"""
    
    async def run_stdio(self) -> None:
        """Run the server using stdio transport"""
        if not MCP_AVAILABLE:
            logger.error("MCP library not available. Cannot run stdio server.")
            return
        
        if not self.server:
            raise RuntimeError("Server not initialized. Call initialize() first.")
        
        async with self.server.stdio_server(StdioServerParameters()) as server:
            logger.info("Toka Analysis MCP Server started on stdio")
            await server.run()
    
    async def run_http(self, port: Optional[int] = None) -> None:
        """Run the server using HTTP transport"""
        port = port or self.config.mcp_port
        
        if not MCP_AVAILABLE:
            logger.error("MCP library not available. Cannot run HTTP server.")
            return
        
        # TODO: Implement HTTP transport when available in MCP library
        logger.error("HTTP transport not yet implemented")
    
    async def execute_tool_direct(self, tool_name: str, **kwargs) -> Dict[str, Any]:
        """Execute a tool directly without MCP (for testing/fallback)"""
        if not self.registry:
            raise RuntimeError("Server not initialized. Call initialize() first.")
        
        return await self.registry.execute_tool(tool_name, **kwargs)
    
    def get_tool_list(self) -> List[Dict[str, Any]]:
        """Get list of available tools (for non-MCP clients)"""
        if not self.registry:
            return []
        
        tools = []
        for tool_name, tool_spec in self.registry.tools.items():
            tools.append({
                "name": tool_name,
                "description": tool_spec.description,
                "capabilities": tool_spec.capabilities,
                "dependencies": tool_spec.dependencies,
                "resources": tool_spec.resources,
                "config_schema": tool_spec.config_schema
            })
        
        return tools

async def main():
    """Main entry point for running the MCP server"""
    import argparse
    
    parser = argparse.ArgumentParser(description="Toka Analysis MCP Server")
    parser.add_argument("--config", help="Configuration file path")
    parser.add_argument("--workspace", help="Workspace path", default=".")
    parser.add_argument("--port", type=int, help="HTTP port (when HTTP transport is available)")
    parser.add_argument("--stdio", action="store_true", help="Use stdio transport", default=True)
    parser.add_argument("--log-level", help="Log level", default="INFO")
    
    args = parser.parse_args()
    
    # Setup logging
    logging.basicConfig(
        level=getattr(logging, args.log_level.upper()),
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )
    
    # Load configuration
    if args.config:
        config = AnalysisConfig.from_file(args.config)
    else:
        config = AnalysisConfig.from_env()
    
    if args.workspace:
        config.workspace_path = args.workspace
    
    # Create and initialize server
    server = TokaAnalysisServer(config)
    await server.initialize()
    
    # Run server
    if args.stdio:
        await server.run_stdio()
    elif args.port:
        await server.run_http(args.port)
    else:
        logger.error("Must specify either --stdio or --port")

if __name__ == "__main__":
    asyncio.run(main())
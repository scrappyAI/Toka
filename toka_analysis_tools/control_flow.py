"""
Control Flow Analysis for Rust Code

This module provides comprehensive control flow analysis capabilities:
- Function-level control flow graphs
- Async/await pattern detection
- Complexity metrics calculation
- Multiple output formats (Mermaid, JSON, Interactive HTML)
- System-wide flow analysis
"""

import os
import re
import json
import asyncio
import aiofiles
from pathlib import Path
from typing import Dict, List, Set, Optional, Tuple, Union, Any
from dataclasses import dataclass, field
from collections import defaultdict, deque
from enum import Enum
from concurrent.futures import ThreadPoolExecutor, as_completed
import logging

from .config import AnalysisConfig

logger = logging.getLogger(__name__)

class FlowNodeType(Enum):
    """Types of nodes in control flow graphs"""
    ENTRY = "entry"
    EXIT = "exit"
    STATEMENT = "statement"
    CONDITION = "condition"
    LOOP = "loop"
    ASYNC_POINT = "async_point"
    ERROR_HANDLER = "error_handler"
    STATE_TRANSITION = "state_transition"
    FUNCTION_CALL = "function_call"
    AWAIT_POINT = "await_point"
    SPAWN_POINT = "spawn_point"
    RETURN_POINT = "return_point"

class ExecutionPattern(Enum):
    """Common execution patterns in the system"""
    SEQUENTIAL = "sequential"
    CONDITIONAL = "conditional"
    LOOP = "loop"
    ASYNC_SPAWN = "async_spawn"
    ERROR_PROPAGATION = "error_propagation"
    STATE_MACHINE = "state_machine"
    EVENT_HANDLING = "event_handling"
    ORCHESTRATION = "orchestration"

@dataclass
class FlowNode:
    """A node in the control flow graph"""
    id: str
    node_type: FlowNodeType
    label: str
    source_line: Optional[int] = None
    source_file: Optional[str] = None
    metadata: Dict[str, Any] = field(default_factory=dict)
    execution_pattern: Optional[ExecutionPattern] = None

@dataclass
class FlowEdge:
    """An edge in the control flow graph"""
    source: str
    target: str
    label: Optional[str] = None
    condition: Optional[str] = None
    edge_type: str = "control"  # control, data, async, error
    probability: Optional[float] = None

@dataclass
class FunctionFlow:
    """Control flow information for a function"""
    name: str
    file_path: str
    start_line: int
    end_line: int
    nodes: Dict[str, FlowNode] = field(default_factory=dict)
    edges: List[FlowEdge] = field(default_factory=list)
    async_function: bool = False
    return_type: Optional[str] = None
    error_paths: List[str] = field(default_factory=list)
    async_spawn_points: List[str] = field(default_factory=list)
    state_transitions: List[Tuple[str, str]] = field(default_factory=list)
    complexity_metrics: Dict[str, int] = field(default_factory=dict)

@dataclass
class SystemFlow:
    """System-wide control flow patterns"""
    component_interactions: Dict[str, List[str]] = field(default_factory=dict)
    async_coordination_patterns: List[Dict[str, Any]] = field(default_factory=list)
    error_propagation_chains: List[List[str]] = field(default_factory=list)
    state_machine_flows: Dict[str, Dict[str, Any]] = field(default_factory=dict)
    orchestration_sequences: List[Dict[str, Any]] = field(default_factory=list)

class RustPatternAnalyzer:
    """Analyzes Rust-specific control flow patterns"""
    
    def __init__(self):
        self.async_patterns = [
            r'async\s+fn\s+(\w+)',
            r'\.await\s*[;\?]?',
            r'tokio::spawn\s*\(',
            r'tokio::select!\s*\{',
            r'futures::\w+',
        ]
        
        self.error_patterns = [
            r'Result<([^>]+)>',
            r'\.map_err\s*\(',
            r'\.unwrap_or\s*\(',
            r'\?\s*[;\}]',
            r'match\s+\w+\s*\{[^}]*Err\s*\(',
            r'if\s+let\s+Err\s*\(',
        ]
        
        self.state_patterns = [
            r'enum\s+(\w*State\w*)',
            r'match\s+self\.state\s*\{',
            r'state\s*=\s*\w+::',
            r'\.transition_to\s*\(',
            r'StateMachine',
        ]
        
        self.concurrency_patterns = [
            r'Arc<([^>]+)>',
            r'Mutex<([^>]+)>',
            r'RwLock<([^>]+)>',
            r'mpsc::\w+',
            r'broadcast::\w+',
            r'oneshot::\w+',
        ]

    def analyze_function_patterns(self, source_code: str, function_name: str) -> Dict[str, Any]:
        """Analyze patterns within a function"""
        patterns = {
            'is_async': any(re.search(pattern, source_code) for pattern in self.async_patterns[:1]),
            'has_error_handling': any(re.search(pattern, source_code) for pattern in self.error_patterns),
            'has_state_transitions': any(re.search(pattern, source_code) for pattern in self.state_patterns),
            'uses_concurrency': any(re.search(pattern, source_code) for pattern in self.concurrency_patterns),
            'await_points': len(re.findall(r'\.await', source_code)),
            'error_propagations': len(re.findall(r'\?', source_code)),
            'spawn_points': len(re.findall(r'tokio::spawn|std::thread::spawn', source_code)),
        }
        return patterns

class ControlFlowAnalyzer:
    """Main control flow analyzer"""
    
    def __init__(self, workspace_path: str, config: Optional[AnalysisConfig] = None):
        self.workspace_path = Path(workspace_path)
        self.config = config or AnalysisConfig()
        self.pattern_analyzer = RustPatternAnalyzer()
        self.function_flows: Dict[str, FunctionFlow] = {}
        self.system_flow = SystemFlow()
        self.rust_files: List[Path] = []
        
    async def analyze_workspace(self) -> None:
        """Analyze control flow patterns across the workspace"""
        logger.info("Starting control flow analysis...")
        
        # Validate workspace
        if not self.workspace_path.exists():
            raise ValueError(f"Workspace path does not exist: {self.workspace_path}")
        
        # Create output directories
        self.config.create_output_dirs()
        
        # Find all Rust source files
        await self._discover_rust_files()
        
        # Analyze function-level control flows
        await self._analyze_function_flows()
        
        # Analyze system-level patterns
        await self._analyze_system_patterns()
        
        # Compute complexity metrics
        self._compute_complexity_metrics()
        
        logger.info(f"Analysis complete. Found {len(self.function_flows)} functions across {len(self.rust_files)} files.")
    
    async def _discover_rust_files(self) -> None:
        """Discover all Rust source files in the workspace"""
        for root in ['crates', 'src', 'tests']:
            root_path = self.workspace_path / root
            if root_path.exists():
                for rust_file in root_path.rglob('*.rs'):
                    # Skip excluded patterns
                    if not any(pattern in str(rust_file) for pattern in self.config.exclude_patterns):
                        self.rust_files.append(rust_file)
        
        logger.info(f"Discovered {len(self.rust_files)} Rust source files")
    
    async def _analyze_function_flows(self) -> None:
        """Analyze control flow within individual functions"""
        with ThreadPoolExecutor(max_workers=self.config.max_concurrent_analyzers) as executor:
            futures = [
                executor.submit(self._analyze_single_file, file_path)
                for file_path in self.rust_files
            ]
            
            for future in as_completed(futures):
                try:
                    functions = future.result()
                    for func in functions:
                        self.function_flows[func.name] = func
                except Exception as e:
                    logger.error(f"Error analyzing file: {e}")
    
    def _analyze_single_file(self, file_path: Path) -> List[FunctionFlow]:
        """Analyze control flow patterns in a single file"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            functions = self._extract_functions(content, str(file_path))
            analyzed_functions = []
            
            for func in functions:
                # Extract function source
                lines = content.split('\n')
                if func.start_line > 0 and func.end_line <= len(lines):
                    func_source = '\n'.join(lines[func.start_line-1:func.end_line])
                    
                    # Analyze patterns
                    patterns = self.pattern_analyzer.analyze_function_patterns(func_source, func.name)
                    func.async_function = patterns['is_async']
                    
                    # Build control flow graph
                    self._build_function_cfg(func, func_source)
                    
                    analyzed_functions.append(func)
            
            return analyzed_functions
            
        except Exception as e:
            logger.error(f"Error analyzing {file_path}: {e}")
            return []
    
    def _extract_functions(self, content: str, file_path: str) -> List[FunctionFlow]:
        """Extract function definitions from source code"""
        functions = []
        lines = content.split('\n')
        
        for i, line in enumerate(lines):
            # Match function definitions
            func_match = re.match(r'\s*(pub\s+)?(async\s+)?fn\s+(\w+)\s*\(', line)
            if func_match:
                func_name = func_match.group(3)
                start_line = i + 1
                
                # Find function end
                brace_count = 0
                end_line = start_line
                
                for j in range(i, len(lines)):
                    if '{' in lines[j]:
                        brace_count += lines[j].count('{')
                    if '}' in lines[j]:
                        brace_count -= lines[j].count('}')
                        if brace_count == 0:
                            end_line = j + 1
                            break
                
                func_flow = FunctionFlow(
                    name=func_name,
                    file_path=file_path,
                    start_line=start_line,
                    end_line=end_line
                )
                functions.append(func_flow)
        
        return functions
    
    def _build_function_cfg(self, func: FunctionFlow, source: str) -> None:
        """Build control flow graph for a function"""
        lines = source.split('\n')
        
        # Create entry node
        entry_id = f"{func.name}_entry"
        func.nodes[entry_id] = FlowNode(
            id=entry_id,
            node_type=FlowNodeType.ENTRY,
            label=f"Entry: {func.name}",
            source_line=func.start_line,
            source_file=func.file_path
        )
        
        current_node_id = entry_id
        
        # Analyze line by line
        for i, line in enumerate(lines):
            line_stripped = line.strip()
            if not line_stripped or line_stripped.startswith('//'):
                continue
            
            node_id = f"{func.name}_{i}"
            node_type = self._classify_line(line_stripped)
            
            node = FlowNode(
                id=node_id,
                node_type=node_type,
                label=line_stripped[:50] + "..." if len(line_stripped) > 50 else line_stripped,
                source_line=func.start_line + i,
                source_file=func.file_path
            )
            
            func.nodes[node_id] = node
            
            # Add edge from previous node
            if current_node_id != node_id:
                edge_type = "control"
                edge_label = None
                
                if '.await' in line_stripped:
                    edge_type = "async"
                    edge_label = "await"
                elif '?' in line_stripped:
                    edge_type = "error"
                    edge_label = "error propagation"
                elif 'match' in line_stripped or 'if' in line_stripped:
                    edge_type = "control"
                    edge_label = "conditional"
                
                func.edges.append(FlowEdge(
                    source=current_node_id,
                    target=node_id,
                    label=edge_label,
                    edge_type=edge_type
                ))
            
            current_node_id = node_id
        
        # Create exit node
        exit_id = f"{func.name}_exit"
        func.nodes[exit_id] = FlowNode(
            id=exit_id,
            node_type=FlowNodeType.EXIT,
            label=f"Exit: {func.name}",
            source_line=func.end_line,
            source_file=func.file_path
        )
        
        # Connect last node to exit
        if current_node_id != exit_id:
            func.edges.append(FlowEdge(
                source=current_node_id,
                target=exit_id,
                edge_type="control"
            ))
    
    def _classify_line(self, line: str) -> FlowNodeType:
        """Classify a line of code into a node type"""
        if '.await' in line:
            return FlowNodeType.AWAIT_POINT
        elif 'tokio::spawn' in line or 'std::thread::spawn' in line:
            return FlowNodeType.SPAWN_POINT
        elif 'return' in line:
            return FlowNodeType.RETURN_POINT
        elif line.startswith('if ') or line.startswith('match '):
            return FlowNodeType.CONDITION
        elif line.startswith('for ') or line.startswith('while ') or line.startswith('loop'):
            return FlowNodeType.LOOP
        elif 'state' in line.lower() and ('=' in line or 'transition' in line):
            return FlowNodeType.STATE_TRANSITION
        elif '?' in line or '.map_err' in line or 'unwrap' in line:
            return FlowNodeType.ERROR_HANDLER
        elif '(' in line and not line.startswith('//'):
            return FlowNodeType.FUNCTION_CALL
        else:
            return FlowNodeType.STATEMENT
    
    async def _analyze_system_patterns(self) -> None:
        """Analyze system-wide control flow patterns"""
        # Analyze component interactions
        await self._analyze_component_interactions()
        
        # Analyze async coordination patterns
        await self._analyze_async_patterns()
        
        # Analyze error propagation chains
        await self._analyze_error_chains()
        
        # Analyze state machines
        await self._analyze_state_machines()
    
    async def _analyze_component_interactions(self) -> None:
        """Analyze how components interact with each other"""
        component_map = {
            'kernel': ['toka-kernel'],
            'runtime': ['toka-runtime', 'toka-agent-runtime'],
            'orchestration': ['toka-orchestration'],
            'storage': ['toka-store-core', 'toka-store-memory', 'toka-store-sled', 'toka-store-sqlite'],
            'bus': ['toka-bus-core'],
            'auth': ['toka-auth'],
            'llm': ['toka-llm-gateway'],
            'cli': ['toka-cli', 'toka-config-cli'],
        }
        
        # Map functions to components
        function_components = {}
        for func_name, func_flow in self.function_flows.items():
            for component, crates in component_map.items():
                if any(crate in func_flow.file_path for crate in crates):
                    function_components[func_name] = component
                    break
        
        # Analyze interactions based on function calls
        for func_name, func_flow in self.function_flows.items():
            source_component = function_components.get(func_name, 'unknown')
            
            for edge in func_flow.edges:
                if edge.edge_type == "control":
                    target_node = func_flow.nodes.get(edge.target)
                    if target_node and target_node.node_type == FlowNodeType.FUNCTION_CALL:
                        # Extract called function name
                        call_match = re.search(r'(\w+)\s*\(', target_node.label)
                        if call_match:
                            called_func = call_match.group(1)
                            target_component = function_components.get(called_func)
                            
                            if target_component and target_component != source_component:
                                if source_component not in self.system_flow.component_interactions:
                                    self.system_flow.component_interactions[source_component] = []
                                if target_component not in self.system_flow.component_interactions[source_component]:
                                    self.system_flow.component_interactions[source_component].append(target_component)
    
    async def _analyze_async_patterns(self) -> None:
        """Analyze async coordination patterns"""
        async_patterns = []
        
        for func_name, func_flow in self.function_flows.items():
            if func_flow.async_function:
                await_points = []
                spawn_points = []
                
                for node_id, node in func_flow.nodes.items():
                    if node.node_type == FlowNodeType.AWAIT_POINT:
                        await_points.append(node_id)
                    elif node.node_type == FlowNodeType.SPAWN_POINT:
                        spawn_points.append(node_id)
                
                if await_points or spawn_points:
                    pattern = {
                        'function': func_name,
                        'file': func_flow.file_path,
                        'await_points': await_points,
                        'spawn_points': spawn_points,
                        'pattern_type': self._classify_async_pattern(await_points, spawn_points)
                    }
                    async_patterns.append(pattern)
        
        self.system_flow.async_coordination_patterns = async_patterns
    
    def _classify_async_pattern(self, await_points: List[str], spawn_points: List[str]) -> str:
        """Classify the type of async pattern"""
        if spawn_points and await_points:
            return "spawn_and_await"
        elif spawn_points:
            return "fire_and_forget"
        elif len(await_points) > 3:
            return "sequential_async"
        elif await_points:
            return "simple_async"
        else:
            return "sync_in_async"
    
    async def _analyze_error_chains(self) -> None:
        """Analyze error propagation chains"""
        # TODO: Implement error chain analysis
        pass
    
    async def _analyze_state_machines(self) -> None:
        """Analyze state machine patterns"""
        # TODO: Implement state machine analysis
        pass
    
    def _compute_complexity_metrics(self) -> None:
        """Compute complexity metrics for each function"""
        for func_name, func_flow in self.function_flows.items():
            # Cyclomatic complexity
            decision_nodes = sum(1 for node in func_flow.nodes.values() 
                               if node.node_type in [FlowNodeType.CONDITION, FlowNodeType.LOOP])
            cyclomatic_complexity = decision_nodes + 1
            
            # Async complexity
            async_points = sum(1 for node in func_flow.nodes.values()
                             if node.node_type in [FlowNodeType.AWAIT_POINT, FlowNodeType.SPAWN_POINT])
            
            # Error handling complexity
            error_points = sum(1 for node in func_flow.nodes.values()
                             if node.node_type == FlowNodeType.ERROR_HANDLER)
            
            func_flow.complexity_metrics = {
                'cyclomatic_complexity': cyclomatic_complexity,
                'async_complexity': async_points,
                'error_handling_complexity': error_points,
                'total_nodes': len(func_flow.nodes),
                'total_edges': len(func_flow.edges)
            }

class ControlFlowVisualizer:
    """Generate visual control flow graphs"""
    
    def __init__(self, analyzer: ControlFlowAnalyzer):
        self.analyzer = analyzer
        self.config = analyzer.config
        self.colors = {
            FlowNodeType.ENTRY: '#4CAF50',
            FlowNodeType.EXIT: '#F44336',
            FlowNodeType.STATEMENT: '#E3F2FD',
            FlowNodeType.CONDITION: '#FFF59D',
            FlowNodeType.LOOP: '#FFECB3',
            FlowNodeType.ASYNC_POINT: '#E1BEE7',
            FlowNodeType.AWAIT_POINT: '#CE93D8',
            FlowNodeType.SPAWN_POINT: '#BA68C8',
            FlowNodeType.ERROR_HANDLER: '#FFCDD2',
            FlowNodeType.STATE_TRANSITION: '#C8E6C9',
            FlowNodeType.FUNCTION_CALL: '#BBDEFB',
            FlowNodeType.RETURN_POINT: '#FFE0B2',
        }
        
        self.edge_colors = {
            'control': 'black',
            'async': 'purple',
            'error': 'red',
            'data': 'blue',
        }
    
    def generate_mermaid_flowchart(self, function_name: str, output_path: Optional[str] = None) -> str:
        """Generate Mermaid flowchart for a function"""
        if function_name not in self.analyzer.function_flows:
            logger.error(f"Function '{function_name}' not found")
            return ""
        
        func_flow = self.analyzer.function_flows[function_name]
        
        # Build Mermaid flowchart
        mermaid = ["flowchart TD"]
        
        # Add nodes with proper IDs and labels
        node_mapping = {}
        for i, (node_id, node) in enumerate(func_flow.nodes.items()):
            clean_id = f"N{i}"
            node_mapping[node_id] = clean_id
            
            # Escape special characters for Mermaid
            label = node.label.replace('"', "'").replace('\n', '<br/>')
            if len(label) > 60:
                label = label[:57] + "..."
            
            # Choose shape based on node type
            if node.node_type == FlowNodeType.ENTRY:
                mermaid.append(f'    {clean_id}(["🚀 {label}"])')
            elif node.node_type == FlowNodeType.EXIT:
                mermaid.append(f'    {clean_id}(["🏁 {label}"])')
            elif node.node_type == FlowNodeType.CONDITION:
                mermaid.append(f'    {clean_id}{{{"{label}"}}}')
            elif node.node_type == FlowNodeType.LOOP:
                mermaid.append(f'    {clean_id}[/"🔄 {label}"\\]')
            elif node.node_type == FlowNodeType.ASYNC_POINT:
                mermaid.append(f'    {clean_id}(("⚡ {label}"))')
            elif node.node_type == FlowNodeType.ERROR_HANDLER:
                mermaid.append(f'    {clean_id}[["❌ {label}"]]')
            else:
                mermaid.append(f'    {clean_id}["{label}"]')
        
        # Add edges with styling
        for edge in func_flow.edges:
            source_id = node_mapping.get(edge.source, edge.source)
            target_id = node_mapping.get(edge.target, edge.target)
            
            edge_label = edge.label or ""
            if edge.condition:
                edge_label = f"{edge_label} [{edge.condition}]" if edge_label else f"[{edge.condition}]"
            
            if edge.edge_type == "async":
                mermaid.append(f'    {source_id} -.->|"{edge_label}"| {target_id}')
            elif edge.edge_type == "error":
                mermaid.append(f'    {source_id} ==>|"{edge_label}"| {target_id}')
            else:
                if edge_label:
                    mermaid.append(f'    {source_id} -->|"{edge_label}"| {target_id}')
                else:
                    mermaid.append(f'    {source_id} --> {target_id}')
        
        # Add styling
        mermaid.extend([
            "",
            "    %% Styling",
            "    classDef entryNode fill:#4CAF50,stroke:#2E7D32,color:#fff",
            "    classDef exitNode fill:#F44336,stroke:#C62828,color:#fff", 
            "    classDef conditionNode fill:#FFF59D,stroke:#F57F17",
            "    classDef loopNode fill:#FFECB3,stroke:#E65100",
            "    classDef asyncNode fill:#E1BEE7,stroke:#7B1FA2",
            "    classDef errorNode fill:#FFCDD2,stroke:#C62828"
        ])
        
        mermaid_content = "\n".join(mermaid)
        
        # Add complexity information as comment
        metrics = func_flow.complexity_metrics
        mermaid_content += f"\n\n%% Complexity Metrics:\n"
        mermaid_content += f"%% Cyclomatic Complexity: {metrics.get('cyclomatic_complexity', 0)}\n"
        mermaid_content += f"%% Async Complexity: {metrics.get('async_complexity', 0)}\n"
        mermaid_content += f"%% Error Handling Complexity: {metrics.get('error_handling_complexity', 0)}\n"
        
        # Save to file if path provided
        if output_path:
            mermaid_file = f"{output_path}.mmd"
            with open(mermaid_file, 'w') as f:
                f.write(mermaid_content)
            logger.info(f"Mermaid flowchart saved to {mermaid_file}")
        
        return mermaid_content
    
    def export_function_json(self, function_name: str, output_path: Optional[str] = None) -> Dict[str, Any]:
        """Export function control flow as structured JSON"""
        if function_name not in self.analyzer.function_flows:
            logger.error(f"Function '{function_name}' not found")
            return {}
        
        func_flow = self.analyzer.function_flows[function_name]
        
        # Convert to JSON-serializable format
        export_data = {
            "function_name": func_flow.name,
            "file_path": func_flow.file_path,
            "location": {
                "start_line": func_flow.start_line,
                "end_line": func_flow.end_line
            },
            "properties": {
                "is_async": func_flow.async_function,
                "return_type": func_flow.return_type
            },
            "complexity_metrics": func_flow.complexity_metrics,
            "nodes": [],
            "edges": [],
            "patterns": {
                "error_paths": func_flow.error_paths,
                "async_spawn_points": func_flow.async_spawn_points,
                "state_transitions": [{"from": t[0], "to": t[1]} for t in func_flow.state_transitions]
            },
            "analysis_summary": self._generate_function_summary(func_flow)
        }
        
        # Add nodes
        for node_id, node in func_flow.nodes.items():
            node_data = {
                "id": node_id,
                "type": node.node_type.value,
                "label": node.label,
                "source_line": node.source_line,
                "source_file": node.source_file,
                "execution_pattern": node.execution_pattern.value if node.execution_pattern else None,
                "metadata": node.metadata
            }
            export_data["nodes"].append(node_data)
        
        # Add edges
        for edge in func_flow.edges:
            edge_data = {
                "source": edge.source,
                "target": edge.target,
                "label": edge.label,
                "condition": edge.condition,
                "edge_type": edge.edge_type,
                "probability": edge.probability
            }
            export_data["edges"].append(edge_data)
        
        # Save to file if path provided
        if output_path:
            json_file = f"{output_path}.json"
            with open(json_file, 'w') as f:
                json.dump(export_data, f, indent=2)
            logger.info(f"Function JSON export saved to {json_file}")
        
        return export_data
    
    def generate_textual_summary(self, function_name: str, output_path: Optional[str] = None) -> str:
        """Generate human and LLM readable textual summary"""
        if function_name not in self.analyzer.function_flows:
            logger.error(f"Function '{function_name}' not found")
            return ""
        
        func_flow = self.analyzer.function_flows[function_name]
        
        summary_lines = [
            f"# Control Flow Analysis: {func_flow.name}",
            "",
            f"**Location:** {func_flow.file_path}:{func_flow.start_line}-{func_flow.end_line}",
            f"**Type:** {'Async Function' if func_flow.async_function else 'Sync Function'}",
            f"**Return Type:** {func_flow.return_type or 'Unknown'}",
            "",
            "## Complexity Metrics",
            f"- Cyclomatic Complexity: {func_flow.complexity_metrics.get('cyclomatic_complexity', 0)}",
            f"- Async Complexity: {func_flow.complexity_metrics.get('async_complexity', 0)}",
            f"- Error Handling Complexity: {func_flow.complexity_metrics.get('error_handling_complexity', 0)}",
            "",
            "## Control Flow Structure",
            f"- Total Nodes: {len(func_flow.nodes)}",
            f"- Total Edges: {len(func_flow.edges)}",
            "",
            "## Flow Analysis",
            self._generate_function_summary(func_flow),
            "",
            "## Architecture Notes",
            self._generate_architecture_insights(func_flow)
        ]
        
        summary_content = "\n".join(summary_lines)
        
        # Save to file if path provided
        if output_path:
            summary_file = f"{output_path}_summary.md"
            with open(summary_file, 'w') as f:
                f.write(summary_content)
            logger.info(f"Textual summary saved to {summary_file}")
        
        return summary_content
    
    def generate_interactive_html(self, function_name: str, output_path: Optional[str] = None) -> str:
        """Generate interactive HTML visualization"""
        if function_name not in self.analyzer.function_flows:
            logger.error(f"Function '{function_name}' not found")
            return ""
        
        func_flow = self.analyzer.function_flows[function_name]
        
        # Convert graph data to Cytoscape.js format
        cytoscape_data = {
            "nodes": [],
            "edges": []
        }
        
        # Add nodes
        for node_id, node in func_flow.nodes.items():
            cytoscape_data["nodes"].append({
                "data": {
                    "id": node_id,
                    "label": node.label,
                    "type": node.node_type.value,
                    "source_line": node.source_line or 0,
                    "source_file": node.source_file or "",
                    "execution_pattern": node.execution_pattern.value if node.execution_pattern else "sequential"
                }
            })
        
        # Add edges
        for edge in func_flow.edges:
            cytoscape_data["edges"].append({
                "data": {
                    "id": f"{edge.source}-{edge.target}",
                    "source": edge.source,
                    "target": edge.target,
                    "label": edge.label or "",
                    "condition": edge.condition or "",
                    "edge_type": edge.edge_type,
                    "probability": edge.probability or 1.0
                }
            })
        
        # Generate HTML template
        html_content = self._generate_html_template(function_name, cytoscape_data, func_flow)
        
        # Save to file if path provided
        if output_path:
            html_file = f"{output_path}_interactive.html"
            with open(html_file, 'w') as f:
                f.write(html_content)
            logger.info(f"Interactive HTML visualization saved to {html_file}")
        
        return html_content
    
    def generate_system_flow_graph(self, output_path: Optional[str] = None) -> str:
        """Generate system-wide control flow visualization"""
        # TODO: Implement system flow graph generation
        return "System flow graph generation not yet implemented"
    
    def generate_complexity_heatmap(self, output_path: Optional[str] = None) -> str:
        """Generate complexity heatmap visualization"""
        # TODO: Implement complexity heatmap generation
        return "Complexity heatmap generation not yet implemented"
    
    def _generate_function_summary(self, func_flow: FunctionFlow) -> str:
        """Generate a concise summary of function's control flow characteristics"""
        characteristics = []
        
        # Complexity assessment
        cyclomatic = func_flow.complexity_metrics.get('cyclomatic_complexity', 0)
        if cyclomatic > 10:
            characteristics.append("high complexity")
        elif cyclomatic > 5:
            characteristics.append("moderate complexity")
        else:
            characteristics.append("low complexity")
        
        # Async patterns
        if func_flow.async_function:
            async_complexity = func_flow.complexity_metrics.get('async_complexity', 0)
            if async_complexity > 3:
                characteristics.append("complex async coordination")
            elif async_complexity > 0:
                characteristics.append("async operations")
        
        # Error handling
        error_complexity = func_flow.complexity_metrics.get('error_handling_complexity', 0)
        if error_complexity > 3:
            characteristics.append("comprehensive error handling")
        elif error_complexity > 0:
            characteristics.append("error handling")
        
        # Control structures
        node_types = [node.node_type for node in func_flow.nodes.values()]
        if FlowNodeType.LOOP in node_types:
            characteristics.append("iterative logic")
        if FlowNodeType.CONDITION in node_types:
            characteristics.append("conditional logic")
        if FlowNodeType.STATE_TRANSITION in node_types:
            characteristics.append("state management")
        
        return f"This function exhibits {', '.join(characteristics)}."
    
    def _generate_architecture_insights(self, func_flow: FunctionFlow) -> str:
        """Generate architectural insights for LLM understanding"""
        insights = []
        
        # Function role assessment
        if func_flow.name.startswith(('init', 'new', 'create')):
            insights.append("• **Constructor/Initializer**: Sets up initial state and configuration")
        elif func_flow.name.startswith(('process', 'handle', 'execute')):
            insights.append("• **Processor/Handler**: Core business logic execution")
        elif func_flow.name.startswith(('validate', 'check', 'verify')):
            insights.append("• **Validator**: Input validation and verification logic")
        
        # Architectural patterns
        if func_flow.async_spawn_points:
            insights.append("• **Concurrency Pattern**: Spawns async tasks for parallel execution")
        
        if len(func_flow.error_paths) > 2:
            insights.append("• **Error Resilience**: Multiple error handling strategies")
        
        if func_flow.state_transitions:
            insights.append("• **State Machine**: Manages state transitions and lifecycle")
        
        return "\n".join(insights) if insights else "• **Simple Function**: Straightforward control flow"
    
    def _generate_html_template(self, function_name: str, cytoscape_data: Dict[str, Any], func_flow: FunctionFlow) -> str:
        """Generate the complete HTML template for interactive visualization"""
        metrics = func_flow.complexity_metrics
        
        html_template = f'''<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Control Flow: {function_name}</title>
    <script src="https://unpkg.com/cytoscape@3.23.0/dist/cytoscape.min.js"></script>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background: #f5f5f5;
        }}
        
        .header {{
            background: #2c3e50;
            color: white;
            padding: 1rem;
            border-radius: 8px;
            margin-bottom: 20px;
        }}
        
        .function-title {{
            font-size: 1.5rem;
            font-weight: 600;
            margin: 0 0 0.5rem 0;
        }}
        
        .function-meta {{
            font-size: 0.9rem;
            opacity: 0.8;
        }}
        
        .metrics {{
            background: #ecf0f1;
            padding: 1rem;
            border-radius: 8px;
            margin-bottom: 20px;
        }}
        
        .metrics h3 {{
            margin: 0 0 0.5rem 0;
            color: #2c3e50;
        }}
        
        .metric-item {{
            display: inline-block;
            margin-right: 20px;
            margin-bottom: 10px;
        }}
        
        #cy {{
            width: 100%;
            height: 600px;
            background: white;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        
        .controls {{
            margin-top: 20px;
            text-align: center;
        }}
        
        .controls button {{
            background: #3498db;
            color: white;
            border: none;
            padding: 0.5rem 1rem;
            border-radius: 4px;
            cursor: pointer;
            margin: 0 5px;
        }}
        
        .controls button:hover {{
            background: #2980b9;
        }}
    </style>
</head>
<body>
    <div class="header">
        <div class="function-title">{function_name}</div>
        <div class="function-meta">
            {func_flow.file_path}:{func_flow.start_line}-{func_flow.end_line} | 
            {'Async' if func_flow.async_function else 'Sync'} Function
        </div>
    </div>
    
    <div class="metrics">
        <h3>Complexity Metrics</h3>
        <div class="metric-item">
            <strong>Cyclomatic:</strong> {metrics.get('cyclomatic_complexity', 0)}
        </div>
        <div class="metric-item">
            <strong>Async:</strong> {metrics.get('async_complexity', 0)}
        </div>
        <div class="metric-item">
            <strong>Error Handling:</strong> {metrics.get('error_handling_complexity', 0)}
        </div>
        <div class="metric-item">
            <strong>Nodes:</strong> {len(func_flow.nodes)}
        </div>
        <div class="metric-item">
            <strong>Edges:</strong> {len(func_flow.edges)}
        </div>
    </div>
    
    <div id="cy"></div>
    
    <div class="controls">
        <button onclick="cy.fit()">Fit to View</button>
        <button onclick="cy.zoom(1); cy.center();">Reset Zoom</button>
        <button onclick="setLayout('dagre')">Hierarchical</button>
        <button onclick="setLayout('breadthfirst')">Breadth First</button>
        <button onclick="setLayout('circle')">Circle</button>
    </div>

    <script>
        // Graph data
        const graphData = {json.dumps(cytoscape_data, indent=2)};
        
        // Initialize Cytoscape
        const cy = cytoscape({{
            container: document.getElementById('cy'),
            elements: graphData.nodes.concat(graphData.edges),
            
            style: [
                {{
                    selector: 'node',
                    style: {{
                        'content': 'data(label)',
                        'text-wrap': 'wrap',
                        'text-max-width': '120px',
                        'font-size': '10px',
                        'text-valign': 'center',
                        'text-halign': 'center',
                        'background-color': function(ele) {{
                            const nodeType = ele.data('type');
                            const colors = {{
                                'entry': '#4CAF50',
                                'exit': '#F44336',
                                'statement': '#E3F2FD',
                                'condition': '#FFF59D',
                                'loop': '#FFECB3',
                                'async_point': '#E1BEE7',
                                'await_point': '#CE93D8',
                                'spawn_point': '#BA68C8',
                                'error_handler': '#FFCDD2',
                                'state_transition': '#C8E6C9',
                                'function_call': '#BBDEFB',
                                'return_point': '#FFE0B2'
                            }};
                            return colors[nodeType] || '#E0E0E0';
                        }},
                        'border-width': 2,
                        'border-color': '#34495e',
                        'width': 60,
                        'height': 40,
                        'shape': function(ele) {{
                            const nodeType = ele.data('type');
                            if (nodeType === 'condition') return 'diamond';
                            if (nodeType === 'entry' || nodeType === 'exit') return 'round-rectangle';
                            return 'rectangle';
                        }}
                    }}
                }},
                {{
                    selector: 'edge',
                    style: {{
                        'width': 2,
                        'line-color': function(ele) {{
                            const edgeType = ele.data('edge_type');
                            const colors = {{
                                'control': '#34495e',
                                'async': '#9b59b6',
                                'error': '#e74c3c',
                                'data': '#3498db'
                            }};
                            return colors[edgeType] || '#34495e';
                        }},
                        'target-arrow-color': function(ele) {{
                            const edgeType = ele.data('edge_type');
                            const colors = {{
                                'control': '#34495e',
                                'async': '#9b59b6',
                                'error': '#e74c3c',
                                'data': '#3498db'
                            }};
                            return colors[edgeType] || '#34495e';
                        }},
                        'target-arrow-shape': 'triangle',
                        'curve-style': 'bezier'
                    }}
                }}
            ],
            
            layout: {{
                name: 'dagre',
                rankDir: 'TB',
                padding: 30
            }}
        }});
        
        // Functions
        function setLayout(layoutName) {{
            const layouts = {{
                'dagre': {{ name: 'dagre', rankDir: 'TB', padding: 30 }},
                'breadthfirst': {{ name: 'breadthfirst', directed: true, padding: 30 }},
                'circle': {{ name: 'circle', padding: 30 }}
            }};
            
            cy.layout(layouts[layoutName]).run();
        }}
        
        // Initial layout
        cy.ready(function() {{
            cy.fit();
        }});
    </script>
</body>
</html>'''
        
        return html_template
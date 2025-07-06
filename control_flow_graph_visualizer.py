#!/usr/bin/env python3
"""
Toka Control Flow Graph Visualizer

This tool creates control flow graph visualizations for the Toka codebase:
- Function-level control flow graphs with Rust-specific patterns
- System-level execution flow between components
- Async/await pattern analysis
- State machine visualizations  
- Error handling flow analysis
- Interactive filtering and exploration capabilities

Generated on: 2025-01-09
"""

import os
import re
import json
import asyncio
import aiofiles
from pathlib import Path
from typing import Dict, List, Set, Optional, Tuple, Union
from dataclasses import dataclass, field
from collections import defaultdict, deque
from enum import Enum
import graphviz
from concurrent.futures import ThreadPoolExecutor, as_completed
import argparse
import logging

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
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
    metadata: Dict = field(default_factory=dict)
    execution_pattern: Optional[ExecutionPattern] = None

@dataclass
class FlowEdge:
    """An edge in the control flow graph"""
    source: str
    target: str
    label: Optional[str] = None
    condition: Optional[str] = None
    edge_type: str = "control"  # control, data, async, error
    probability: Optional[float] = None  # estimated execution probability

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
    complexity_metrics: Dict = field(default_factory=dict)

@dataclass
class SystemFlow:
    """System-wide control flow patterns"""
    component_interactions: Dict[str, List[str]] = field(default_factory=dict)
    async_coordination_patterns: List[Dict] = field(default_factory=list)
    error_propagation_chains: List[List[str]] = field(default_factory=list)
    state_machine_flows: Dict[str, Dict] = field(default_factory=dict)
    orchestration_sequences: List[Dict] = field(default_factory=list)

class RustPatternAnalyzer:
    """Analyzes Rust-specific control flow patterns"""
    
    def __init__(self):
        # Rust-specific patterns for control flow analysis
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

    def analyze_function_patterns(self, source_code: str, function_name: str) -> Dict:
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
    
    def __init__(self, workspace_path: str):
        self.workspace_path = Path(workspace_path)
        self.pattern_analyzer = RustPatternAnalyzer()
        self.function_flows: Dict[str, FunctionFlow] = {}
        self.system_flow = SystemFlow()
        self.rust_files: List[Path] = []
        
    async def analyze_workspace(self) -> None:
        """Analyze control flow patterns across the workspace"""
        logger.info("Starting control flow analysis...")
        
        # Find all Rust source files
        await self._discover_rust_files()
        
        # Analyze function-level control flows
        await self._analyze_function_flows()
        
        # Analyze system-level patterns
        await self._analyze_system_patterns()
        
        # Compute complexity metrics
        self._compute_complexity_metrics()
        
        logger.info(f"Control flow analysis complete. Analyzed {len(self.function_flows)} functions across {len(self.rust_files)} files.")
    
    async def _discover_rust_files(self) -> None:
        """Discover all Rust source files in the workspace"""
        for root in ['crates', 'src', 'tests']:
            root_path = self.workspace_path / root
            if root_path.exists():
                for rust_file in root_path.rglob('*.rs'):
                    if not any(skip in str(rust_file) for skip in ['target', '.git', 'build.rs']):
                        self.rust_files.append(rust_file)
        
        logger.info(f"Discovered {len(self.rust_files)} Rust source files")
    
    async def _analyze_function_flows(self) -> None:
        """Analyze control flow within individual functions"""
        with ThreadPoolExecutor(max_workers=8) as executor:
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
        
        # Simple regex-based function extraction (could be enhanced with proper parsing)
        for i, line in enumerate(lines):
            # Match function definitions
            func_match = re.match(r'\s*(pub\s+)?(async\s+)?fn\s+(\w+)\s*\(', line)
            if func_match:
                func_name = func_match.group(3)
                start_line = i + 1
                
                # Find function end (simplified - looks for closing brace at same indentation)
                indent_level = len(line) - len(line.lstrip())
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
        node_counter = 0
        
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
        
        # Analyze line by line for control structures
        for i, line in enumerate(lines):
            line_stripped = line.strip()
            if not line_stripped or line_stripped.startswith('//'):
                continue
            
            node_id = f"{func.name}_{i}"
            
            # Identify node type based on content
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
                
                # Special handling for different edge types
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
            
            # Find function calls and their target components
            for edge in func_flow.edges:
                if edge.edge_type == "control":
                    target_node = func_flow.nodes.get(edge.target)
                    if target_node and target_node.node_type == FlowNodeType.FUNCTION_CALL:
                        # Extract called function name (simplified)
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
        # Implementation for error chain analysis
        pass
    
    async def _analyze_state_machines(self) -> None:
        """Analyze state machine patterns"""
        # Implementation for state machine analysis
        pass
    
    def _compute_complexity_metrics(self) -> None:
        """Compute complexity metrics for each function"""
        for func_name, func_flow in self.function_flows.items():
            # Cyclomatic complexity (simplified)
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
    
    def generate_function_cfg(self, function_name: str, output_path: str = None) -> None:
        """Generate control flow graph for a specific function"""
        if function_name not in self.analyzer.function_flows:
            logger.error(f"Function '{function_name}' not found")
            return
        
        func_flow = self.analyzer.function_flows[function_name]
        
        dot = graphviz.Digraph(comment=f'Control Flow: {function_name}')
        dot.attr(rankdir='TB', size='12,16', dpi='300')
        dot.attr('node', shape='box', style='filled,rounded')
        
        # Add nodes
        for node_id, node in func_flow.nodes.items():
            color = self.colors.get(node.node_type, '#E0E0E0')
            
            # Create detailed label
            label = f"{node.label}\\l"
            if node.source_line:
                label += f"Line: {node.source_line}\\l"
            if node.execution_pattern:
                label += f"Pattern: {node.execution_pattern.value}\\l"
            
            dot.node(node_id, label=label, fillcolor=color, fontsize='10')
        
        # Add edges
        for edge in func_flow.edges:
            color = self.edge_colors.get(edge.edge_type, 'black')
            style = 'dashed' if edge.edge_type == 'async' else 'solid'
            
            edge_label = edge.label or ''
            if edge.condition:
                edge_label += f"\\n[{edge.condition}]"
            
            dot.edge(edge.source, edge.target, label=edge_label, 
                    color=color, style=style, fontsize='8')
        
        # Add complexity info
        metrics = func_flow.complexity_metrics
        info_label = f"Complexity Metrics:\\l"
        info_label += f"Cyclomatic: {metrics.get('cyclomatic_complexity', 0)}\\l"
        info_label += f"Async: {metrics.get('async_complexity', 0)}\\l"
        info_label += f"Error Handling: {metrics.get('error_handling_complexity', 0)}\\l"
        
        dot.node('metrics', label=info_label, shape='note', fillcolor='lightyellow')
        
        # Render
        if not output_path:
            output_path = f"cfg_{function_name}"
        
        dot.render(output_path, format='png', cleanup=True)
        dot.render(output_path, format='svg', cleanup=True)
        logger.info(f"Function CFG saved to {output_path}")
    
    def generate_system_flow_graph(self, output_path: str = "system_control_flow") -> None:
        """Generate system-wide control flow visualization"""
        dot = graphviz.Digraph(comment='System Control Flow')
        dot.attr(rankdir='TB', size='20,16', dpi='300')
        dot.attr('node', shape='box', style='filled,rounded')
        
        # Add component nodes
        for component, targets in self.analyzer.system_flow.component_interactions.items():
            color = self._get_component_color(component)
            dot.node(component, label=f"{component}\\nComponent", 
                    fillcolor=color, fontsize='12', penwidth='2')
        
        # Add interaction edges
        for source, targets in self.analyzer.system_flow.component_interactions.items():
            for target in targets:
                dot.edge(source, target, color='blue', penwidth='2')
        
        # Add async patterns cluster
        if self.analyzer.system_flow.async_coordination_patterns:
            with dot.subgraph(name='cluster_async') as async_cluster:
                async_cluster.attr(label='Async Coordination Patterns', 
                                 style='filled', fillcolor='lightpurple')
                
                for i, pattern in enumerate(self.analyzer.system_flow.async_coordination_patterns[:10]):
                    pattern_id = f"async_pattern_{i}"
                    label = f"{pattern['function']}\\n{pattern['pattern_type']}"
                    async_cluster.node(pattern_id, label=label, 
                                     fillcolor='mediumpurple', fontsize='10')
        
        # Render
        dot.render(output_path, format='png', cleanup=True)
        dot.render(output_path, format='svg', cleanup=True)
        logger.info(f"System control flow graph saved to {output_path}")
    
    def generate_async_coordination_graph(self, output_path: str = "async_coordination") -> None:
        """Generate async coordination patterns visualization"""
        dot = graphviz.Digraph(comment='Async Coordination Patterns')
        dot.attr(rankdir='TB', size='16,12', dpi='300')
        dot.attr('node', shape='ellipse', style='filled')
        
        patterns = self.analyzer.system_flow.async_coordination_patterns
        
        # Group patterns by type
        pattern_groups = defaultdict(list)
        for pattern in patterns:
            pattern_groups[pattern['pattern_type']].append(pattern)
        
        # Create subgraphs for each pattern type
        for pattern_type, type_patterns in pattern_groups.items():
            with dot.subgraph(name=f'cluster_{pattern_type}') as cluster:
                cluster.attr(label=f'{pattern_type.replace("_", " ").title()}', 
                           style='filled', fillcolor='lightblue')
                
                for i, pattern in enumerate(type_patterns[:5]):  # Limit to 5 per type
                    pattern_id = f"{pattern_type}_{i}"
                    func_name = pattern['function']
                    await_count = len(pattern['await_points'])
                    spawn_count = len(pattern['spawn_points'])
                    
                    label = f"{func_name}\\nAwaits: {await_count}\\nSpawns: {spawn_count}"
                    cluster.node(pattern_id, label=label, fillcolor='lightcyan')
        
        # Render
        dot.render(output_path, format='png', cleanup=True)
        dot.render(output_path, format='svg', cleanup=True)
        logger.info(f"Async coordination graph saved to {output_path}")
    
    def generate_complexity_heatmap(self, output_path: str = "complexity_heatmap") -> None:
        """Generate complexity heatmap visualization"""
        dot = graphviz.Digraph(comment='Function Complexity Heatmap')
        dot.attr(rankdir='TB', size='20,16', dpi='300')
        dot.attr('node', shape='box', style='filled')
        
        # Sort functions by complexity
        sorted_functions = sorted(
            self.analyzer.function_flows.items(),
            key=lambda x: x[1].complexity_metrics.get('cyclomatic_complexity', 0),
            reverse=True
        )
        
        # Create complexity ranges
        max_complexity = max(
            func.complexity_metrics.get('cyclomatic_complexity', 0)
            for func in self.analyzer.function_flows.values()
        ) if self.analyzer.function_flows else 1
        
        # Add top 20 most complex functions
        for i, (func_name, func_flow) in enumerate(sorted_functions[:20]):
            metrics = func_flow.complexity_metrics
            complexity = metrics.get('cyclomatic_complexity', 0)
            
            # Color based on complexity
            intensity = min(complexity / max_complexity, 1.0)
            color = self._get_complexity_color(intensity)
            
            label = f"{func_name}\\n"
            label += f"Cyclomatic: {complexity}\\n"
            label += f"Async: {metrics.get('async_complexity', 0)}\\n"
            label += f"Error: {metrics.get('error_handling_complexity', 0)}"
            
            dot.node(f"func_{i}", label=label, fillcolor=color, fontsize='10')
        
        # Add legend
        with dot.subgraph(name='cluster_legend') as legend:
            legend.attr(label='Complexity Legend', style='filled', fillcolor='lightgray')
            legend.node('low', label='Low\\nComplexity', fillcolor='lightgreen')
            legend.node('medium', label='Medium\\nComplexity', fillcolor='yellow')
            legend.node('high', label='High\\nComplexity', fillcolor='orange')
            legend.node('very_high', label='Very High\\nComplexity', fillcolor='red')
        
        # Render
        dot.render(output_path, format='png', cleanup=True)
        dot.render(output_path, format='svg', cleanup=True)
        logger.info(f"Complexity heatmap saved to {output_path}")
    
    def _get_component_color(self, component: str) -> str:
        """Get color for a system component"""
        colors = {
            'kernel': '#FF6B6B',
            'runtime': '#4ECDC4', 
            'orchestration': '#45B7D1',
            'storage': '#96CEB4',
            'bus': '#FFEAA7',
            'auth': '#DDA0DD',
            'llm': '#98D8C8',
            'cli': '#F7DC6F',
        }
        return colors.get(component, '#D5DBDB')
    
    def _get_complexity_color(self, intensity: float) -> str:
        """Get color based on complexity intensity"""
        if intensity < 0.25:
            return 'lightgreen'
        elif intensity < 0.5:
            return 'yellow'
        elif intensity < 0.75:
            return 'orange'
        else:
            return 'red'

async def main():
    """Main function to run control flow analysis and visualization"""
    parser = argparse.ArgumentParser(description='Toka Control Flow Graph Visualizer')
    parser.add_argument('--workspace', default='.', help='Workspace path (default: current directory)')
    parser.add_argument('--output-dir', default='control_flow_graphs', help='Output directory for graphs')
    parser.add_argument('--function', help='Generate CFG for specific function')
    parser.add_argument('--system', action='store_true', help='Generate system-wide flow graph')
    parser.add_argument('--async-patterns', action='store_true', help='Generate async coordination graph')
    parser.add_argument('--complexity', action='store_true', help='Generate complexity heatmap')
    parser.add_argument('--all', action='store_true', help='Generate all graphs')
    
    args = parser.parse_args()
    
    if args.all:
        args.system = True
        setattr(args, 'async_patterns', True)
        args.complexity = True
    
    # Create output directory
    os.makedirs(args.output_dir, exist_ok=True)
    
    # Initialize analyzer
    analyzer = ControlFlowAnalyzer(args.workspace)
    
    # Analyze workspace
    await analyzer.analyze_workspace()
    
    # Generate visualizations
    visualizer = ControlFlowVisualizer(analyzer)
    
    if args.function:
        output_path = os.path.join(args.output_dir, f"cfg_{args.function}")
        visualizer.generate_function_cfg(args.function, output_path)
    
    if args.system:
        system_path = os.path.join(args.output_dir, "system_control_flow")
        visualizer.generate_system_flow_graph(system_path)
    
    if getattr(args, 'async_patterns', False):
        async_path = os.path.join(args.output_dir, "async_coordination")
        visualizer.generate_async_coordination_graph(async_path)
    
    if args.complexity:
        complexity_path = os.path.join(args.output_dir, "complexity_heatmap")
        visualizer.generate_complexity_heatmap(complexity_path)
    
    # Generate summary report
    await generate_control_flow_report(analyzer, args.output_dir)
    
    logger.info("Control flow analysis and visualization complete!")

async def generate_control_flow_report(analyzer: ControlFlowAnalyzer, output_dir: str):
    """Generate a summary report of the control flow analysis"""
    report_path = os.path.join(output_dir, "control_flow_analysis_report.md")
    
    with open(report_path, 'w') as f:
        f.write("# Toka Control Flow Analysis Report\n\n")
        f.write(f"Generated on: {os.popen('date').read().strip()}\n\n")
        
        f.write("## Overview\n\n")
        f.write(f"- Total functions analyzed: {len(analyzer.function_flows)}\n")
        f.write(f"- Total source files: {len(analyzer.rust_files)}\n")
        f.write(f"- Async functions: {sum(1 for f in analyzer.function_flows.values() if f.async_function)}\n")
        f.write(f"- Component interactions: {len(analyzer.system_flow.component_interactions)}\n\n")
        
        # Complexity analysis
        f.write("## Complexity Analysis\n\n")
        complexities = [f.complexity_metrics.get('cyclomatic_complexity', 0) 
                       for f in analyzer.function_flows.values()]
        if complexities:
            f.write(f"- Average cyclomatic complexity: {sum(complexities) / len(complexities):.2f}\n")
            f.write(f"- Max cyclomatic complexity: {max(complexities)}\n")
            f.write(f"- Functions with complexity > 10: {sum(1 for c in complexities if c > 10)}\n\n")
        
        # Top complex functions
        f.write("## Most Complex Functions\n\n")
        sorted_funcs = sorted(
            analyzer.function_flows.items(),
            key=lambda x: x[1].complexity_metrics.get('cyclomatic_complexity', 0),
            reverse=True
        )
        
        for func_name, func_flow in sorted_funcs[:10]:
            complexity = func_flow.complexity_metrics.get('cyclomatic_complexity', 0)
            f.write(f"- **{func_name}**: {complexity} (in {func_flow.file_path})\n")
        
        f.write("\n## Async Patterns\n\n")
        pattern_counts = defaultdict(int)
        for pattern in analyzer.system_flow.async_coordination_patterns:
            pattern_counts[pattern['pattern_type']] += 1
        
        for pattern_type, count in pattern_counts.items():
            f.write(f"- {pattern_type.replace('_', ' ').title()}: {count} functions\n")
        
        f.write("\n## Component Interactions\n\n")
        for component, targets in analyzer.system_flow.component_interactions.items():
            f.write(f"- **{component}** interacts with: {', '.join(targets)}\n")
    
    logger.info(f"Control flow report saved to {report_path}")

if __name__ == "__main__":
    asyncio.run(main()) 
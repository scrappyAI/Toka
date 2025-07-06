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

    def generate_mermaid_flowchart(self, function_name: str, output_path: str = None) -> str:
        """Generate Mermaid flowchart for a function - LLM and documentation friendly"""
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

    def export_function_json(self, function_name: str, output_path: str = None) -> dict:
        """Export function control flow as structured JSON for LLM consumption"""
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
                import json
                json.dump(export_data, f, indent=2)
            logger.info(f"Function JSON export saved to {json_file}")
        
        return export_data

    def generate_textual_summary(self, function_name: str, output_path: str = None) -> str:
        """Generate human and LLM readable textual summary of control flow"""
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
            ""
        ]
        
        # Node type distribution
        node_types = defaultdict(int)
        for node in func_flow.nodes.values():
            node_types[node.node_type.value] += 1
        
        summary_lines.append("### Node Distribution")
        for node_type, count in sorted(node_types.items()):
            summary_lines.append(f"- {node_type.replace('_', ' ').title()}: {count}")
        summary_lines.append("")
        
        # Control flow patterns
        if func_flow.async_spawn_points:
            summary_lines.extend([
                "### Async Patterns",
                f"- Spawn Points: {len(func_flow.async_spawn_points)}",
                f"- Spawn Locations: {', '.join(func_flow.async_spawn_points)}",
                ""
            ])
        
        if func_flow.error_paths:
            summary_lines.extend([
                "### Error Handling",
                f"- Error Paths: {len(func_flow.error_paths)}",
                f"- Error Handlers: {', '.join(func_flow.error_paths)}",
                ""
            ])
        
        if func_flow.state_transitions:
            summary_lines.extend([
                "### State Transitions",
                f"- Transition Count: {len(func_flow.state_transitions)}",
            ])
            for from_state, to_state in func_flow.state_transitions:
                summary_lines.append(f"- {from_state} → {to_state}")
            summary_lines.append("")
        
        # Flow analysis
        summary_lines.extend([
            "## Flow Analysis",
            self._generate_function_summary(func_flow),
            "",
            "## Architecture Notes",
            self._generate_architecture_insights(func_flow)
        ])
        
        summary_content = "\n".join(summary_lines)
        
        # Save to file if path provided
        if output_path:
            summary_file = f"{output_path}_summary.md"
            with open(summary_file, 'w') as f:
                f.write(summary_content)
            logger.info(f"Textual summary saved to {summary_file}")
        
        return summary_content

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
        elif func_flow.name.startswith(('send', 'publish', 'emit')):
            insights.append("• **Publisher**: Event/message distribution")
        elif func_flow.name.startswith(('receive', 'listen', 'on_')):
            insights.append("• **Subscriber/Listener**: Event/message consumption")
        
        # Architectural patterns
        if func_flow.async_spawn_points:
            insights.append("• **Concurrency Pattern**: Spawns async tasks for parallel execution")
        
        if len(func_flow.error_paths) > 2:
            insights.append("• **Error Resilience**: Multiple error handling strategies")
        
        if func_flow.state_transitions:
            insights.append("• **State Machine**: Manages state transitions and lifecycle")
        
        cyclomatic = func_flow.complexity_metrics.get('cyclomatic_complexity', 0)
        if cyclomatic > 15:
            insights.append("• **Complexity Warning**: High complexity may indicate need for refactoring")
        
        return "\n".join(insights) if insights else "• **Simple Function**: Straightforward control flow"

    def export_system_json(self, output_path: str = "system_control_flow") -> dict:
        """Export system-wide control flow as structured JSON"""
        export_data = {
            "system_overview": {
                "total_functions": len(self.analyzer.function_flows),
                "total_components": len(self.analyzer.system_flow.component_interactions),
                "async_patterns": len(self.analyzer.system_flow.async_coordination_patterns),
                "error_chains": len(self.analyzer.system_flow.error_propagation_chains)
            },
            "component_interactions": dict(self.analyzer.system_flow.component_interactions),
            "async_coordination_patterns": self.analyzer.system_flow.async_coordination_patterns,
            "error_propagation_chains": self.analyzer.system_flow.error_propagation_chains,
            "state_machine_flows": self.analyzer.system_flow.state_machine_flows,
            "orchestration_sequences": self.analyzer.system_flow.orchestration_sequences,
            "complexity_distribution": self._generate_complexity_distribution(),
            "architectural_insights": self._generate_system_insights()
        }
        
        # Save to file
        json_file = f"{output_path}.json"
        with open(json_file, 'w') as f:
            import json
            json.dump(export_data, f, indent=2)
        logger.info(f"System JSON export saved to {json_file}")
        
        return export_data

    def _generate_complexity_distribution(self) -> dict:
        """Generate complexity distribution across the system"""
        complexities = []
        for func_flow in self.analyzer.function_flows.values():
            complexities.append(func_flow.complexity_metrics.get('cyclomatic_complexity', 0))
        
        if not complexities:
            return {}
        
        return {
            "mean": sum(complexities) / len(complexities),
            "max": max(complexities),
            "min": min(complexities),
            "high_complexity_functions": [
                func.name for func in self.analyzer.function_flows.values()
                if func.complexity_metrics.get('cyclomatic_complexity', 0) > 10
            ]
        }

    def _generate_system_insights(self) -> list:
        """Generate system-level architectural insights"""
        insights = []
        
        # Component interaction analysis
        interactions = self.analyzer.system_flow.component_interactions
        if len(interactions) > 5:
            insights.append("Complex multi-component architecture with extensive inter-component communication")
        
        # Async pattern analysis
        async_patterns = self.analyzer.system_flow.async_coordination_patterns
        if len(async_patterns) > 10:
            insights.append("Heavy async coordination suggesting event-driven or reactive architecture")
        
        # Error handling analysis
        error_chains = self.analyzer.system_flow.error_propagation_chains
        if len(error_chains) > 5:
            insights.append("Comprehensive error propagation indicating robust error handling strategy")
        
        return insights

    def generate_interactive_html(self, function_name: str, output_path: str = None) -> str:
        """Generate interactive HTML visualization using Cytoscape.js"""
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

    def _generate_html_template(self, function_name: str, cytoscape_data: dict, func_flow: FunctionFlow) -> str:
        """Generate the complete HTML template for interactive visualization"""
        
        # Convert cytoscape_data to JSON string for embedding
        import json
        graph_data_json = json.dumps(cytoscape_data, indent=2)
        metrics = func_flow.complexity_metrics
        
        html_template = f'''<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Control Flow: {function_name}</title>
    <script src="https://unpkg.com/cytoscape@3.23.0/dist/cytoscape.min.js"></script>
    <script src="https://unpkg.com/dagre@0.8.5/dist/dagre.min.js"></script>
    <script src="https://unpkg.com/cytoscape-dagre@2.5.0/cytoscape-dagre.js"></script>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 0;
            background: #f5f5f5;
        }}
        
        .header {{
            background: #2c3e50;
            color: white;
            padding: 1rem;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
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
        
        .container {{
            display: flex;
            height: calc(100vh - 80px);
        }}
        
        .sidebar {{
            width: 300px;
            background: white;
            border-right: 1px solid #ddd;
            overflow-y: auto;
            padding: 1rem;
        }}
        
        .graph-container {{
            flex: 1;
            position: relative;
        }}
        
        #cy {{
            width: 100%;
            height: 100%;
            background: white;
        }}
        
        .controls {{
            background: white;
            padding: 1rem;
            border-bottom: 1px solid #ddd;
        }}
        
        .control-group {{
            margin-bottom: 1rem;
        }}
        
        .control-group label {{
            display: block;
            font-weight: 500;
            margin-bottom: 0.25rem;
            color: #2c3e50;
        }}
        
        .control-group button {{
            background: #3498db;
            color: white;
            border: none;
            padding: 0.5rem 1rem;
            border-radius: 4px;
            cursor: pointer;
            margin-right: 0.5rem;
            margin-bottom: 0.5rem;
        }}
        
        .control-group button:hover {{
            background: #2980b9;
        }}
        
        .control-group button.active {{
            background: #e74c3c;
        }}
        
        .metrics {{
            background: #ecf0f1;
            padding: 1rem;
            border-radius: 4px;
            margin-bottom: 1rem;
        }}
        
        .metrics h3 {{
            margin: 0 0 0.5rem 0;
            color: #2c3e50;
        }}
        
        .metric-item {{
            display: flex;
            justify-content: space-between;
            margin-bottom: 0.25rem;
        }}
        
        .node-info {{
            background: #f8f9fa;
            border: 1px solid #dee2e6;
            border-radius: 4px;
            padding: 1rem;
            margin-top: 1rem;
            display: none;
        }}
        
        .node-info h4 {{
            margin: 0 0 0.5rem 0;
            color: #2c3e50;
        }}
        
        .legend {{
            margin-top: 1rem;
        }}
        
        .legend h3 {{
            margin: 0 0 0.5rem 0;
            color: #2c3e50;
        }}
        
        .legend-item {{
            display: flex;
            align-items: center;
            margin-bottom: 0.25rem;
        }}
        
        .legend-color {{
            width: 16px;
            height: 16px;
            border-radius: 50%;
            margin-right: 0.5rem;
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
    
    <div class="container">
        <div class="sidebar">
            <div class="metrics">
                <h3>Complexity Metrics</h3>
                <div class="metric-item">
                    <span>Cyclomatic:</span>
                    <strong>{metrics.get('cyclomatic_complexity', 0)}</strong>
                </div>
                <div class="metric-item">
                    <span>Async:</span>
                    <strong>{metrics.get('async_complexity', 0)}</strong>
                </div>
                <div class="metric-item">
                    <span>Error Handling:</span>
                    <strong>{metrics.get('error_handling_complexity', 0)}</strong>
                </div>
            </div>
            
            <div class="controls">
                <div class="control-group">
                    <label>Layout</label>
                    <button onclick="setLayout('dagre')">Hierarchical</button>
                    <button onclick="setLayout('breadthfirst')">Breadth First</button>
                    <button onclick="setLayout('circle')">Circle</button>
                </div>
                
                <div class="control-group">
                    <label>Filter</label>
                    <button onclick="filterNodes('all')" class="active">All</button>
                    <button onclick="filterNodes('entry')">Entry/Exit</button>
                    <button onclick="filterNodes('condition')">Conditions</button>
                    <button onclick="filterNodes('async')">Async</button>
                    <button onclick="filterNodes('error')">Errors</button>
                </div>
                
                <div class="control-group">
                    <label>View</label>
                    <button onclick="cy.fit()">Fit to View</button>
                    <button onclick="cy.zoom(1); cy.center();">Reset Zoom</button>
                </div>
            </div>
            
            <div class="legend">
                <h3>Node Types</h3>
                <div class="legend-item">
                    <div class="legend-color" style="background: #4CAF50;"></div>
                    <span>Entry Point</span>
                </div>
                <div class="legend-item">
                    <div class="legend-color" style="background: #F44336;"></div>
                    <span>Exit Point</span>
                </div>
                <div class="legend-item">
                    <div class="legend-color" style="background: #FFF59D;"></div>
                    <span>Condition</span>
                </div>
                <div class="legend-item">
                    <div class="legend-color" style="background: #FFECB3;"></div>
                    <span>Loop</span>
                </div>
                <div class="legend-item">
                    <div class="legend-color" style="background: #E1BEE7;"></div>
                    <span>Async Point</span>
                </div>
                <div class="legend-item">
                    <div class="legend-color" style="background: #FFCDD2;"></div>
                    <span>Error Handler</span>
                </div>
            </div>
            
            <div class="node-info" id="nodeInfo">
                <h4>Node Information</h4>
                <div id="nodeDetails"></div>
            </div>
        </div>
        
        <div class="graph-container">
            <div id="cy"></div>
        </div>
    </div>

    <script>
        // Graph data
        const graphData = {graph_data_json};
        
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
                        'width': function(ele) {{
                            return Math.max(60, ele.data('label').length * 6);
                        }},
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
                        'width': function(ele) {{
                            const edgeType = ele.data('edge_type');
                            return edgeType === 'error' ? 3 : 2;
                        }},
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
                        'line-style': function(ele) {{
                            return ele.data('edge_type') === 'async' ? 'dashed' : 'solid';
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
                        'curve-style': 'bezier',
                        'content': function(ele) {{
                            const label = ele.data('label');
                            const condition = ele.data('condition');
                            if (label && condition) return label + ' [' + condition + ']';
                            return label || condition || '';
                        }},
                        'font-size': '8px',
                        'text-rotation': 'autorotate',
                        'text-margin-y': -8
                    }}
                }},
                {{
                    selector: 'node:selected',
                    style: {{
                        'border-width': 4,
                        'border-color': '#e74c3c'
                    }}
                }},
                {{
                    selector: '.highlighted',
                    style: {{
                        'background-color': '#f39c12',
                        'border-color': '#e67e22'
                    }}
                }},
                {{
                    selector: '.filtered',
                    style: {{
                        'opacity': 0.3
                    }}
                }}
            ],
            
            layout: {{
                name: 'dagre',
                rankDir: 'TB',
                padding: 30,
                spacingFactor: 1.5
            }}
        }});
        
        // Event handlers
        cy.on('tap', 'node', function(evt) {{
            const node = evt.target;
            showNodeInfo(node);
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
        
        function filterNodes(filterType) {{
            // Remove active class from all buttons
            document.querySelectorAll('.control-group button').forEach(btn => {{
                btn.classList.remove('active');
            }});
            
            // Add active class to clicked button
            event.target.classList.add('active');
            
            // Reset all nodes
            cy.nodes().removeClass('filtered');
            
            if (filterType !== 'all') {{
                cy.nodes().forEach(node => {{
                    const nodeType = node.data('type');
                    let shouldShow = false;
                    
                    switch(filterType) {{
                        case 'entry':
                            shouldShow = nodeType === 'entry' || nodeType === 'exit';
                            break;
                        case 'condition':
                            shouldShow = nodeType === 'condition';
                            break;
                        case 'async':
                            shouldShow = nodeType.includes('async') || nodeType.includes('await') || nodeType.includes('spawn');
                            break;
                        case 'error':
                            shouldShow = nodeType === 'error_handler';
                            break;
                    }}
                    
                    if (!shouldShow) {{
                        node.addClass('filtered');
                    }}
                }});
            }}
        }}
        
        function showNodeInfo(node) {{
            const nodeInfo = document.getElementById('nodeInfo');
            const nodeDetails = document.getElementById('nodeDetails');
            
            const data = node.data();
            const details = `
                <p><strong>Type:</strong> ${{data.type.replace('_', ' ')}}</p>
                <p><strong>Label:</strong> ${{data.label}}</p>
                <p><strong>Source Line:</strong> ${{data.source_line || 'N/A'}}</p>
                <p><strong>Execution Pattern:</strong> ${{data.execution_pattern || 'sequential'}}</p>
            `;
            
            nodeDetails.innerHTML = details;
            nodeInfo.style.display = 'block';
        }}
        
        // Initial layout
        cy.ready(function() {{
            cy.fit();
        }});
    </script>
</body>
</html>'''
        
        return html_template

    def generate_system_interactive_html(self, output_path: str = "system_control_flow") -> str:
        """Generate interactive HTML for system-wide control flow"""
        # Convert system flow to Cytoscape format
        cytoscape_data = {
            "nodes": [],
            "edges": []
        }
        
        # Add component nodes
        for component in self.analyzer.system_flow.component_interactions.keys():
            cytoscape_data["nodes"].append({
                "data": {
                    "id": component,
                    "label": component,
                    "type": "component"
                }
            })
        
        # Add interaction edges
        for source, targets in self.analyzer.system_flow.component_interactions.items():
            for target in targets:
                cytoscape_data["edges"].append({
                    "data": {
                        "id": f"{source}-{target}",
                        "source": source,
                        "target": target,
                        "edge_type": "interaction"
                    }
                })
        
        # Generate simplified HTML for system view
        html_content = self._generate_system_html_template(cytoscape_data)
        
        # Save to file
        html_file = f"{output_path}_interactive.html"
        with open(html_file, 'w') as f:
            f.write(html_content)
        logger.info(f"Interactive system HTML visualization saved to {html_file}")
        
        return html_content

    def _generate_system_html_template(self, cytoscape_data: dict) -> str:
        """Generate HTML template for system-wide visualization"""
        import json
        graph_data_json = json.dumps(cytoscape_data, indent=2)
        
        # Simplified template for system view
        return f'''<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>System Control Flow</title>
    <script src="https://unpkg.com/cytoscape@3.23.0/dist/cytoscape.min.js"></script>
    <script src="https://unpkg.com/dagre@0.8.5/dist/dagre.min.js"></script>
    <script src="https://unpkg.com/cytoscape-dagre@2.5.0/cytoscape-dagre.js"></script>
    <style>
        body {{ margin: 0; font-family: Arial, sans-serif; }}
        #cy {{ width: 100vw; height: 100vh; }}
        .controls {{ position: absolute; top: 10px; left: 10px; background: white; padding: 10px; border-radius: 5px; }}
    </style>
</head>
<body>
    <div class="controls">
        <h3>System Control Flow</h3>
        <button onclick="cy.fit()">Fit to View</button>
        <button onclick="setLayout('dagre')">Hierarchical</button>
        <button onclick="setLayout('circle')">Circle</button>
    </div>
    <div id="cy"></div>
    
    <script>
        const graphData = {graph_data_json};
        const cy = cytoscape({{
            container: document.getElementById('cy'),
            elements: graphData.nodes.concat(graphData.edges),
            style: [
                {{
                    selector: 'node',
                    style: {{
                        'content': 'data(label)',
                        'background-color': '#3498db',
                        'color': 'white',
                        'font-size': '12px',
                        'text-valign': 'center',
                        'width': 80,
                        'height': 40
                    }}
                }},
                {{
                    selector: 'edge',
                    style: {{
                        'width': 2,
                        'line-color': '#34495e',
                        'target-arrow-color': '#34495e',
                        'target-arrow-shape': 'triangle'
                    }}
                }}
            ],
            layout: {{ name: 'dagre', rankDir: 'TB' }}
        }});
        
        function setLayout(name) {{
            cy.layout({{ name: name }}).run();
        }}
    </script>
</body>
</html>'''

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
    parser.add_argument('--mermaid', action='store_true', help='Generate Mermaid flowcharts (LLM/docs friendly)')
    parser.add_argument('--json', action='store_true', help='Export structured JSON data')
    parser.add_argument('--summary', action='store_true', help='Generate textual summaries')
    parser.add_argument('--llm-friendly', action='store_true', help='Generate all LLM-friendly formats (Mermaid, JSON, summaries)')
    parser.add_argument('--interactive', action='store_true', help='Generate interactive HTML visualizations')
    
    args = parser.parse_args()
    
    if args.all:
        args.system = True
        setattr(args, 'async_patterns', True)
        args.complexity = True
        args.llm_friendly = True
    
    if args.llm_friendly:
        args.mermaid = True
        args.json = True
        args.summary = True
        args.interactive = True
    
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
        
        # Generate traditional graphviz output
        visualizer.generate_function_cfg(args.function, output_path)
        
        # Generate LLM-friendly formats if requested
        if args.mermaid:
            visualizer.generate_mermaid_flowchart(args.function, output_path)
        if args.json:
            visualizer.export_function_json(args.function, output_path)
        if args.summary:
            visualizer.generate_textual_summary(args.function, output_path)
        if args.interactive:
            visualizer.generate_interactive_html(args.function, output_path)
    
    if args.system:
        system_path = os.path.join(args.output_dir, "system_control_flow")
        visualizer.generate_system_flow_graph(system_path)
        
        if args.json:
            visualizer.export_system_json(system_path)
        if args.interactive:
            visualizer.generate_system_interactive_html(system_path)
    
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
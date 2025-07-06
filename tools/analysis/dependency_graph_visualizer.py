#!/usr/bin/env python3
"""
Toka Dependency Graph Visualizer

This tool creates dependency graph visualizations for the Toka codebase:
- Individual crate dependency graphs
- System-wide dependency visualization
- Agent composition visualization based on agent spec schema
- Parallel processing for efficient analysis

Generated on: 2025-07-06
"""

import os
import re
import json
import toml
import yaml
import asyncio
import aiofiles
from pathlib import Path
from typing import Dict, List, Set, Optional, Tuple
from dataclasses import dataclass, field
from collections import defaultdict
import graphviz
from concurrent.futures import ThreadPoolExecutor, as_completed
import argparse
import logging

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

@dataclass
class CrateInfo:
    """Information about a Rust crate"""
    name: str
    path: str
    version: str
    description: str
    dependencies: Dict[str, str] = field(default_factory=dict)
    dev_dependencies: Dict[str, str] = field(default_factory=dict)
    workspace_dependencies: Set[str] = field(default_factory=set)
    external_dependencies: Set[str] = field(default_factory=set)
    category: str = "general"

@dataclass
class AgentSpec:
    """Agent specification from schema"""
    name: str
    domain: str
    priority: str
    capabilities: List[str]
    dependencies: Dict[str, str] = field(default_factory=dict)
    objectives: List[str] = field(default_factory=list)

class DependencyAnalyzer:
    """Main dependency analyzer class"""
    
    def __init__(self, workspace_path: str):
        self.workspace_path = Path(workspace_path)
        self.crates: Dict[str, CrateInfo] = {}
        self.workspace_manifest = None
        self.agent_specs: List[AgentSpec] = []
        self.dependency_graph = defaultdict(set)
        
    async def analyze_workspace(self) -> None:
        """Analyze the entire workspace"""
        logger.info("Starting workspace analysis...")
        
        # Load workspace manifest
        await self._load_workspace_manifest()
        
        # Analyze all crates in parallel
        await self._analyze_crates_parallel()
        
        # Load agent specifications
        await self._load_agent_specs()
        
        # Build dependency graph
        self._build_dependency_graph()
        
        logger.info(f"Analysis complete. Found {len(self.crates)} crates and {len(self.agent_specs)} agents.")
    
    async def _load_workspace_manifest(self) -> None:
        """Load workspace Cargo.toml"""
        workspace_toml = self.workspace_path / "Cargo.toml"
        if workspace_toml.exists():
            async with aiofiles.open(workspace_toml, 'r') as f:
                content = await f.read()
                self.workspace_manifest = toml.loads(content)
        else:
            raise FileNotFoundError("Workspace Cargo.toml not found")
    
    async def _analyze_crates_parallel(self) -> None:
        """Analyze all crates in parallel"""
        crate_paths = []
        
        # Find all crate directories
        if self.workspace_manifest:
            for member in self.workspace_manifest.get('workspace', {}).get('members', []):
                crate_path = self.workspace_path / member
                if crate_path.exists():
                    crate_paths.append(crate_path)
        
        # Process crates in parallel
        with ThreadPoolExecutor(max_workers=8) as executor:
            futures = [executor.submit(self._analyze_single_crate, path) for path in crate_paths]
            
            for future in as_completed(futures):
                try:
                    crate_info = future.result()
                    if crate_info:
                        self.crates[crate_info.name] = crate_info
                except Exception as e:
                    logger.error(f"Error analyzing crate: {e}")
    
    def _analyze_single_crate(self, crate_path: Path) -> Optional[CrateInfo]:
        """Analyze a single crate"""
        try:
            cargo_toml = crate_path / "Cargo.toml"
            if not cargo_toml.exists():
                return None
                
            with open(cargo_toml, 'r') as f:
                manifest = toml.load(f)
            
            package = manifest.get('package', {})
            name = package.get('name', crate_path.name)
            # Get version from package or workspace default
            workspace_version = '0.0.0'
            if self.workspace_manifest:
                workspace_version = self.workspace_manifest.get('workspace', {}).get('package', {}).get('version', '0.0.0')
            version = package.get('version', workspace_version)
            description = package.get('description', '')
            
            # Categorize crate
            category = self._categorize_crate(name, description)
            
            crate_info = CrateInfo(
                name=name,
                path=str(crate_path),
                version=version,
                description=description,
                category=category
            )
            
            # Parse dependencies
            dependencies = manifest.get('dependencies', {})
            dev_dependencies = manifest.get('dev-dependencies', {})
            
            for dep_name, dep_info in dependencies.items():
                if isinstance(dep_info, dict):
                    if 'path' in dep_info:
                        # Workspace dependency
                        crate_info.workspace_dependencies.add(dep_name)
                        crate_info.dependencies[dep_name] = dep_info.get('version', 'path')
                    else:
                        # External dependency
                        crate_info.external_dependencies.add(dep_name)
                        crate_info.dependencies[dep_name] = dep_info.get('version', 'workspace')
                else:
                    # Simple version string
                    crate_info.external_dependencies.add(dep_name)
                    crate_info.dependencies[dep_name] = str(dep_info)
            
            # Parse dev dependencies
            for dep_name, dep_info in dev_dependencies.items():
                if isinstance(dep_info, dict):
                    if 'path' in dep_info:
                        crate_info.dev_dependencies[dep_name] = dep_info.get('version', 'path')
                    else:
                        crate_info.dev_dependencies[dep_name] = dep_info.get('version', 'workspace')
                else:
                    crate_info.dev_dependencies[dep_name] = str(dep_info)
            
            return crate_info
            
        except Exception as e:
            logger.error(f"Error analyzing crate at {crate_path}: {e}")
            return None
    
    def _categorize_crate(self, name: str, description: str) -> str:
        """Categorize crate based on name and description"""
        name_lower = name.lower()
        desc_lower = description.lower()
        
        if 'store' in name_lower or 'storage' in name_lower:
            return 'storage'
        elif 'auth' in name_lower or 'security' in name_lower:
            return 'security'
        elif 'kernel' in name_lower or 'core' in name_lower:
            return 'core'
        elif 'runtime' in name_lower or 'agent' in name_lower:
            return 'runtime'
        elif 'cli' in name_lower or 'tools' in name_lower:
            return 'tools'
        elif 'llm' in name_lower or 'gateway' in name_lower:
            return 'llm'
        elif 'raft' in name_lower:
            return 'consensus'
        elif 'orchestration' in name_lower:
            return 'orchestration'
        elif 'bus' in name_lower:
            return 'messaging'
        else:
            return 'general'
    
    async def _load_agent_specs(self) -> None:
        """Load agent specifications from config"""
        try:
            agents_config = self.workspace_path / "config" / "agents.toml"
            if agents_config.exists():
                async with aiofiles.open(agents_config, 'r') as f:
                    content = await f.read()
                    config = toml.loads(content)
                    
                    for agent_config in config.get('agents', []):
                        agent_spec = AgentSpec(
                            name=agent_config.get('name', ''),
                            domain=agent_config.get('domain', ''),
                            priority=agent_config.get('priority', 'medium'),
                            capabilities=agent_config.get('capabilities', {}).get('primary', []),
                            dependencies=agent_config.get('dependencies', {}),
                            objectives=[obj.get('description', '') for obj in agent_config.get('objectives', [])]
                        )
                        self.agent_specs.append(agent_spec)
        except Exception as e:
            logger.warning(f"Could not load agent specs: {e}")
    
    def _build_dependency_graph(self) -> None:
        """Build the dependency graph"""
        for crate_name, crate_info in self.crates.items():
            for dep_name in crate_info.workspace_dependencies:
                if dep_name in self.crates:
                    self.dependency_graph[crate_name].add(dep_name)

class GraphVisualizer:
    """Generate visual dependency graphs"""
    
    def __init__(self, analyzer: DependencyAnalyzer):
        self.analyzer = analyzer
        self.colors = {
            'core': '#FF6B6B',
            'storage': '#4ECDC4',
            'security': '#45B7D1',
            'runtime': '#96CEB4',
            'tools': '#FFEAA7',
            'llm': '#DDA0DD',
            'consensus': '#98D8C8',
            'orchestration': '#F7DC6F',
            'messaging': '#AED6F1',
            'general': '#D5DBDB'
        }
    
    def generate_system_graph(self, output_path: str = "system_dependency_graph") -> None:
        """Generate system-wide dependency graph"""
        dot = graphviz.Digraph(comment='Toka System Dependencies')
        dot.attr(rankdir='TB', size='20,16', dpi='300')
        dot.attr('node', shape='box', style='filled,rounded')
        
        # Add nodes with categories
        for crate_name, crate_info in self.analyzer.crates.items():
            color = self.colors.get(crate_info.category, self.colors['general'])
            label = f"{crate_name}\\n({crate_info.category})\\nv{crate_info.version}"
            dot.node(crate_name, label=label, fillcolor=color, fontsize='10')
        
        # Add edges
        for crate_name, dependencies in self.analyzer.dependency_graph.items():
            for dep in dependencies:
                dot.edge(crate_name, dep, color='gray', arrowsize='0.5')
        
        # Add legend
        with dot.subgraph(name='cluster_legend') as legend:
            legend.attr(label='Categories', style='filled', fillcolor='lightgray')
            for category, color in self.colors.items():
                legend.node(f'legend_{category}', label=category, fillcolor=color, shape='box')
        
        # Render graph
        dot.render(output_path, format='png', cleanup=True)
        dot.render(output_path, format='svg', cleanup=True)
        logger.info(f"System dependency graph saved to {output_path}")
    
    def generate_individual_crate_graphs(self, output_dir: str = "crate_graphs") -> None:
        """Generate individual crate dependency graphs"""
        os.makedirs(output_dir, exist_ok=True)
        
        for crate_name, crate_info in self.analyzer.crates.items():
            if not crate_info.workspace_dependencies and not crate_info.external_dependencies:
                continue
                
            dot = graphviz.Digraph(comment=f'{crate_name} Dependencies')
            dot.attr(rankdir='TB', size='12,8', dpi='300')
            dot.attr('node', shape='box', style='filled,rounded')
            
            # Central node
            color = self.colors.get(crate_info.category, self.colors['general'])
            dot.node(crate_name, label=f"{crate_name}\\nv{crate_info.version}", 
                    fillcolor=color, fontsize='12', penwidth='2')
            
            # Workspace dependencies
            for dep in crate_info.workspace_dependencies:
                if dep in self.analyzer.crates:
                    dep_info = self.analyzer.crates[dep]
                    dep_color = self.colors.get(dep_info.category, self.colors['general'])
                    dot.node(dep, label=f"{dep}\\n(workspace)", fillcolor=dep_color, fontsize='10')
                    dot.edge(crate_name, dep, color='blue', penwidth='2')
            
            # External dependencies (limited to important ones)
            important_external = ['tokio', 'serde', 'anyhow', 'thiserror', 'tracing', 'axum', 'sqlx']
            for dep in crate_info.external_dependencies:
                if dep in important_external:
                    dot.node(f"ext_{dep}", label=f"{dep}\\n(external)", fillcolor='lightgray', fontsize='9')
                    dot.edge(crate_name, f"ext_{dep}", color='gray', style='dashed')
            
            # Render graph
            output_path = os.path.join(output_dir, f"{crate_name}_dependencies")
            dot.render(output_path, format='png', cleanup=True)
            dot.render(output_path, format='svg', cleanup=True)
        
        logger.info(f"Individual crate graphs saved to {output_dir}")
    
    def generate_agent_composition_graph(self, output_path: str = "agent_composition_graph") -> None:
        """Generate agent composition graph based on agent specs"""
        if not self.analyzer.agent_specs:
            logger.warning("No agent specifications found")
            return
        
        dot = graphviz.Digraph(comment='Agent Composition')
        dot.attr(rankdir='TB', size='16,12', dpi='300')
        dot.attr('node', shape='ellipse', style='filled')
        
        # Domain colors
        domain_colors = {
            'code-analysis': '#FF6B6B',
            'testing': '#4ECDC4',
            'security': '#45B7D1',
            'performance': '#96CEB4',
            'infrastructure': '#FFEAA7',
            'devops-infrastructure': '#DDA0DD',
            'quality-assurance': '#98D8C8',
            'kernel-architecture': '#F7DC6F',
            'storage-architecture': '#AED6F1',
            'operations': '#D5DBDB'
        }
        
        # Add agent nodes
        for agent in self.analyzer.agent_specs:
            color = domain_colors.get(agent.domain, '#D5DBDB')
            priority_style = 'bold' if agent.priority == 'critical' else 'normal'
            
            label = f"{agent.name}\\n({agent.domain})\\nPriority: {agent.priority}\\nCapabilities: {len(agent.capabilities)}"
            dot.node(agent.name, label=label, fillcolor=color, fontweight=priority_style)
        
        # Add dependencies between agents
        for agent in self.analyzer.agent_specs:
            for dep_name, reason in agent.dependencies.items():
                if any(a.name == dep_name for a in self.analyzer.agent_specs):
                    dot.edge(agent.name, dep_name, label=reason[:20] + "..." if len(reason) > 20 else reason,
                            fontsize='8', color='gray')
        
        # Add domain clusters
        domain_agents = defaultdict(list)
        for agent in self.analyzer.agent_specs:
            domain_agents[agent.domain].append(agent.name)
        
        for domain, agents in domain_agents.items():
            if len(agents) > 1:
                with dot.subgraph(name=f'cluster_{domain}') as cluster:
                    cluster.attr(label=f'Domain: {domain}', style='dashed')
                    for agent_name in agents:
                        cluster.node(agent_name)
        
        # Render graph
        dot.render(output_path, format='png', cleanup=True)
        dot.render(output_path, format='svg', cleanup=True)
        logger.info(f"Agent composition graph saved to {output_path}")
    
    def generate_layered_architecture_graph(self, output_path: str = "layered_architecture") -> None:
        """Generate layered architecture visualization"""
        dot = graphviz.Digraph(comment='Toka Layered Architecture')
        dot.attr(rankdir='TB', size='20,16', dpi='300')
        dot.attr('node', shape='box', style='filled,rounded')
        
        # Define layers
        layers = {
            'Applications': ['toka-cli', 'toka-config-cli'],
            'Agent Layer': ['toka-agent-runtime', 'toka-orchestration'],
            'Runtime Layer': ['toka-runtime', 'toka-llm-gateway'],
            'Core Layer': ['toka-kernel', 'toka-bus-core', 'toka-types', 'toka-auth'],
            'Storage Layer': ['toka-store-core', 'toka-store-memory', 'toka-store-sled', 
                            'toka-store-sqlite', 'toka-store-semantic'],
            'Consensus Layer': ['raft-core', 'raft-storage'],
            'Security Layer': ['toka-capability-core', 'toka-capability-jwt-hs256', 
                             'toka-key-rotation', 'toka-rate-limiter'],
            'Utilities': ['toka-tools', 'toka-performance']
        }
        
        # Create layer subgraphs
        for layer_name, crates in layers.items():
            with dot.subgraph(name=f'cluster_{layer_name.replace(" ", "_")}') as cluster:
                cluster.attr(label=layer_name, style='filled', fillcolor='lightgray', fontsize='14')
                
                for crate_name in crates:
                    if crate_name in self.analyzer.crates:
                        crate_info = self.analyzer.crates[crate_name]
                        color = self.colors.get(crate_info.category, self.colors['general'])
                        cluster.node(crate_name, label=f"{crate_name}\\nv{crate_info.version}", 
                                   fillcolor=color, fontsize='10')
        
        # Add cross-layer dependencies
        for crate_name, dependencies in self.analyzer.dependency_graph.items():
            for dep in dependencies:
                if crate_name in self.analyzer.crates and dep in self.analyzer.crates:
                    dot.edge(crate_name, dep, color='blue', arrowsize='0.5')
        
        # Render graph
        dot.render(output_path, format='png', cleanup=True)
        dot.render(output_path, format='svg', cleanup=True)
        logger.info(f"Layered architecture graph saved to {output_path}")

async def main():
    """Main function to run the dependency analysis and visualization"""
    parser = argparse.ArgumentParser(description='Toka Dependency Graph Visualizer')
    parser.add_argument('--workspace', default='.', help='Workspace path (default: current directory)')
    parser.add_argument('--output-dir', default='dependency_graphs', help='Output directory for graphs')
    parser.add_argument('--individual', action='store_true', help='Generate individual crate graphs')
    parser.add_argument('--agents', action='store_true', help='Generate agent composition graph')
    parser.add_argument('--layered', action='store_true', help='Generate layered architecture graph')
    parser.add_argument('--all', action='store_true', help='Generate all graphs')
    
    args = parser.parse_args()
    
    if args.all:
        args.individual = args.agents = args.layered = True
    
    # Create output directory
    os.makedirs(args.output_dir, exist_ok=True)
    
    # Initialize analyzer
    analyzer = DependencyAnalyzer(args.workspace)
    
    # Analyze workspace
    await analyzer.analyze_workspace()
    
    # Generate visualizations
    visualizer = GraphVisualizer(analyzer)
    
    # Always generate system graph
    system_graph_path = os.path.join(args.output_dir, "system_dependency_graph")
    visualizer.generate_system_graph(system_graph_path)
    
    if args.individual:
        individual_dir = os.path.join(args.output_dir, "individual_crates")
        visualizer.generate_individual_crate_graphs(individual_dir)
    
    if args.agents:
        agent_graph_path = os.path.join(args.output_dir, "agent_composition_graph")
        visualizer.generate_agent_composition_graph(agent_graph_path)
    
    if args.layered:
        layered_graph_path = os.path.join(args.output_dir, "layered_architecture")
        visualizer.generate_layered_architecture_graph(layered_graph_path)
    
    # Generate summary report
    await generate_summary_report(analyzer, args.output_dir)
    
    logger.info("Dependency analysis and visualization complete!")

async def generate_summary_report(analyzer: DependencyAnalyzer, output_dir: str):
    """Generate a summary report of the analysis"""
    report_path = os.path.join(output_dir, "dependency_analysis_report.md")
    
    with open(report_path, 'w') as f:
        f.write("# Toka Dependency Analysis Report\n\n")
        f.write(f"Generated on: {os.popen('date').read().strip()}\n\n")
        
        f.write("## Overview\n\n")
        f.write(f"- Total crates analyzed: {len(analyzer.crates)}\n")
        f.write(f"- Total agent specifications: {len(analyzer.agent_specs)}\n")
        f.write(f"- Total internal dependencies: {sum(len(deps) for deps in analyzer.dependency_graph.values())}\n\n")
        
        f.write("## Crate Categories\n\n")
        categories = defaultdict(int)
        for crate in analyzer.crates.values():
            categories[crate.category] += 1
        
        for category, count in sorted(categories.items()):
            f.write(f"- {category}: {count} crates\n")
        
        f.write("\n## Crate Details\n\n")
        for crate_name, crate_info in sorted(analyzer.crates.items()):
            f.write(f"### {crate_name}\n")
            f.write(f"- Version: {crate_info.version}\n")
            f.write(f"- Category: {crate_info.category}\n")
            f.write(f"- Description: {crate_info.description}\n")
            f.write(f"- Workspace dependencies: {len(crate_info.workspace_dependencies)}\n")
            f.write(f"- External dependencies: {len(crate_info.external_dependencies)}\n\n")
        
        if analyzer.agent_specs:
            f.write("## Agent Specifications\n\n")
            for agent in analyzer.agent_specs:
                f.write(f"### {agent.name}\n")
                f.write(f"- Domain: {agent.domain}\n")
                f.write(f"- Priority: {agent.priority}\n")
                f.write(f"- Capabilities: {', '.join(agent.capabilities)}\n")
                f.write(f"- Dependencies: {len(agent.dependencies)}\n\n")
    
    logger.info(f"Summary report saved to {report_path}")

if __name__ == "__main__":
    asyncio.run(main())
"""
Dependency Graph Analysis for Rust Workspaces

This module provides comprehensive dependency analysis capabilities:
- Crate dependency mapping
- Agent composition analysis
- System architecture visualization
- Multi-format output support
"""

import os
import re
import json
import toml
import asyncio
import aiofiles
from pathlib import Path
from typing import Dict, List, Set, Optional, Tuple, Union, Any
from dataclasses import dataclass, field
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor, as_completed
import logging

from .config import AnalysisConfig

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
    
    def __init__(self, workspace_path: str, config: Optional[AnalysisConfig] = None):
        self.workspace_path = Path(workspace_path)
        self.config = config or AnalysisConfig()
        self.crates: Dict[str, CrateInfo] = {}
        self.workspace_manifest: Optional[Dict[str, Any]] = None
        self.agent_specs: List[AgentSpec] = []
        self.dependency_graph = defaultdict(set)
        
    async def analyze_workspace(self) -> None:
        """Analyze the entire workspace"""
        logger.info("Starting dependency analysis...")
        
        # Validate workspace
        if not self.workspace_path.exists():
            raise ValueError(f"Workspace path does not exist: {self.workspace_path}")
        
        # Create output directories
        self.config.create_output_dirs()
        
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
        with ThreadPoolExecutor(max_workers=self.config.max_concurrent_analyzers) as executor:
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

class DependencyVisualizer:
    """Generate visual dependency graphs"""
    
    def __init__(self, analyzer: DependencyAnalyzer):
        self.analyzer = analyzer
        self.config = analyzer.config
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
    
    def generate_mermaid_graph(self, output_path: Optional[str] = None) -> str:
        """Generate Mermaid diagram for dependencies"""
        mermaid_lines = ["graph TD"]
        
        # Add nodes with categories
        for crate_name, crate_info in self.analyzer.crates.items():
            clean_name = crate_name.replace('-', '_')
            label = f"{crate_name}\\n[{crate_info.category}]"
            mermaid_lines.append(f'    {clean_name}["{label}"]')
        
        # Add edges
        for crate_name, dependencies in self.analyzer.dependency_graph.items():
            clean_source = crate_name.replace('-', '_')
            for dep in dependencies:
                clean_target = dep.replace('-', '_')
                mermaid_lines.append(f'    {clean_source} --> {clean_target}')
        
        # Add styling
        mermaid_lines.extend([
            "",
            "    %% Styling by category",
            "    classDef core fill:#FF6B6B,stroke:#C62828,color:#fff",
            "    classDef storage fill:#4ECDC4,stroke:#00695C,color:#fff",
            "    classDef security fill:#45B7D1,stroke:#1565C0,color:#fff",
            "    classDef runtime fill:#96CEB4,stroke:#2E7D32,color:#fff",
            "    classDef tools fill:#FFEAA7,stroke:#F57F17",
            "    classDef llm fill:#DDA0DD,stroke:#7B1FA2,color:#fff",
            "    classDef consensus fill:#98D8C8,stroke:#00695C,color:#fff",
            "    classDef orchestration fill:#F7DC6F,stroke:#F57F17",
            "    classDef messaging fill:#AED6F1,stroke:#1565C0"
        ])
        
        # Apply classes to nodes
        category_nodes = defaultdict(list)
        for crate_name, crate_info in self.analyzer.crates.items():
            category_nodes[crate_info.category].append(crate_name.replace('-', '_'))
        
        for category, nodes in category_nodes.items():
            if nodes:
                node_list = ','.join(nodes)
                mermaid_lines.append(f"    class {node_list} {category}")
        
        mermaid_content = "\n".join(mermaid_lines)
        
        # Add metadata
        mermaid_content += f"\n\n%% Dependency Analysis Metadata:\n"
        mermaid_content += f"%% Total Crates: {len(self.analyzer.crates)}\n"
        mermaid_content += f"%% Total Dependencies: {sum(len(deps) for deps in self.analyzer.dependency_graph.values())}\n"
        mermaid_content += f"%% Total Agents: {len(self.analyzer.agent_specs)}\n"
        
        # Save to file if path provided
        if output_path:
            mermaid_file = f"{output_path}.mmd"
            with open(mermaid_file, 'w') as f:
                f.write(mermaid_content)
            logger.info(f"Mermaid dependency graph saved to {mermaid_file}")
        
        return mermaid_content
    
    def export_json(self, output_path: Optional[str] = None) -> Dict[str, Any]:
        """Export dependency data as structured JSON"""
        export_data = {
            "workspace_info": {
                "path": str(self.analyzer.workspace_path),
                "total_crates": len(self.analyzer.crates),
                "total_agents": len(self.analyzer.agent_specs),
                "total_dependencies": sum(len(deps) for deps in self.analyzer.dependency_graph.values())
            },
            "crates": {},
            "agents": [],
            "dependency_graph": {},
            "category_distribution": {},
            "analysis_summary": self._generate_analysis_summary()
        }
        
        # Add crates
        for crate_name, crate_info in self.analyzer.crates.items():
            export_data["crates"][crate_name] = {
                "name": crate_info.name,
                "path": crate_info.path,
                "version": crate_info.version,
                "description": crate_info.description,
                "category": crate_info.category,
                "workspace_dependencies": list(crate_info.workspace_dependencies),
                "external_dependencies": list(crate_info.external_dependencies),
                "dependency_count": len(crate_info.dependencies),
                "dev_dependency_count": len(crate_info.dev_dependencies)
            }
        
        # Add agents
        for agent in self.analyzer.agent_specs:
            export_data["agents"].append({
                "name": agent.name,
                "domain": agent.domain,
                "priority": agent.priority,
                "capabilities": agent.capabilities,
                "dependencies": agent.dependencies,
                "objectives": agent.objectives
            })
        
        # Add dependency graph
        for crate_name, deps in self.analyzer.dependency_graph.items():
            export_data["dependency_graph"][crate_name] = list(deps)
        
        # Add category distribution
        category_counts = defaultdict(int)
        for crate_info in self.analyzer.crates.values():
            category_counts[crate_info.category] += 1
        export_data["category_distribution"] = dict(category_counts)
        
        # Save to file if path provided
        if output_path:
            json_file = f"{output_path}.json"
            with open(json_file, 'w') as f:
                json.dump(export_data, f, indent=2)
            logger.info(f"Dependency JSON export saved to {json_file}")
        
        return export_data
    
    def generate_textual_summary(self, output_path: Optional[str] = None) -> str:
        """Generate human and LLM readable summary"""
        summary_lines = [
            "# Dependency Analysis Summary",
            "",
            f"**Workspace:** {self.analyzer.workspace_path}",
            f"**Total Crates:** {len(self.analyzer.crates)}",
            f"**Total Agents:** {len(self.analyzer.agent_specs)}",
            f"**Total Internal Dependencies:** {sum(len(deps) for deps in self.analyzer.dependency_graph.values())}",
            "",
            "## Crate Categories",
        ]
        
        # Category distribution
        category_counts = defaultdict(int)
        for crate_info in self.analyzer.crates.values():
            category_counts[crate_info.category] += 1
        
        for category, count in sorted(category_counts.items()):
            summary_lines.append(f"- **{category.title()}:** {count} crates")
        
        summary_lines.extend([
            "",
            "## High-Level Architecture",
            "",
            "### Core Components",
        ])
        
        # Core components
        core_crates = [name for name, info in self.analyzer.crates.items() if info.category == 'core']
        for crate in sorted(core_crates):
            crate_info = self.analyzer.crates[crate]
            deps_count = len(self.analyzer.dependency_graph.get(crate, set()))
            summary_lines.append(f"- **{crate}**: {crate_info.description} (depends on {deps_count} internal crates)")
        
        summary_lines.extend([
            "",
            "### Storage Layer",
        ])
        
        # Storage components
        storage_crates = [name for name, info in self.analyzer.crates.items() if info.category == 'storage']
        for crate in sorted(storage_crates):
            crate_info = self.analyzer.crates[crate]
            deps_count = len(self.analyzer.dependency_graph.get(crate, set()))
            summary_lines.append(f"- **{crate}**: {crate_info.description} (depends on {deps_count} internal crates)")
        
        summary_lines.extend([
            "",
            "### Runtime & Orchestration",
        ])
        
        # Runtime components
        runtime_crates = [name for name, info in self.analyzer.crates.items() 
                         if info.category in ['runtime', 'orchestration']]
        for crate in sorted(runtime_crates):
            crate_info = self.analyzer.crates[crate]
            deps_count = len(self.analyzer.dependency_graph.get(crate, set()))
            summary_lines.append(f"- **{crate}**: {crate_info.description} (depends on {deps_count} internal crates)")
        
        if self.analyzer.agent_specs:
            summary_lines.extend([
                "",
                "## Agent Composition",
            ])
            
            # Group agents by domain
            domain_agents = defaultdict(list)
            for agent in self.analyzer.agent_specs:
                domain_agents[agent.domain].append(agent)
            
            for domain, agents in sorted(domain_agents.items()):
                summary_lines.append(f"### {domain.replace('-', ' ').title()}")
                for agent in agents:
                    capabilities = len(agent.capabilities)
                    summary_lines.append(f"- **{agent.name}**: {capabilities} capabilities, priority: {agent.priority}")
        
        summary_lines.extend([
            "",
            "## Dependency Complexity Analysis",
            self._generate_complexity_analysis(),
            "",
            "## Architecture Insights",
            self._generate_architecture_insights()
        ])
        
        summary_content = "\n".join(summary_lines)
        
        # Save to file if path provided
        if output_path:
            summary_file = f"{output_path}_summary.md"
            with open(summary_file, 'w') as f:
                f.write(summary_content)
            logger.info(f"Dependency summary saved to {summary_file}")
        
        return summary_content
    
    def generate_interactive_html(self, output_path: Optional[str] = None) -> str:
        """Generate interactive HTML visualization"""
        # Convert dependency data to network format
        nodes = []
        edges = []
        
        # Add nodes
        for crate_name, crate_info in self.analyzer.crates.items():
            nodes.append({
                "id": crate_name,
                "label": crate_name,
                "category": crate_info.category,
                "version": crate_info.version,
                "description": crate_info.description,
                "workspace_deps": len(crate_info.workspace_dependencies),
                "external_deps": len(crate_info.external_dependencies)
            })
        
        # Add edges
        for crate_name, deps in self.analyzer.dependency_graph.items():
            for dep in deps:
                edges.append({
                    "source": crate_name,
                    "target": dep,
                    "type": "dependency"
                })
        
        network_data = {
            "nodes": nodes,
            "edges": edges
        }
        
        # Generate HTML template
        html_content = self._generate_html_template(network_data)
        
        # Save to file if path provided
        if output_path:
            html_file = f"{output_path}_interactive.html"
            with open(html_file, 'w') as f:
                f.write(html_content)
            logger.info(f"Interactive dependency visualization saved to {html_file}")
        
        return html_content
    
    def generate_system_graph(self, output_path: Optional[str] = None) -> str:
        """Generate system-wide dependency graph"""
        # TODO: Implement using graphviz for complex system graphs
        return "System dependency graph generation not yet implemented"
    
    def generate_individual_crate_graphs(self, output_dir: Optional[str] = None) -> str:
        """Generate individual crate dependency graphs"""
        # TODO: Implement individual crate graph generation
        return "Individual crate graphs generation not yet implemented"
    
    def generate_agent_composition_graph(self, output_path: Optional[str] = None) -> str:
        """Generate agent composition graph"""
        # TODO: Implement agent composition visualization
        return "Agent composition graph generation not yet implemented"
    
    def generate_layered_architecture_graph(self, output_path: Optional[str] = None) -> str:
        """Generate layered architecture visualization"""
        # TODO: Implement layered architecture graph
        return "Layered architecture graph generation not yet implemented"
    
    def _generate_analysis_summary(self) -> str:
        """Generate a concise analysis summary"""
        total_crates = len(self.analyzer.crates)
        total_deps = sum(len(deps) for deps in self.analyzer.dependency_graph.values())
        avg_deps = total_deps / total_crates if total_crates > 0 else 0
        
        # Find most connected crates
        most_connected = sorted(
            self.analyzer.dependency_graph.items(),
            key=lambda x: len(x[1]),
            reverse=True
        )[:3]
        
        summary = f"Analyzed {total_crates} crates with {total_deps} internal dependencies. "
        summary += f"Average {avg_deps:.1f} dependencies per crate. "
        
        if most_connected:
            top_crate = most_connected[0]
            summary += f"Most connected crate: {top_crate[0]} ({len(top_crate[1])} dependencies)."
        
        return summary
    
    def _generate_complexity_analysis(self) -> str:
        """Generate complexity analysis of the dependency structure"""
        analysis_lines = []
        
        # Calculate metrics
        total_crates = len(self.analyzer.crates)
        total_deps = sum(len(deps) for deps in self.analyzer.dependency_graph.values())
        
        if total_crates == 0:
            return "No crates found for analysis."
        
        avg_deps = total_deps / total_crates
        
        # Find crates with high dependency counts
        high_dep_crates = [
            (name, len(deps)) for name, deps in self.analyzer.dependency_graph.items()
            if len(deps) > avg_deps * 1.5
        ]
        
        analysis_lines.append(f"- **Average Dependencies per Crate:** {avg_deps:.1f}")
        analysis_lines.append(f"- **Total Internal Dependencies:** {total_deps}")
        
        if high_dep_crates:
            analysis_lines.append("- **High Dependency Crates:**")
            for name, count in sorted(high_dep_crates, key=lambda x: x[1], reverse=True)[:5]:
                analysis_lines.append(f"  - {name}: {count} dependencies")
        
        # Category complexity
        category_deps = defaultdict(list)
        for crate_name, crate_info in self.analyzer.crates.items():
            deps_count = len(self.analyzer.dependency_graph.get(crate_name, set()))
            category_deps[crate_info.category].append(deps_count)
        
        analysis_lines.append("- **Complexity by Category:**")
        for category, deps_list in sorted(category_deps.items()):
            if deps_list:
                avg_cat_deps = sum(deps_list) / len(deps_list)
                analysis_lines.append(f"  - {category.title()}: {avg_cat_deps:.1f} avg dependencies")
        
        return "\n".join(analysis_lines)
    
    def _generate_architecture_insights(self) -> str:
        """Generate architectural insights for LLM understanding"""
        insights = []
        
        # Analyze layering
        category_deps = defaultdict(set)
        for crate_name, deps in self.analyzer.dependency_graph.items():
            crate_info = self.analyzer.crates[crate_name]
            for dep in deps:
                if dep in self.analyzer.crates:
                    dep_category = self.analyzer.crates[dep].category
                    category_deps[crate_info.category].add(dep_category)
        
        insights.append("• **Layered Architecture**: The system follows a layered approach with clear separation of concerns")
        
        # Core dependencies
        if 'core' in category_deps:
            core_used_by = sum(1 for deps in category_deps.values() if 'core' in deps)
            insights.append(f"• **Core Foundation**: Core components are used by {core_used_by} different categories")
        
        # Storage abstraction
        if 'storage' in self.analyzer.crates:
            storage_crates = [name for name, info in self.analyzer.crates.items() if info.category == 'storage']
            if len(storage_crates) > 2:
                insights.append("• **Storage Abstraction**: Multiple storage implementations suggest pluggable architecture")
        
        # Agent orchestration
        if self.analyzer.agent_specs:
            total_agents = len(self.analyzer.agent_specs)
            domains = set(agent.domain for agent in self.analyzer.agent_specs)
            insights.append(f"• **Agent Orchestration**: {total_agents} agents across {len(domains)} domains for specialized tasks")
        
        # Microservices patterns
        if len(self.analyzer.crates) > 10:
            insights.append("• **Modular Design**: High crate count indicates microservices-like modular architecture")
        
        return "\n".join(insights)
    
    def _generate_html_template(self, network_data: Dict[str, Any]) -> str:
        """Generate HTML template for interactive visualization"""
        
        html_template = f'''<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Dependency Graph Visualization</title>
    <script src="https://unpkg.com/vis-network@9.1.2/dist/vis-network.min.js"></script>
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
            text-align: center;
        }}
        
        .title {{
            font-size: 1.8rem;
            font-weight: 600;
            margin: 0 0 0.5rem 0;
        }}
        
        .subtitle {{
            font-size: 1rem;
            opacity: 0.8;
        }}
        
        .stats {{
            background: #ecf0f1;
            padding: 1rem;
            border-radius: 8px;
            margin-bottom: 20px;
            display: flex;
            justify-content: space-around;
            text-align: center;
        }}
        
        .stat-item {{
            flex: 1;
        }}
        
        .stat-value {{
            font-size: 1.5rem;
            font-weight: bold;
            color: #2c3e50;
        }}
        
        .stat-label {{
            font-size: 0.9rem;
            color: #7f8c8d;
        }}
        
        #network {{
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
        
        .legend {{
            margin-top: 20px;
            background: white;
            padding: 1rem;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        
        .legend h3 {{
            margin: 0 0 10px 0;
            color: #2c3e50;
        }}
        
        .legend-item {{
            display: inline-block;
            margin-right: 20px;
            margin-bottom: 10px;
        }}
        
        .legend-color {{
            display: inline-block;
            width: 16px;
            height: 16px;
            border-radius: 50%;
            margin-right: 8px;
            vertical-align: middle;
        }}
    </style>
</head>
<body>
    <div class="header">
        <div class="title">Toka Dependency Graph</div>
        <div class="subtitle">Interactive visualization of crate dependencies</div>
    </div>
    
    <div class="stats">
        <div class="stat-item">
            <div class="stat-value">{len(self.analyzer.crates)}</div>
            <div class="stat-label">Crates</div>
        </div>
        <div class="stat-item">
            <div class="stat-value">{sum(len(deps) for deps in self.analyzer.dependency_graph.values())}</div>
            <div class="stat-label">Dependencies</div>
        </div>
        <div class="stat-item">
            <div class="stat-value">{len(self.analyzer.agent_specs)}</div>
            <div class="stat-label">Agents</div>
        </div>
        <div class="stat-item">
            <div class="stat-value">{len(set(info.category for info in self.analyzer.crates.values()))}</div>
            <div class="stat-label">Categories</div>
        </div>
    </div>
    
    <div id="network"></div>
    
    <div class="controls">
        <button onclick="network.fit()">Fit to View</button>
        <button onclick="resetZoom()">Reset Zoom</button>
        <button onclick="togglePhysics()">Toggle Physics</button>
        <button onclick="filterByCategory('core')">Show Core</button>
        <button onclick="filterByCategory('storage')">Show Storage</button>
        <button onclick="showAll()">Show All</button>
    </div>
    
    <div class="legend">
        <h3>Categories</h3>
        <div class="legend-item">
            <span class="legend-color" style="background: #FF6B6B;"></span>
            <span>Core</span>
        </div>
        <div class="legend-item">
            <span class="legend-color" style="background: #4ECDC4;"></span>
            <span>Storage</span>
        </div>
        <div class="legend-item">
            <span class="legend-color" style="background: #45B7D1;"></span>
            <span>Security</span>
        </div>
        <div class="legend-item">
            <span class="legend-color" style="background: #96CEB4;"></span>
            <span>Runtime</span>
        </div>
        <div class="legend-item">
            <span class="legend-color" style="background: #FFEAA7;"></span>
            <span>Tools</span>
        </div>
        <div class="legend-item">
            <span class="legend-color" style="background: #DDA0DD;"></span>
            <span>LLM</span>
        </div>
        <div class="legend-item">
            <span class="legend-color" style="background: #98D8C8;"></span>
            <span>Consensus</span>
        </div>
        <div class="legend-item">
            <span class="legend-color" style="background: #F7DC6F;"></span>
            <span>Orchestration</span>
        </div>
        <div class="legend-item">
            <span class="legend-color" style="background: #AED6F1;"></span>
            <span>Messaging</span>
        </div>
    </div>

    <script>
        // Network data
        const networkData = {json.dumps(network_data, indent=2)};
        
        // Color mapping
        const categoryColors = {{
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
        }};
        
        // Prepare nodes and edges for vis.js
        const nodes = new vis.DataSet(
            networkData.nodes.map(node => ({{
                id: node.id,
                label: node.label,
                color: {{
                    background: categoryColors[node.category] || '#D5DBDB',
                    border: '#2c3e50',
                    highlight: {{
                        background: '#e74c3c',
                        border: '#c0392b'
                    }}
                }},
                font: {{
                    size: 12,
                    color: '#2c3e50'
                }},
                title: `${{node.label}}\\nCategory: ${{node.category}}\\nVersion: ${{node.version}}\\nWorkspace Deps: ${{node.workspace_deps}}\\nExternal Deps: ${{node.external_deps}}\\n\\n${{node.description}}`
            }}))
        );
        
        const edges = new vis.DataSet(
            networkData.edges.map(edge => ({{
                from: edge.source,
                to: edge.target,
                arrows: 'to',
                color: {{
                    color: '#7f8c8d',
                    highlight: '#e74c3c'
                }},
                width: 2
            }}))
        );
        
        // Create network
        const container = document.getElementById('network');
        const data = {{ nodes: nodes, edges: edges }};
        
        const options = {{
            physics: {{
                enabled: true,
                stabilization: {{
                    iterations: 100
                }},
                barnesHut: {{
                    gravitationalConstant: -8000,
                    centralGravity: 0.3,
                    springLength: 95,
                    springConstant: 0.04,
                    damping: 0.09
                }}
            }},
            nodes: {{
                shape: 'dot',
                size: 20,
                borderWidth: 2,
                shadow: true
            }},
            edges: {{
                width: 2,
                shadow: true,
                smooth: {{
                    type: 'continuous'
                }}
            }},
            interaction: {{
                hover: true,
                tooltipDelay: 200
            }},
            layout: {{
                improvedLayout: true
            }}
        }};
        
        const network = new vis.Network(container, data, options);
        
        // Control functions
        function resetZoom() {{
            network.fit();
        }}
        
        let physicsEnabled = true;
        function togglePhysics() {{
            physicsEnabled = !physicsEnabled;
            network.setOptions({{ physics: {{ enabled: physicsEnabled }} }});
        }}
        
        function filterByCategory(category) {{
            const filteredNodes = networkData.nodes.filter(node => node.category === category);
            const nodeIds = filteredNodes.map(node => node.id);
            
            // Filter edges that connect filtered nodes
            const filteredEdges = networkData.edges.filter(edge => 
                nodeIds.includes(edge.source) || nodeIds.includes(edge.target)
            );
            
            nodes.clear();
            edges.clear();
            
            nodes.add(filteredNodes.map(node => ({{
                id: node.id,
                label: node.label,
                color: {{
                    background: categoryColors[node.category] || '#D5DBDB',
                    border: '#2c3e50'
                }},
                font: {{ size: 12, color: '#2c3e50' }},
                title: `${{node.label}}\\nCategory: ${{node.category}}\\nVersion: ${{node.version}}`
            }})));
            
            edges.add(filteredEdges.map(edge => ({{
                from: edge.source,
                to: edge.target,
                arrows: 'to',
                color: {{ color: '#7f8c8d' }},
                width: 2
            }})));
        }}
        
        function showAll() {{
            nodes.clear();
            edges.clear();
            
            nodes.add(networkData.nodes.map(node => ({{
                id: node.id,
                label: node.label,
                color: {{
                    background: categoryColors[node.category] || '#D5DBDB',
                    border: '#2c3e50'
                }},
                font: {{ size: 12, color: '#2c3e50' }},
                title: `${{node.label}}\\nCategory: ${{node.category}}\\nVersion: ${{node.version}}`
            }})));
            
            edges.add(networkData.edges.map(edge => ({{
                from: edge.source,
                to: edge.target,
                arrows: 'to',
                color: {{ color: '#7f8c8d' }},
                width: 2
            }})));
        }}
        
        // Initial fit
        network.on('stabilizationIterationsDone', function() {{
            network.fit();
        }});
    </script>
</body>
</html>'''
        
        return html_template
"""
Configuration management for Toka Analysis Tools
"""

import os
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Optional, Union
import toml
import logging

logger = logging.getLogger(__name__)

@dataclass
class AnalysisConfig:
    """Configuration for analysis tools"""
    
    # Workspace settings
    workspace_path: str = "."
    output_dir: str = "analysis_output"
    
    # Visualization settings
    enable_mermaid: bool = True
    enable_interactive: bool = True
    enable_png: bool = True
    enable_svg: bool = True
    
    # Analysis settings
    max_complexity_threshold: int = 15
    max_concurrent_analyzers: int = 8
    analysis_timeout: int = 300
    
    # Tool registration
    tool_name: str = "toka-analysis"
    tool_version: str = "0.2.1"
    tool_description: str = "Code analysis and visualization tools"
    
    # LLM integration
    llm_provider: str = "anthropic"
    llm_model: str = "claude-3-5-sonnet-20241022"
    
    # MCP server settings
    mcp_server_name: str = "toka-analysis-server"
    mcp_server_version: str = "1.0.0"
    mcp_port: int = 8080
    
    # Security settings
    sandbox_enabled: bool = True
    resource_limits: Dict[str, str] = field(default_factory=lambda: {
        "max_memory": "512MB",
        "max_cpu": "0.5",
        "timeout": "1800"
    })
    
    # Logging
    log_level: str = "INFO"
    log_format: str = "%(asctime)s - %(name)s - %(levelname)s - %(message)s"
    
    # File patterns
    rust_file_patterns: List[str] = field(default_factory=lambda: ["*.rs"])
    exclude_patterns: List[str] = field(default_factory=lambda: [
        "target/*", ".git/*", "*.tmp", "build.rs"
    ])
    
    @classmethod
    def from_file(cls, config_path: Union[str, Path]) -> "AnalysisConfig":
        """Load configuration from TOML file"""
        try:
            with open(config_path, 'r') as f:
                config_data = toml.load(f)
            
            # Extract analysis tool config section
            analysis_config = config_data.get('analysis_tools', {})
            
            # Create config with defaults, then update with file values
            config = cls()
            for key, value in analysis_config.items():
                if hasattr(config, key):
                    setattr(config, key, value)
            
            return config
            
        except Exception as e:
            logger.warning(f"Failed to load config from {config_path}: {e}")
            return cls()  # Return default config
    
    @classmethod
    def from_env(cls) -> "AnalysisConfig":
        """Load configuration from environment variables"""
        config = cls()
        
        # Map environment variables to config attributes
        env_mappings = {
            "TOKA_WORKSPACE_PATH": "workspace_path",
            "TOKA_OUTPUT_DIR": "output_dir",
            "TOKA_MAX_COMPLEXITY": "max_complexity_threshold",
            "TOKA_MAX_CONCURRENT": "max_concurrent_analyzers",
            "TOKA_ANALYSIS_TIMEOUT": "analysis_timeout",
            "TOKA_LLM_PROVIDER": "llm_provider",
            "TOKA_LLM_MODEL": "llm_model",
            "TOKA_MCP_PORT": "mcp_port",
            "TOKA_LOG_LEVEL": "log_level",
        }
        
        for env_var, attr_name in env_mappings.items():
            value = os.getenv(env_var)
            if value is not None:
                # Convert types as needed
                if attr_name in ["max_complexity_threshold", "max_concurrent_analyzers", 
                               "analysis_timeout", "mcp_port"]:
                    try:
                        value = int(value)
                    except ValueError:
                        logger.warning(f"Invalid integer value for {env_var}: {value}")
                        continue
                elif attr_name in ["enable_mermaid", "enable_interactive", "enable_png", 
                                 "enable_svg", "sandbox_enabled"]:
                    value = value.lower() in ("true", "1", "yes", "on")
                
                setattr(config, attr_name, value)
        
        return config
    
    def to_dict(self) -> Dict:
        """Convert configuration to dictionary"""
        return {
            "workspace_path": self.workspace_path,
            "output_dir": self.output_dir,
            "visualization": {
                "enable_mermaid": self.enable_mermaid,
                "enable_interactive": self.enable_interactive,
                "enable_png": self.enable_png,
                "enable_svg": self.enable_svg,
            },
            "analysis": {
                "max_complexity_threshold": self.max_complexity_threshold,
                "max_concurrent_analyzers": self.max_concurrent_analyzers,
                "analysis_timeout": self.analysis_timeout,
            },
            "tool": {
                "name": self.tool_name,
                "version": self.tool_version,
                "description": self.tool_description,
            },
            "llm": {
                "provider": self.llm_provider,
                "model": self.llm_model,
            },
            "mcp": {
                "server_name": self.mcp_server_name,
                "server_version": self.mcp_server_version,
                "port": self.mcp_port,
            },
            "security": {
                "sandbox_enabled": self.sandbox_enabled,
                "resource_limits": self.resource_limits,
            },
            "logging": {
                "level": self.log_level,
                "format": self.log_format,
            },
            "patterns": {
                "rust_files": self.rust_file_patterns,
                "exclude": self.exclude_patterns,
            },
        }
    
    def setup_logging(self) -> None:
        """Setup logging configuration"""
        logging.basicConfig(
            level=getattr(logging, self.log_level.upper()),
            format=self.log_format,
            handlers=[
                logging.StreamHandler(),
                logging.FileHandler(
                    Path(self.output_dir) / "analysis.log"
                )
            ]
        )
    
    def validate(self) -> List[str]:
        """Validate configuration and return any errors"""
        errors = []
        
        # Validate workspace path
        if not Path(self.workspace_path).exists():
            errors.append(f"Workspace path does not exist: {self.workspace_path}")
        
        # Validate thresholds
        if self.max_complexity_threshold < 1:
            errors.append("max_complexity_threshold must be >= 1")
        
        if self.max_concurrent_analyzers < 1:
            errors.append("max_concurrent_analyzers must be >= 1")
        
        if self.analysis_timeout < 1:
            errors.append("analysis_timeout must be >= 1")
        
        # Validate port
        if not (1024 <= self.mcp_port <= 65535):
            errors.append("mcp_port must be between 1024 and 65535")
        
        return errors
    
    def create_output_dirs(self) -> None:
        """Create necessary output directories"""
        output_path = Path(self.output_dir)
        output_path.mkdir(parents=True, exist_ok=True)
        
        # Create subdirectories
        for subdir in ["control_flow", "dependency_graphs", "reports", "interactive"]:
            (output_path / subdir).mkdir(exist_ok=True)
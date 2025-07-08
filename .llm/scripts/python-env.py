#!/usr/bin/env python3
"""
Python Environment Manager

Manages Python virtual environments, dependencies, and cleanup for the universal extension system.
"""

import json
import os
import sys
import argparse
import subprocess
import venv
from pathlib import Path
from typing import Dict, Any, List, Optional


class PythonEnvManager:
    """Manages Python virtual environments and dependencies."""
    
    def __init__(self, config: Dict[str, Any]):
        """Initialize the Python environment manager."""
        self.config = config
        self.venv_dir = Path(config.get('venvDirectory', 'venv'))
        self.requirements_file = Path(config.get('requirementsFile', 'requirements.txt'))
        self.python_version = config.get('pythonVersion', '3.8')
        self.auto_cleanup = config.get('autoCleanup', True)
        
    def create_venv(self, force: bool = False) -> Dict[str, Any]:
        """Create a new virtual environment."""
        try:
            if self.venv_dir.exists() and not force:
                return {
                    "success": False,
                    "message": f"Virtual environment already exists at {self.venv_dir}",
                    "data": {"venv_path": str(self.venv_dir)}
                }
            
            if self.venv_dir.exists() and force:
                self.cleanup_venv()
            
            # Create virtual environment
            venv.create(self.venv_dir, with_pip=True)
            
            # Install dependencies if requirements file exists
            if self.requirements_file.exists():
                self.install_dependencies()
            
            return {
                "success": True,
                "message": f"Virtual environment created at {self.venv_dir}",
                "data": {
                    "venv_path": str(self.venv_dir),
                    "python_path": str(self.venv_dir / "bin" / "python"),
                    "pip_path": str(self.venv_dir / "bin" / "pip")
                }
            }
            
        except Exception as e:
            return {
                "success": False,
                "message": f"Failed to create virtual environment: {e}",
                "data": {}
            }
    
    def install_dependencies(self) -> Dict[str, Any]:
        """Install dependencies from requirements file."""
        try:
            if not self.venv_dir.exists():
                return {
                    "success": False,
                    "message": "Virtual environment does not exist",
                    "data": {}
                }
            
            pip_path = self.venv_dir / "bin" / "pip"
            if not pip_path.exists():
                return {
                    "success": False,
                    "message": "pip not found in virtual environment",
                    "data": {}
                }
            
            # Install requirements
            result = subprocess.run([
                str(pip_path), "install", "-r", str(self.requirements_file)
            ], capture_output=True, text=True)
            
            if result.returncode == 0:
                return {
                    "success": True,
                    "message": "Dependencies installed successfully",
                    "data": {
                        "requirements_file": str(self.requirements_file),
                        "output": result.stdout
                    }
                }
            else:
                return {
                    "success": False,
                    "message": f"Failed to install dependencies: {result.stderr}",
                    "data": {"error": result.stderr}
                }
                
        except Exception as e:
            return {
                "success": False,
                "message": f"Failed to install dependencies: {e}",
                "data": {}
            }
    
    def cleanup_venv(self) -> Dict[str, Any]:
        """Clean up virtual environment."""
        try:
            if not self.venv_dir.exists():
                return {
                    "success": True,
                    "message": "Virtual environment does not exist",
                    "data": {}
                }
            
            # Remove virtual environment directory
            import shutil
            shutil.rmtree(self.venv_dir)
            
            return {
                "success": True,
                "message": f"Virtual environment cleaned up: {self.venv_dir}",
                "data": {}
            }
            
        except Exception as e:
            return {
                "success": False,
                "message": f"Failed to cleanup virtual environment: {e}",
                "data": {}
            }
    
    def activate_venv(self) -> Dict[str, Any]:
        """Get activation command for virtual environment."""
        if not self.venv_dir.exists():
            return {
                "success": False,
                "message": "Virtual environment does not exist",
                "data": {}
            }
        
        activate_script = self.venv_dir / "bin" / "activate"
        if not activate_script.exists():
            return {
                "success": False,
                "message": "Activation script not found",
                "data": {}
            }
        
        return {
            "success": True,
            "message": "Virtual environment activation command",
            "data": {
                "activate_command": f"source {activate_script}",
                "python_path": str(self.venv_dir / "bin" / "python"),
                "pip_path": str(self.venv_dir / "bin" / "pip")
            }
        }
    
    def check_venv(self) -> Dict[str, Any]:
        """Check virtual environment status."""
        if not self.venv_dir.exists():
            return {
                "success": False,
                "message": "Virtual environment does not exist",
                "data": {"exists": False}
            }
        
        python_path = self.venv_dir / "bin" / "python"
        pip_path = self.venv_dir / "bin" / "pip"
        
        # Check if Python and pip exist
        python_exists = python_path.exists()
        pip_exists = pip_path.exists()
        
        # Get Python version
        python_version = None
        if python_exists:
            try:
                result = subprocess.run([str(python_path), "--version"], 
                                      capture_output=True, text=True)
                if result.returncode == 0:
                    python_version = result.stdout.strip()
            except Exception:
                pass
        
        # Check installed packages
        installed_packages = []
        if pip_exists:
            try:
                result = subprocess.run([str(pip_path), "list"], 
                                      capture_output=True, text=True)
                if result.returncode == 0:
                    # Parse pip list output
                    lines = result.stdout.strip().split('\n')[2:]  # Skip header
                    for line in lines:
                        if line.strip():
                            parts = line.split()
                            if len(parts) >= 2:
                                installed_packages.append({
                                    "package": parts[0],
                                    "version": parts[1]
                                })
            except Exception:
                pass
        
        return {
            "success": True,
            "message": "Virtual environment status",
            "data": {
                "exists": True,
                "python_exists": python_exists,
                "pip_exists": pip_exists,
                "python_version": python_version,
                "installed_packages": installed_packages,
                "venv_path": str(self.venv_dir)
            }
        }
    
    def run_script(self, script_path: str, args: List[str] = None) -> Dict[str, Any]:
        """Run a script in the virtual environment."""
        try:
            if not self.venv_dir.exists():
                return {
                    "success": False,
                    "message": "Virtual environment does not exist",
                    "data": {}
                }
            
            python_path = self.venv_dir / "bin" / "python"
            if not python_path.exists():
                return {
                    "success": False,
                    "message": "Python not found in virtual environment",
                    "data": {}
                }
            
            # Build command
            cmd = [str(python_path), script_path]
            if args:
                cmd.extend(args)
            
            # Run script
            result = subprocess.run(cmd, capture_output=True, text=True)
            
            return {
                "success": result.returncode == 0,
                "message": f"Script execution {'succeeded' if result.returncode == 0 else 'failed'}",
                "data": {
                    "returncode": result.returncode,
                    "stdout": result.stdout,
                    "stderr": result.stderr,
                    "command": " ".join(cmd)
                }
            }
            
        except Exception as e:
            return {
                "success": False,
                "message": f"Failed to run script: {e}",
                "data": {}
            }


def main():
    """Main CLI interface for Python environment management."""
    parser = argparse.ArgumentParser(description="Python Environment Manager")
    parser.add_argument('command', choices=['create', 'install', 'cleanup', 'activate', 'check', 'run'],
                       help='Command to execute')
    parser.add_argument('--force', action='store_true',
                       help='Force recreation of virtual environment')
    parser.add_argument('--script', help='Script to run (for run command)')
    parser.add_argument('--args', nargs='*', help='Arguments for script')
    parser.add_argument('--output', '-o', choices=['json', 'text'], default='json',
                       help='Output format')
    
    args = parser.parse_args()
    
    # Load configuration
    config = {
        'venvDirectory': 'venv',
        'requirementsFile': 'requirements.txt',
        'pythonVersion': '3.8',
        'autoCleanup': True
    }
    
    manager = PythonEnvManager(config)
    
    try:
        if args.command == 'create':
            result = manager.create_venv(force=args.force)
            
        elif args.command == 'install':
            result = manager.install_dependencies()
            
        elif args.command == 'cleanup':
            result = manager.cleanup_venv()
            
        elif args.command == 'activate':
            result = manager.activate_venv()
            
        elif args.command == 'check':
            result = manager.check_venv()
            
        elif args.command == 'run':
            if not args.script:
                print("Error: --script is required for run command")
                sys.exit(1)
            result = manager.run_script(args.script, args.args)
        
        if args.output == 'json':
            print(json.dumps(result, indent=2))
        else:
            if result["success"]:
                print(f"✅ {result['message']}")
                if 'data' in result:
                    data = result['data']
                    if 'venv_path' in data:
                        print(f"  Virtual environment: {data['venv_path']}")
                    if 'python_path' in data:
                        print(f"  Python path: {data['python_path']}")
                    if 'activate_command' in data:
                        print(f"  Activate command: {data['activate_command']}")
                    if 'installed_packages' in data:
                        print(f"  Installed packages: {len(data['installed_packages'])}")
            else:
                print(f"❌ {result['message']}")
    
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main() 
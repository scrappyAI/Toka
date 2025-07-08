#!/usr/bin/env python3
"""
Python Environment Cleanup Script

Cleans up Python virtual environment remnants and manages environment properly.
"""

import os
import sys
import shutil
import subprocess
from pathlib import Path
from typing import List, Dict, Any, Optional


def find_venv_remnants(directory: Path) -> List[Path]:
    """Find virtual environment remnants in the directory."""
    remnants = []
    
    # Common virtual environment directory names
    venv_names = [
        'venv', 'env', '.venv', '.env', 'virtualenv',
        'python_env', 'python-env', 'pyenv'
    ]
    
    # Common virtual environment files
    venv_files = [
        'pyvenv.cfg', 'activate', 'activate.bat', 'activate.ps1',
        'python.exe', 'python', 'pip.exe', 'pip'
    ]
    
    for item in directory.rglob('*'):
        if item.is_dir():
            # Check for virtual environment directories
            if item.name in venv_names:
                remnants.append(item)
            # Check for bin/lib directories that indicate venv
            elif item.name in ['bin', 'lib', 'lib64', 'Scripts']:
                parent = item.parent
                if (parent / 'pyvenv.cfg').exists() or (parent / 'activate').exists():
                    if parent not in remnants:
                        remnants.append(parent)
        
        elif item.is_file():
            # Check for virtual environment configuration files
            if item.name in venv_files:
                parent = item.parent
                if parent not in remnants:
                    remnants.append(parent)
    
    return remnants


def cleanup_venv_remnants(remnants: List[Path], dry_run: bool = True) -> Dict[str, Any]:
    """Clean up virtual environment remnants."""
    results = {
        "cleaned": [],
        "skipped": [],
        "errors": [],
        "total_size_freed": 0
    }
    
    for remnant in remnants:
        try:
            if dry_run:
                # Calculate size for dry run
                size = sum(f.stat().st_size for f in remnant.rglob('*') if f.is_file())
                results["cleaned"].append({
                    "path": str(remnant),
                    "size": size,
                    "dry_run": True
                })
                results["total_size_freed"] += size
            else:
                # Actually remove the remnant
                size = sum(f.stat().st_size for f in remnant.rglob('*') if f.is_file())
                shutil.rmtree(remnant)
                results["cleaned"].append({
                    "path": str(remnant),
                    "size": size,
                    "dry_run": False
                })
                results["total_size_freed"] += size
                
        except Exception as e:
            results["errors"].append({
                "path": str(remnant),
                "error": str(e)
            })
    
    return results


def create_proper_venv(venv_dir: Path, requirements_file: Optional[Path] = None) -> Dict[str, Any]:
    """Create a proper virtual environment."""
    try:
        # Remove existing venv if it exists
        if venv_dir.exists():
            shutil.rmtree(venv_dir)
        
        # Create new virtual environment
        import venv as venv_module
        venv_module.create(venv_dir, with_pip=True)
        
        # Install requirements if provided
        if requirements_file and requirements_file.exists():
            pip_path = venv_dir / "bin" / "pip"
            if pip_path.exists():
                subprocess.run([
                    str(pip_path), "install", "-r", str(requirements_file)
                ], check=True)
        
        return {
            "success": True,
            "message": f"Virtual environment created at {venv_dir}",
            "data": {
                "venv_path": str(venv_dir),
                "python_path": str(venv_dir / "bin" / "python"),
                "pip_path": str(venv_dir / "bin" / "pip")
            }
        }
        
    except Exception as e:
        return {
            "success": False,
            "message": f"Failed to create virtual environment: {e}",
            "data": {}
        }


def main():
    """Main cleanup function."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Clean up Python virtual environment remnants")
    parser.add_argument('--directory', '-d', default='.',
                       help='Directory to search for remnants')
    parser.add_argument('--dry-run', action='store_true', default=True,
                       help='Show what would be cleaned without actually cleaning')
    parser.add_argument('--force', action='store_true',
                       help='Actually perform cleanup (overrides --dry-run)')
    parser.add_argument('--create-venv', action='store_true',
                       help='Create a new proper virtual environment after cleanup')
    parser.add_argument('--output', '-o', choices=['json', 'text'], default='json',
                       help='Output format')
    
    args = parser.parse_args()
    
    # Determine if this is a dry run
    dry_run = args.dry_run and not args.force
    
    # Find remnants
    directory = Path(args.directory)
    remnants = find_venv_remnants(directory)
    
    if not remnants:
        result = {
            "success": True,
            "message": "No virtual environment remnants found",
            "data": {"remnants": []}
        }
    else:
        # Clean up remnants
        cleanup_results = cleanup_venv_remnants(remnants, dry_run=dry_run)
        
        result = {
            "success": True,
            "message": f"Found {len(remnants)} virtual environment remnants",
            "data": {
                "remnants": [str(r) for r in remnants],
                "cleanup_results": cleanup_results,
                "dry_run": dry_run
            }
        }
    
    # Create new venv if requested
    if args.create_venv:
        venv_dir = directory / "venv"
        requirements_file = directory / "requirements.txt"
        
        venv_result = create_proper_venv(venv_dir, requirements_file)
        result["data"]["new_venv"] = venv_result
    
    # Output results
    if args.output == 'json':
        import json
        print(json.dumps(result, indent=2))
    else:
        print(f"🔍 {result['message']}")
        
        if 'data' in result and 'remnants' in result['data']:
            remnants = result['data']['remnants']
            if remnants:
                print(f"\n📁 Found {len(remnants)} remnants:")
                for remnant in remnants:
                    print(f"  - {remnant}")
            else:
                print("\n✅ No remnants found")
        
        if 'data' in result and 'cleanup_results' in result['data']:
            cleanup = result['data']['cleanup_results']
            if cleanup['cleaned']:
                print(f"\n🧹 Cleaned {len(cleanup['cleaned'])} remnants:")
                for item in cleanup['cleaned']:
                    size_mb = item['size'] / (1024 * 1024)
                    print(f"  - {item['path']} ({size_mb:.2f} MB)")
                print(f"📊 Total space freed: {cleanup['total_size_freed'] / (1024 * 1024):.2f} MB")
            
            if cleanup['errors']:
                print(f"\n❌ {len(cleanup['errors'])} errors:")
                for error in cleanup['errors']:
                    print(f"  - {error['path']}: {error['error']}")
        
        if 'data' in result and 'new_venv' in result['data']:
            venv_result = result['data']['new_venv']
            if venv_result['success']:
                print(f"\n✅ {venv_result['message']}")
            else:
                print(f"\n❌ {venv_result['message']}")


if __name__ == "__main__":
    main() 
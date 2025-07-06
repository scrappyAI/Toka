#!/usr/bin/env python3
"""
Setup script for Toka Analysis Tools
"""

from setuptools import setup, find_packages
from pathlib import Path

# Read README
readme_path = Path(__file__).parent / "README.md"
if readme_path.exists():
    with open(readme_path, "r", encoding="utf-8") as f:
        long_description = f.read()
else:
    long_description = "Toka Analysis Tools - Code analysis and visualization for the Toka agentic OS"

# Read requirements
requirements_path = Path(__file__).parent / "requirements.txt"
if requirements_path.exists():
    with open(requirements_path, "r", encoding="utf-8") as f:
        requirements = [
            line.strip() for line in f 
            if line.strip() and not line.startswith("#")
        ]
else:
    requirements = [
        "graphviz>=0.20.1",
        "toml>=0.10.2", 
        "aiofiles>=23.2.1",
        "PyYAML>=6.0.1"
    ]

setup(
    name="toka-analysis-tools",
    version="0.2.1",
    description="Code analysis and visualization tools for the Toka agentic OS",
    long_description=long_description,
    long_description_content_type="text/markdown",
    author="Toka Team",
    author_email="team@toka.dev",
    url="https://github.com/ScrappyAI/toka",
    packages=find_packages(),
    python_requires=">=3.8",
    install_requires=requirements,
    extras_require={
        "mcp": ["mcp>=0.1.0"],
        "dev": [
            "pytest>=7.0",
            "pytest-asyncio>=0.21",
            "black>=23.0",
            "isort>=5.0",
            "mypy>=1.0",
            "flake8>=6.0",
        ],
        "all": ["mcp>=0.1.0"],
    },
    entry_points={
        "console_scripts": [
            "toka-analysis=toka_analysis_tools.cli:cli_main",
            "toka-mcp-server=toka_analysis_tools.mcp_server:main",
        ],
    },
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: Apache Software License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Topic :: Software Development :: Libraries :: Python Modules",
        "Topic :: Software Development :: Quality Assurance",
        "Topic :: Software Development :: Testing",
        "Topic :: Text Processing :: Markup",
    ],
    keywords=[
        "code-analysis",
        "visualization", 
        "control-flow",
        "dependency-graph",
        "rust",
        "mermaid",
        "toka",
        "agentic-os",
        "mcp",
        "cursor"
    ],
    include_package_data=True,
    zip_safe=False,
    project_urls={
        "Bug Reports": "https://github.com/ScrappyAI/toka/issues",
        "Source": "https://github.com/ScrappyAI/toka",
        "Documentation": "https://github.com/ScrappyAI/toka/blob/main/README.md",
    },
)
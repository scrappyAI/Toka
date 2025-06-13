#!/usr/bin/env python3

"""
Toka Prompt Manager

A CLI tool for managing and using prompts from the Toka prompt library.
"""

import os
import sys
import json
import argparse
from pathlib import Path
from typing import List, Dict, Optional
import re

class PromptManager:
    def __init__(self, prompts_dir: str = "prompts"):
        self.prompts_dir = Path(prompts_dir)
        if not self.prompts_dir.exists():
            raise FileNotFoundError(f"Prompts directory not found: {prompts_dir}")

    def list_prompts(self, category: Optional[str] = None) -> List[str]:
        """List all available prompts, optionally filtered by category."""
        prompts = []
        for root, _, files in os.walk(self.prompts_dir):
            if category and not root.endswith(category):
                continue
            for file in files:
                if file.endswith('.md') and not file.startswith('_'):
                    rel_path = os.path.relpath(os.path.join(root, file), self.prompts_dir)
                    prompts.append(rel_path)
        return sorted(prompts)

    def get_prompt(self, prompt_path: str) -> str:
        """Get the content of a specific prompt."""
        full_path = self.prompts_dir / prompt_path
        if not full_path.exists():
            raise FileNotFoundError(f"Prompt not found: {prompt_path}")
        return full_path.read_text()

    def search_prompts(self, query: str) -> List[str]:
        """Search prompts by content or tags."""
        results = []
        for root, _, files in os.walk(self.prompts_dir):
            for file in files:
                if file.endswith('.md') and not file.startswith('_'):
                    full_path = os.path.join(root, file)
                    content = Path(full_path).read_text()
                    if query.lower() in content.lower():
                        rel_path = os.path.relpath(full_path, self.prompts_dir)
                        results.append(rel_path)
        return sorted(results)

    def get_prompt_metadata(self, prompt_path: str) -> Dict:
        """Extract metadata from a prompt file."""
        content = self.get_prompt(prompt_path)
        metadata = {
            'title': '',
            'summary': '',
            'tags': [],
            'usage': ''
        }

        # Extract title
        title_match = re.search(r'^# (.+)$', content, re.MULTILINE)
        if title_match:
            metadata['title'] = title_match.group(1)

        # Extract summary
        summary_match = re.search(r'## Summary\n(.+?)(?=\n##|\n---|\n$)', content, re.DOTALL)
        if summary_match:
            metadata['summary'] = summary_match.group(1).strip()

        # Extract tags
        tags_match = re.search(r'## Tags\n(.+?)(?=\n##|\n---|\n$)', content, re.DOTALL)
        if tags_match:
            metadata['tags'] = [tag.strip() for tag in tags_match.group(1).split(',')]

        # Extract usage
        usage_match = re.search(r'## Usage\n(.+?)(?=\n##|\n---|\n$)', content, re.DOTALL)
        if usage_match:
            metadata['usage'] = usage_match.group(1).strip()

        return metadata

def main():
    parser = argparse.ArgumentParser(description="Toka Prompt Manager")
    subparsers = parser.add_subparsers(dest='command', help='Commands')

    # List command
    list_parser = subparsers.add_parser('list', help='List available prompts')
    list_parser.add_argument('--category', help='Filter by category')

    # Use command
    use_parser = subparsers.add_parser('use', help='Use a specific prompt')
    use_parser.add_argument('prompt_path', help='Path to the prompt')

    # Search command
    search_parser = subparsers.add_parser('search', help='Search prompts')
    search_parser.add_argument('query', help='Search query')

    # Info command
    info_parser = subparsers.add_parser('info', help='Get prompt metadata')
    info_parser.add_argument('prompt_path', help='Path to the prompt')

    args = parser.parse_args()
    manager = PromptManager()

    try:
        if args.command == 'list':
            prompts = manager.list_prompts(args.category)
            for prompt in prompts:
                print(prompt)
        elif args.command == 'use':
            content = manager.get_prompt(args.prompt_path)
            print(content)
        elif args.command == 'search':
            results = manager.search_prompts(args.query)
            for result in results:
                print(result)
        elif args.command == 'info':
            metadata = manager.get_prompt_metadata(args.prompt_path)
            print(json.dumps(metadata, indent=2))
        else:
            parser.print_help()
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == '__main__':
    main() 
# Toka Prompt Library

A structured, versioned collection of prompts for Toka development, organized by category and purpose.

## 📚 Structure

```
/prompts
  ├── README.md                      # This file
  ├── _common.md                     # Shared context and terminology
  ├── pricing/                       # Pricing-related prompts
  │   └── pricing_engine_v1.md
  ├── pubsub/                        # Pub/Sub and event-related prompts
  │   └── schema_migration.md
  ├── testing/                       # Testing-related prompts
  │   └── unit_testing.md
  └── tools/                         # CLI and utility tools
      └── prompt_manager.py
```

## 🎯 Purpose

This prompt library serves as a centralized repository for:
- Reusable development prompts
- Documentation templates
- Code generation patterns
- Best practices and guidelines

## 🚀 Usage

### Via CLI

```bash
# List all available prompts
python tools/prompt_manager.py list

# Use a specific prompt
python tools/prompt_manager.py use pricing/pricing_engine_v1

# Search prompts by tag
python tools/prompt_manager.py search "schema migration"
```

### Via Cursor

1. Open the desired prompt file
2. Copy the content
3. Paste into your Cursor Context.
4. or use @filname here.

## 📝 Prompt Format

Each prompt file follows this structure:

```markdown
# Title

## Summary
Brief description of the prompt's purpose and use cases.

## Tags
Comma-separated list of relevant tags.

## Usage
When and how to use this prompt.

---

[Prompt content]
```

## 🔄 Versioning

- Each prompt file is versioned using semantic versioning
- Major versions are tracked in the filename (e.g., `v1`, `v2`)
- Changes are documented in the prompt's content

## 🤝 Contributing

1. Create a new prompt file in the appropriate category
2. Follow the standard format
3. Add relevant tags
4. Update this README if adding new categories

## 📋 Categories

- **pricing/**: Pricing engine, economics, and financial calculations
- **pubsub/**: Event-driven architecture, message patterns, and schema management
- **testing/**: Unit testing, integration testing, and test patterns
- **tools/**: CLI tools and utilities for prompt management 
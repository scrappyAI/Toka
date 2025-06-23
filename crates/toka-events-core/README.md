# Toka Events Core

Core event system primitives and type definitions for the Toka platform.

## Overview

This crate provides the foundational types and interfaces for the Toka event system. It defines the core event primitives that are used across the platform for inter-component communication.

## Features

- Event type definitions and traits
- Event metadata and context structures
- Serialization/deserialization support
- Lightweight, dependency-free design

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
toka-events-core = "0.1.0"
```

## Design Philosophy

The crate is designed with the following principles:
- Minimal dependencies for maximum compatibility
- Strong typing for event safety
- Extensible architecture for custom event types
- Serialization support for persistence and transmission

## License

This project is licensed under either of:
- MIT License
- Apache License 2.0

at your option. 
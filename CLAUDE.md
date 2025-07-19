# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Banshee (Emotional Agents Framework) is a Rust-based AI agent system implementing emotional intelligence using the OCC psychological model. The project features 22 discrete emotions with realistic temporal decay, personality systems based on Big Five traits, and modern AI integration through MCP and AI SDK 5 beta.

## Key Commands

### Building and Running
```bash
cargo build              # Build debug version
cargo build --release    # Build optimized release
cargo run               # Run the application
cargo test --all        # Run all tests
```

### Code Quality
```bash
./scripts/pre-commit.sh  # Run full quality checks (format, lint, test, audit)
cargo fmt --all         # Format all code
cargo clippy --all-targets --all-features -- -D warnings  # Lint with all warnings as errors
```

### Development Workflow
```bash
cargo watch -x check -x test -x run  # Live development with auto-reload
cargo test -p <crate_name>           # Test specific crate
cargo build -p <crate_name>          # Build specific crate
```

## Architecture

The project is undergoing migration from a legacy crate structure to a modern plugin-based architecture:

### Modern Plugin Architecture (`packages/`)
- **core**: Core abstractions and plugin interfaces
- **runtime**: Main runtime engine orchestrating plugins
- **cli**: Command-line interface
- **plugin-bootstrap**: Basic agent functionality
- **plugin-emotion**: Emotional intelligence implementation
- **plugin-memory**: Memory and persistence layer
- **plugin-providers**: AI provider integrations
- **plugin-web3**: Web3/blockchain integration

### Legacy Crates (`crates/`) - Being Migrated
- **emotion_engine**: OCC emotion model with 22 emotions and decay mechanics
- **character_sheet**: Agent personality definitions (Big Five traits)
- **mcp_manager**: Model Context Protocol integration
- **agent_runtime**: Actor-based concurrent agent system using Actix

### Key Design Patterns
1. **Actor Model**: Uses Actix for concurrent message-driven architecture
2. **Plugin System**: Modular design allowing extensible functionality
3. **Emotion Decay**: Mathematical models for realistic emotional temporal dynamics
4. **Personality Mapping**: Big Five traits mapped to PAD (Pleasure-Arousal-Dominance) space

## Technical Stack
- **Language**: Rust 2021 edition (MSRV 1.70.0)
- **Async Runtime**: Tokio with full features
- **Actor Framework**: Actix 0.13
- **Databases**: PostgreSQL (via SQLx) + Redis
- **Web**: Actix-web 4.5
- **Testing**: Built-in Rust testing + rstest for parameterized tests

## Development Guidelines

1. **Pre-commit Checks**: Always run `./scripts/pre-commit.sh` before committing
2. **Workspace Structure**: Use `-p <crate_name>` to target specific crates
3. **Error Handling**: Use `anyhow` for applications, `thiserror` for libraries
4. **Async Code**: Prefer `async-trait` for trait implementations
5. **Configuration**: Warnings are errors (`-D warnings` in rustflags)

## Current Migration Status

The project is transitioning from the legacy crate structure to the modern plugin architecture. When working on new features:
- Implement in the `packages/` directory structure
- Follow the plugin interface patterns in `packages/core`
- Maintain compatibility with existing emotion engine and character sheet functionality
# Contributing to Falcon

Thank you for your interest in contributing to Falcon! This document provides guidelines for contributing.

## Getting Started

1. Fork the repository
2. Clone your fork
3. Create a branch for your work
4. Make your changes
5. Submit a pull request

## Development Setup

### Prerequisites

- Rust (latest stable version)
- LLVM 11+ (for code generation)

### Building

```bash
cd compiler
cargo build
```

### Running Tests

```bash
cargo test
```

### Running Examples

```bash
cargo run -- build ../examples/hello_world.fc
```

## Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Use meaningful variable and function names
- Add comments for complex logic
- Write tests for new features

## Design Principles

Before proposing features, please read:
- [Design Principles](DESIGN_PRINCIPLES.md)
- [What Falcon Will Never Do](WHAT_FALCON_WILL_NEVER_DO.md)

## Pull Request Process

1. Ensure your code follows the design principles
2. Add tests if applicable
3. Update documentation if needed
4. Ensure all tests pass
5. Submit PR with clear description

## Areas for Contribution

- Compiler improvements
- Standard library modules
- Documentation
- Examples
- Tests
- Performance optimizations

Thank you for contributing to Falcon! 🦅


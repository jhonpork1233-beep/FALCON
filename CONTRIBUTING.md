# Contributing to Falcon

Falcon is published as an experimental language project. Contributions are welcome, especially when they improve correctness, diagnostics, documentation, tests, and profile enforcement.

## Before Opening a Pull Request

Read these documents first:

- [README.md](README.md)
- [DESIGN_PRINCIPLES.md](DESIGN_PRINCIPLES.md)
- [WHAT_FALCON_WILL_NEVER_DO.md](WHAT_FALCON_WILL_NEVER_DO.md)

The project values conservative engineering decisions over feature volume. If a proposal weakens profile enforcement, hides runtime behavior, or expands the language surface without a clear need, it is unlikely to be accepted.

## Development Setup

Prerequisites:

- Rust stable
- Cargo
- LLVM/Clang available locally for native code generation

Typical workflow:

```bash
cd compiler
cargo check
cargo test
cargo check --features llvm
```

Run an example from source:

```bash
cargo run --features llvm --bin falcon -- ../examples/hello_world.fc
```

## Contribution Guidelines

- keep changes tightly scoped
- prefer existing compiler and runtime patterns over new abstractions
- add or update tests when behavior changes
- update documentation when user-visible behavior changes
- avoid overstating implementation status in comments or docs

## Pull Request Expectations

Please include:

- a short description of the change
- why the change is needed
- test coverage or verification notes
- any limitations that remain after the change

## Good First Contribution Areas

- diagnostics and error messages
- parser and IR tests
- profile-law hardening
- docs cleanup
- example maintenance
- standard-library clarification

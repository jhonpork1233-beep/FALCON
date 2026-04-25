# Falcon Current Capabilities

This document summarizes the implementation status of Falcon as published in this repository. It is intended to be descriptive rather than aspirational.

## Summary

Falcon already has:

- a working lexer, parser, AST, and IR pipeline
- profile-aware compilation for `userland`, `kernel`, and `baremetal`
- explicit import validation and IR import-contract checks
- an LLVM-based native compilation path
- runtime-backed library modules for several userland facilities
- a Python-style `.fpy` front end for userland programs

Falcon does not yet present itself as a finished production language. Several language and safety subsystems remain partial.

## Compiler Status

| Area | Status | Notes |
| --- | --- | --- |
| Lexer | implemented | used by `.fc` and generated `.fpy` sources |
| Parser | implemented | covers the language surface used by the repository examples |
| AST | implemented | used for source-oriented validation and profile filtering |
| Import resolution | implemented | resolves local modules plus routed library modules |
| IR lowering | implemented | main semantic handoff for later validation and code generation |
| LLVM backend | primary backend | main native compilation target |
| C backend | legacy/debug path | kept for inspection and debugging, not the primary target |

## Language Feature Status

| Feature | Status | Notes |
| --- | --- | --- |
| Functions | available | includes freestanding entrypoint patterns |
| Variables and mutability | available | mutable bindings are tracked and reassignment is checked |
| Structs | available | core support exists |
| Enums | available | support exists, but some advanced paths still need hardening |
| Pattern matching | available | implemented, with edge cases still being tightened |
| Traits and impl checks | partial | conformance checks exist, ecosystem is still early |
| Generics | partial | monomorphization exists, inference and specialization still need work |
| Closures | partial | non-capturing closures are the current focus; captured closures are not complete |
| Error propagation | partial | core pieces exist, but the broader error-type story is still evolving |
| Python-style `.fpy` | available | userland only |

## Profile Status

| Profile | Status | Notes |
| --- | --- | --- |
| `userland` | available | hosted runtime and library surface |
| `kernel` | available for freestanding experiments | strict capability limits and explicit imports |
| `baremetal` | available for freestanding experiments | direct-hardware oriented, no hosted runtime |

The profile model is one of Falcon's strongest implemented ideas. Imports and runtime-backed facilities are checked against profile capabilities before code generation.

## Runtime and Library Status

| Area | Status | Notes |
| --- | --- | --- |
| `library/` bindings | available | includes `math`, `io`, `random`, `string`, and `ai` |
| Profile-specific runtime sources | available | separate runtime source selection per profile |
| `stdlib/` | partial | broader than the currently stable implementation surface |
| Ollama bindings | available in userland | exposed through `library/ai` |

## Current Limitations

The following limitations are important:

- ownership verification is useful, but not yet equivalent to a full Rust borrow checker
- lifetime reasoning is incomplete
- destructor/drop behavior is not fully modeled
- generic inference and specialization still need refinement
- captured closures are not a finished feature
- some examples and secondary docs remain under active cleanup

## Recommended Positioning

The most accurate short description for Falcon at this stage is:

> An experimental systems language with compile-time execution profiles for userland, kernel, and baremetal targets.

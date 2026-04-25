# Falcon Design Principles (Immutable)

These principles form the foundation of Falcon and **CANNOT** change. Features may evolve, but these principles remain constant.

## Core Principles

1. **Simple surface, powerful core**
   - The language syntax should be approachable for beginners
   - Complexity is hidden in the compiler, not exposed to users
   - Advanced features are opt-in, not mandatory

2. **Zero mandatory runtime**
   - No garbage collector required
   - Baremetal profile has zero runtime overhead
   - Userland profile has minimal runtime (only what's necessary)

3. **Native-first compilation**
   - LLVM backend for native code generation
   - No interpreter or JIT by default
   - Cross-compilation support from day one

4. **AI as first-class citizen**
   - Tensor types in standard library
   - Streaming operations for LLM inference
   - Model integration APIs
   - But: AI features are libraries, not core syntax

5. **Safety by default, danger by choice**
   - Memory safety is the default
   - `unsafe` keyword is explicit and clear
   - Profiles control safety guarantees

6. **IR is sacred and versioned**
   - Intermediate Representation is versioned (v0.1, v0.2, etc.)
   - IR compatibility is maintained across versions
   - Plugins operate on IR, not syntax
   - IR is the stable interface

7. **If it can be a library, it's not in core**
   - Core language = syntax, ownership, IR, profiles
   - Everything else = standard library or external crates
   - Prevents language bloat
   - Enables ecosystem growth

8. **Explicit over implicit**
   - Type annotations when needed
   - Explicit ownership transfers
   - Clear error messages
   - No hidden magic

9. **Profiles, not keywords**
   - Compilation profiles control behavior
   - Same syntax, different guarantees
   - No keyword explosion
   - Contextual safety levels

10. **Restraint over features**
    - Every feature must justify its existence
    - Prefer composition over language features
    - Say "no" to feature requests that break principles
    - Quality over quantity

## What This Means in Practice

### ✅ IN CORE LANGUAGE:
- Ownership system (move, borrow, lifetime)
- Tasks + channels (concurrency primitives)
- `unsafe` keyword (explicit unsafe blocks)
- IR primitives (versioned, stable)
- Profile system (userland/kernel/baremetal)
- Basic types (integers, floats, bool, strings)
- Control flow (if, while, for, match)
- Functions, structs, enums
- Error handling (Result, Option)

### ❌ NOT IN CORE (must be external):
- Notebooks → tooling (separate tool)
- JIT compilation → optional backend
- GPU kernels → library + IR pass
- DSLs → macros or plugins
- Package manager → separate tool (`falcon pkg`)
- IDE features → LSP server
- Web frameworks → standard library or crates
- ORMs → external libraries
- Testing framework → standard library module

## Enforcement

These principles are enforced through:
1. **Code review**: Every feature PR must justify alignment with principles
2. **Documentation**: Principles are referenced in all design decisions
3. **Community**: Community understands and upholds these principles
4. **Versioning**: Breaking these principles requires a major version bump

## Evolution

Principles may be **clarified** but never **changed**. If a principle becomes outdated, it indicates a fundamental shift in language direction, requiring a major version (e.g., Falcon 2.0).

---

**Last Updated**: 2024-12-29  
**Version**: 1.0 (Immutable)


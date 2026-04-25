# What Falcon Will NEVER Do

This document defines hard boundaries for Falcon. These are non-negotiable decisions that will **never** change, regardless of community requests or trends.

## Core Language Decisions

### 1. ❌ Add Garbage Collection

**Decision**: Falcon will never have garbage collection.

**Reason**: GC conflicts with zero-runtime goals, adds unpredictable pauses, and prevents baremetal use.

**Alternatives**: 
- Ownership system (default)
- Reference counting (`Rc`/`Arc`) when needed
- Manual management in unsafe contexts

### 2. ❌ Make `unsafe` "Safe" Through Keywords

**Decision**: `unsafe` will always mean "bypass safety checks."

**Reason**: The meaning of `unsafe` must be clear and unambiguous. Adding "safer unsafe" keywords would confuse the mental model.

**What we do**: Provide clear documentation and profile-specific guarantees, but `unsafe` always means what it says.

### 3. ❌ Break IR Compatibility (Only Versioned Additions)

**Decision**: IR versions are immutable. We add new IR versions, never break existing ones.

**Reason**: IR is the stable interface for plugins, tools, and future compilers. Breaking it would destroy the ecosystem.

**What we do**: 
- IR v0.1, v0.2, v0.3... (additive only)
- Old IR versions remain supported
- Migration tools provided when needed

### 4. ❌ Bake Frameworks Into Core Syntax

**Decision**: No web frameworks, ORMs, or application frameworks in core language.

**Reason**: Frameworks change, languages should last decades. Core syntax must be stable.

**What we do**: 
- Frameworks live in standard library or external crates
- Core provides primitives (HTTP server, not web framework)
- Ecosystem builds on top

### 5. ❌ Sacrifice Simplicity for Features

**Decision**: Every feature must justify its complexity cost.

**Reason**: Language complexity is the enemy of adoption. Simple languages win long-term.

**What we do**: 
- Reject features that add complexity without clear benefit
- Prefer composition over language features
- Say "no" more often than "yes"

### 6. ❌ Copy Other Languages' Complexity

**Decision**: We won't add features just because Rust/C++/others have them.

**Reason**: Other languages' features come with their own problems. We learn from them but don't copy blindly.

**What we do**: 
- Evaluate each feature on its own merits
- Adapt ideas to fit Falcon's philosophy
- Reject features that don't align with principles

### 7. ❌ Promise Magic Optimization

**Decision**: We won't claim the compiler will "magically optimize" code.

**Reason**: Overpromising leads to disappointment. Real optimization requires programmer understanding.

**What we do**: 
- Provide clear performance characteristics
- Document optimization flags
- Be honest about trade-offs
- Let programmers make informed choices

### 8. ❌ Support Every Platform Day 1

**Decision**: We won't try to support all platforms immediately.

**Reason**: Quality over quantity. Better to support a few platforms well than many poorly.

**What we do**: 
- Start with major platforms (Linux, macOS, Windows)
- Add platforms incrementally
- Community contributions for niche platforms
- Clear roadmap for platform support

## Additional Boundaries

### 9. ❌ Add Runtime Reflection

**Decision**: No runtime type information or reflection system.

**Reason**: Adds runtime overhead, complicates compiler, conflicts with zero-runtime goals.

**Alternatives**: 
- Compile-time generics
- Macros for code generation
- Explicit type parameters

### 10. ❌ Add Exceptions

**Decision**: No exception-based error handling.

**Reason**: Exceptions are invisible control flow, add runtime overhead, complicate semantics.

**What we do**: 
- `Result<T, E>` for recoverable errors
- `Option<T>` for optional values
- Explicit error propagation with `?` operator

### 11. ❌ Add Inheritance

**Decision**: No class-based inheritance.

**Reason**: Inheritance leads to fragile base classes, tight coupling, and complexity.

**What we do**: 
- Composition over inheritance
- Traits/interfaces for polymorphism
- Struct embedding for code reuse

### 12. ❌ Add Null Types

**Decision**: No `null` or `nil` in the language.

**Reason**: Null is the "billion-dollar mistake." Optional types are safer.

**What we do**: 
- `Option<T>` for nullable values
- Compiler enforces null-safety
- No null pointer exceptions

### 13. ❌ Add Implicit Conversions

**Decision**: No automatic type conversions.

**Reason**: Implicit conversions hide bugs and make code harder to understand.

**What we do**: 
- Explicit conversions only
- Type inference for same types
- Clear, visible type changes

### 14. ❌ Add Global Variables by Default

**Decision**: No mutable global variables without explicit `unsafe`.

**Reason**: Global mutable state causes data races, makes testing hard, breaks reasoning.

**What we do**: 
- Constants allowed (immutable)
- `static mut` requires `unsafe`
- Prefer passing state explicitly

### 15. ❌ Add Operator Overloading for Everything

**Decision**: Limited, conservative operator overloading.

**Reason**: Excessive overloading makes code unreadable and unpredictable.

**What we do**: 
- Overload only standard operators (`+`, `-`, `*`, `/`, etc.)
- No custom operators
- Clear semantics for each operator

## Why These Boundaries Matter

These boundaries:

1. **Prevent scope creep** - Keep language focused
2. **Maintain simplicity** - Avoid feature bloat
3. **Set expectations** - Community knows what to expect
4. **Enable long-term planning** - Stable foundation
5. **Preserve philosophy** - Uphold design principles

## What This Means for Contributors

When proposing features:

1. ✅ Check if it violates any boundary above
2. ✅ Ensure it aligns with design principles
3. ✅ Justify why it can't be a library
4. ✅ Show how it maintains simplicity
5. ✅ Demonstrate long-term value

**If a feature violates boundaries, it will be rejected, regardless of popularity.**

## Evolution vs. Revolution

Falcon will **evolve** (additive changes) but never **revolutionize** (breaking changes to core principles).

- ✅ Add new standard library modules
- ✅ Add new IR versions (additive)
- ✅ Improve compiler optimizations
- ✅ Add new profiles (if justified)
- ❌ Change ownership model
- ❌ Add garbage collection
- ❌ Break IR compatibility
- ❌ Remove profiles

---

**These boundaries are non-negotiable. They define what Falcon is and what it will never become.**

---

**Last Updated**: 2024-12-29  
**Version**: 1.0


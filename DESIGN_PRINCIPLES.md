# Falcon Design Principles

This document records the design principles that currently guide Falcon's implementation and documentation.

## 1. Profiles define legality

Falcon treats execution context as a compile-time rule. The selected profile determines what code is legal, which imports are valid, and which runtime surfaces are available.

## 2. AST and IR serve different jobs

Falcon keeps an AST for source-oriented work such as parsing, profile filtering, and import resolution. It lowers into IR for semantic validation and backend handoff.

## 3. Runtime behavior should be explicit

Falcon avoids relying on hidden runtime assumptions. Runtime-backed facilities should come from explicit imports and profile-aware resolution, not backend guesswork.

## 4. The same language should span hosted and freestanding code

Falcon aims to use one language surface across `userland`, `kernel`, and `baremetal`, while allowing the compiler to reject operations that do not fit the active environment.

## 5. Backends should implement semantics, not invent them

Language behavior should be determined before code generation. Backends should translate verified intent rather than deciding correctness on their own.

## 6. Language growth should be conservative

New features should earn their complexity. Falcon prefers a smaller, clearer core over expanding the language surface to imitate larger ecosystems.

## 7. Documentation should describe reality

Public documentation should reflect the current implementation, note significant gaps, and avoid presenting internal milestones as finished guarantees.

# What Falcon Will Never Do

This document records the project constraints and non-goals that currently define Falcon's direction.

## 1. Hide execution context behind convenience

Falcon is built around explicit profiles. The language should not erase the difference between hosted and freestanding environments behind silent fallbacks.

## 2. Depend on hidden runtime magic

Runtime-backed behavior should come from explicit imports and profile-aware compilation, not from backend-only knowledge.

## 3. Treat backend behavior as language law

Code generation backends should implement validated semantics, not decide them.

## 4. Put application frameworks into the core language

Falcon may grow libraries and tooling, but the core language should remain smaller and more stable than the surrounding ecosystem.

## 5. Market unfinished guarantees as complete

The repository should not claim finished memory-safety, ownership, or platform coverage before those guarantees are actually implemented and maintained.

## 6. Expand the language without a strong need

Feature growth should remain conservative. Complexity in the language core has a long maintenance cost.

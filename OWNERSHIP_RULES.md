# Falcon Ownership Rules

This page is a short reference for the ownership model Falcon is aiming to enforce.

## Core Rules

1. Every value has one active owner.
2. Ownership can be moved to another binding or function.
3. Values can be borrowed instead of moved.
4. Borrows must not outlive the value they refer to.
5. Mutable and immutable borrowing must not conflict.

## Examples

### Move

```falcon
let value = data;
consume(value);
```

### Immutable Borrow

```falcon
let view = &data;
```

### Mutable Borrow

```falcon
let mut data = numbers;
let edit = &mut data;
```

## Current Implementation Note

The compiler in this repository performs ownership-related checks, but the implementation is still partial. This document should therefore be read as the intended model and current direction, not as a claim that every rule is already enforced with Rust-level completeness.

# Falcon Memory Model

This document describes the memory model Falcon is working toward and the parts that are already reflected in the current compiler implementation.

## High-Level Model

Falcon is designed around:

- ownership by default
- explicit mutation
- borrowing for shared or exclusive access
- profile-aware access to runtime and allocation facilities

## Ownership

The intended model is move-first:

```falcon
let value = data;
consume(value);
```

After ownership is moved, the previous binding should no longer be used.

## Borrowing

Falcon's intended ownership rules follow the familiar shape:

- multiple immutable borrows are allowed
- one mutable borrow is allowed
- mutable and immutable borrows must not overlap incompatibly

Example:

```falcon
let mut data = numbers;
let view = &data;
```

## Mutability

Mutable reassignment requires `let mut`.

```falcon
let mut counter = 0;
counter = counter + 1;
```

## Profile Differences

### Userland

- hosted runtime available
- runtime-backed allocation facilities available
- userland bounds checks are inserted for indexed access

### Kernel

- no hosted runtime
- no default hosted allocation model
- freestanding restrictions apply

### Baremetal

- no hosted runtime
- direct-hardware oriented
- minimal assumptions

## Arrays and Indexing

Falcon currently includes userland bounds-check insertion for indexed access. This is an implementation detail of the current compiler rather than a claim that the full memory-safety story is complete.

## Unsafe Code

`unsafe` is the explicit escape hatch for lower-level operations such as raw pointer access. It does not, by itself, grant profile capabilities that the active build is not allowed to use.

## Implementation Status

The current compiler includes ownership-related verification, but the memory model should still be described carefully:

- move and borrow checks exist
- mutability is enforced
- profile rules affect available runtime behavior
- full lifetime reasoning is not finished
- destructor/drop semantics are not fully modeled

The repository therefore does not claim complete Rust-equivalent memory safety.

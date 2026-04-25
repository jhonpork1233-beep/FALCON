# Falcon Memory Specification

This document describes the current memory model direction used by the Falcon compiler and runtime sources in this repository.

## Core Direction

Falcon is designed around:

- ownership by default
- explicit mutation
- borrowing rather than implicit aliasing
- profile-aware access to allocation and runtime facilities

## Regions and Allocation

The language distinguishes, conceptually, between:

- stack-resident values
- runtime-backed allocation in hosted builds
- freestanding code paths without a hosted heap/runtime model

In practice:

- `userland` can use hosted runtime-backed facilities
- `kernel` and `baremetal` reject hosted allocation paths by profile rule

## Borrowing and Mutation

Falcon aims for the standard ownership shape:

- multiple immutable borrows
- one mutable borrow
- explicit mutation through mutable bindings

The current compiler performs ownership-related verification, but the implementation remains partial.

## Arrays and Bounds

Userland builds include compiler-inserted bounds checks for indexed access. This improves safety in hosted builds, but it should not be confused with a complete end-to-end memory-safety proof.

## Status

This is the current specification direction for the repository. It should be read as an implementation-oriented document for an active language project, not as a final language standard.

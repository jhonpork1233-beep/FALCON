# Falcon Error Model

This document describes the current error-handling direction used by the Falcon repository.

## Principles

Falcon prefers:

- explicit error flow
- return-value based recoverable errors
- profile-aware treatment of unrecoverable failures

## Recoverable Errors

Recoverable failures are expected to use explicit result values rather than hidden exception machinery.

Typical shape:

```falcon
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

The repository also includes examples and parser/codegen support around propagation-oriented patterns such as `?`.

## Unrecoverable Errors

Hosted builds may use panic-like abort behavior through runtime-backed facilities. Freestanding profiles are stricter and reject hosted panic paths that do not fit the active execution environment.

## Compile-Time Diagnostics

Falcon already produces compile-time errors in several areas:

- lexing and parsing
- profile violations
- import-contract violations
- undefined calls
- ownership-related misuse detected by the current verification pass

## Status

The core philosophy is clear: error behavior should be explicit and profile-aware. The repository should still be described as evolving rather than as a finalized language standard.

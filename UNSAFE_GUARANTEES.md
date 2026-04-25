# Unsafe Guarantees by Profile

This document describes what `unsafe` means in the current Falcon repository.

## General Meaning

`unsafe` marks code that opts out of some compiler-enforced safety checks and uses lower-level operations directly.

Typical examples include:

- raw pointer access
- volatile operations
- low-level FFI boundaries

`unsafe` is explicit. It does not silently expand profile capabilities.

## Userland

In `userland`, `unsafe` can be used for lower-level operations while still running inside a hosted build. Runtime-backed facilities remain available through normal imports.

Important distinction:

- `unsafe` does not remove the need for explicit imports
- `unsafe` does not turn `userland` into `kernel` or `baremetal`

## Kernel

In `kernel`, `unsafe` is expected in lower-level code paths, but the profile still rejects hosted runtime surfaces that do not belong in freestanding code.

## Baremetal

In `baremetal`, `unsafe` often appears alongside direct hardware access. The profile remains freestanding and does not inherit hosted runtime support.

## Current Caveat

Falcon's unsafe story is meaningful but not final. The language should not yet be marketed as having a finished, formally complete unsafe model across every subsystem.

# Falcon Kernel Scope

This document explains what the current `kernel` profile is intended for and what it should not yet be described as supporting.

## Current Positioning

The `kernel` profile is best understood as a freestanding profile for:

- kernel-adjacent components
- low-level services
- device-facing code
- experiments that need stricter capability boundaries than userland

It should not currently be marketed as a complete kernel-development environment.

## What the Profile Enforces

In broad terms, the `kernel` profile rejects hosted facilities that belong to userland, including runtime-backed imports and OS-facing helpers that require capabilities not allowed by the profile.

Current expectations include:

- freestanding entrypoints
- explicit imports
- no hosted runtime
- no default heap/runtime/OS/thread capabilities
- hardware boundary or low-level interaction appropriate to freestanding code

## Appropriate Use Cases

Reasonable uses for the current profile include:

- low-level control loops
- memory-mapped I/O wrappers
- freestanding experiments
- driver-like components
- runtime and platform bring-up work

## Areas That Still Require Care

The current profile does not provide a complete operating-system framework. Contributors should expect manual work around:

- architecture-specific bring-up
- interrupt handling details
- allocator strategy
- ABI boundaries
- platform integration

## When to Use Baremetal Instead

Use `baremetal` rather than `kernel` when the program is primarily:

- firmware
- boot code
- direct register-level control
- minimal freestanding startup code

## Documentation Rule

Public descriptions of the `kernel` profile should remain conservative. The current implementation is meaningful and useful, but it is still part of an experimental language project.

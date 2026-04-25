# Falcon Runtime Specification

This document describes the runtime boundary used by the current Falcon repository.

## Principle

Falcon keeps runtime concerns profile-specific rather than pretending that every build has the same execution environment.

## Userland

`userland` builds use hosted runtime support. In the current repository, this includes runtime-backed facilities used by:

- console and string helpers
- selected file and process helpers
- userland library bindings
- hosted AI/Ollama integration

## Kernel

`kernel` builds use a freestanding runtime source rather than the hosted userland runtime. The purpose is to keep the build environment compatible with lower-level code and to reject hosted facilities that do not belong in that profile.

## Baremetal

`baremetal` builds also use a freestanding runtime source. The profile is intended for direct-hardware oriented programs and does not assume the hosted userland runtime.

## Runtime Source Selection

The CLI selects runtime source files by profile during the build pipeline. This is a real part of the current implementation.

## Status

The repository has meaningful runtime separation today, but the public documentation should still present the runtime model as evolving rather than complete.

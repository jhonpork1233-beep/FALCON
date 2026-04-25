# Falcon Repository Overview

This document summarizes the major components currently present in the Falcon repository.

## Compiler

The repository contains a compiler pipeline with:

- lexer
- parser
- AST
- import resolution
- IR lowering
- profile-aware validation
- LLVM-based native code generation

## Runtime

The compiler ships with profile-specific runtime source files for:

- userland
- kernel
- baremetal

These runtime sources back the current compilation flow and library bindings.

## Libraries

The `library/` tree contains runtime-backed Falcon modules such as:

- `math`
- `io`
- `random`
- `string`
- `ai`

## Examples

The repository includes:

- userland examples
- freestanding examples
- multi-profile examples
- Python-style `.fpy` examples
- Ollama integration demos

## Documentation

The repository includes:

- project overview documents
- design notes
- import and ownership documentation
- technical documentation under `docs/`

## Tooling

The repository also includes the `.fpy` transpiler under `tools/fpy_transpiler/`.

## Implementation Status

The repository is substantial and runnable, but still early. It should be read as an active compiler project rather than as a finished language release.

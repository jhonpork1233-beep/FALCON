# Falcon Import System Specification

This document describes the import behavior implemented in the Falcon compiler in this repository.

## Goals

The import system is designed to:

- keep module access explicit
- remain profile-aware
- prevent imports from bypassing profile restrictions
- avoid backend-only symbol inference

## Import Syntax

Basic forms:

```falcon
import string;
import random::raw;
import net::http;
```

Selector form:

```falcon
import crypto::{sha256, hmac};
```

## Resolution Model

The compiler resolves imports from:

- the source tree relative to the current file
- `library/`
- `stdlib/`

Some modules are treated as routed modules. In `userland`, a plain module import such as `import string;` resolves to the userland surface for that module. In freestanding profiles, the same import is rejected unless an explicit profile-safe path is used.

## Profile-Aware Routing

Examples:

- `import string;` in `userland` resolves to the userland surface
- `import string;` in `kernel` is rejected
- `import string::raw;` remains the low-level binding surface

The active profile determines legality. Import syntax selects symbols; it does not grant capabilities on its own.

## Import Contract

Falcon enforces imports in multiple stages:

1. source-level import validation
2. import resolution and cycle detection
3. missing-import linting for runtime-backed symbols
4. IR import-contract validation

This is intended to prevent code generation backends from quietly accepting undeclared runtime facilities.

## Raw Imports

`::raw` modules are the low-level binding surface. They are intended for explicit, lower-level use where profile rules permit them.

Example:

```falcon
import random::raw;
```

## Developer Notes

Useful commands:

```bash
falcon check file.fc --dump-imports
falcon build file.fc --strict-imports
```

## Current Limitations

- the routed-library surface is more mature than parts of `stdlib/`
- some library areas are still evolving
- import rules are real, but the surrounding library ecosystem is not final

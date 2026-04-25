# Falcon Library Bindings

The `library/` directory contains Falcon modules that bind to runtime-backed functionality implemented under `compiler/runtime/`.

## Typical Layout

```text
library/
  <module>/
    mod.fc
    raw.fc
    kernel.fc
    baremetal.fc
```

Not every module uses every file variant, but the layout reflects the intended split between:

- userland-facing API surfaces
- lower-level raw bindings
- optional profile-specific surfaces

## Current Modules

The repository currently includes bindings such as:

- `math`
- `io`
- `random`
- `string`
- `ai`

## Import Usage

Bindings are expected to be used through explicit imports:

```falcon
import random;
import random::raw;
```

## Adding a New Binding

Typical workflow:

1. implement the runtime-backed function in `compiler/runtime/`
2. declare it in `compiler/runtime/falcon_runtime.h`
3. add Falcon bindings under `library/<module>/`
4. keep profile rules aligned with the import system

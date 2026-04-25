# Falcon Library Binding System

The `library/` directory contains Falcon module bindings for runtime symbols implemented in
`compiler/runtime/falcon_runtime.c`.

## Canonical Layout

```
library/
  <module>/
    mod.fc        # userland-facing module surface
    raw.fc        # low-level binding surface
    kernel.fc     # optional kernel-safe surface
    baremetal.fc  # optional baremetal-safe surface
```

Implemented modules:

- `library/math/{mod.fc,raw.fc}`
- `library/io/{mod.fc,raw.fc}`
- `library/random/{mod.fc,raw.fc}`
- `library/string/{mod.fc,raw.fc}`
- `library/ai/{mod.fc,raw.fc}`

## Import-Driven Usage

Bindings should be referenced through explicit imports, not implicit global visibility:

```falcon
import random;

func main() {
    falcon_random_seed(42);
    let n = falcon_randint(1, 10);
    print_int(n);
}
```

Low-level bindings remain available with explicit raw imports:

```falcon
import random::raw;
```

## Adding New Bindings

1. Add C implementation to `compiler/runtime/falcon_runtime.c`.
2. Add declaration to `compiler/runtime/falcon_runtime.h`.
3. Add Falcon module declarations under `library/<module>/`.
4. Keep capability/profile rules aligned with `IMPORT_SYSTEM_SPEC.md`.

## Available Libraries

| Library | Description |
|---------|-------------|
| `math` | math functions (sin, cos, sqrt, pow, log, constants) |
| `io` | file I/O helpers |
| `random` | Python-style random primitives |
| `ai` | local LLM integration bindings |
| `string` | console/string utility bindings |

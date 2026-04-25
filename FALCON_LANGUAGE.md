# Falcon Language Reference

This document is a concise reference for the Falcon language surface represented in this repository. It describes the current implementation surface and major conventions used by the compiler and examples.

## Source Forms

Falcon currently accepts two source forms:

- `.fc` - standard Falcon source
- `.fpy` - Python-style Falcon source for userland programs

`.fpy` files are transpiled to generated Falcon source before entering the main compiler pipeline.

## Compilation Modes

Common CLI entry points:

```bash
falcon <file>
falcon run <file>
falcon build <file>
falcon check <file>
```

Useful flags:

```bash
--profile userland|kernel|baremetal
--profiles all
--emit-ir
--emit-llvm
--emit-c
--strict-imports
--dump-imports
```

## Declarations

### Functions

```falcon
func add(a: i32, b: i32) -> i32 {
    return a + b;
}
```

### External Functions

```falcon
extern func write_volatile(addr: *mut u32, value: u32);
```

### Structs

```falcon
struct Point {
    x: i32,
    y: i32,
}
```

### Enums

```falcon
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### Traits and Implementations

Trait syntax and implementation checks exist in the compiler, although the feature set is still maturing.

```falcon
trait Display {
    func show(self) -> str;
}
```

## Variables and Mutability

Immutable bindings:

```falcon
let value = 42;
```

Mutable bindings:

```falcon
let mut counter = 0;
counter = counter + 1;
```

## Types

Primitive numeric types include:

- signed integers: `i8`, `i16`, `i32`, `i64`, `i128`
- unsigned integers: `u8`, `u16`, `u32`, `u64`, `u128`
- pointer-sized integers: `isize`, `usize`
- floating point: `f32`, `f64`
- `bool`

Other commonly used types in the current surface:

- `str`
- `String`
- `()`
- `!`
- pointers such as `*const T` and `*mut T`

## Control Flow

### Conditionals

```falcon
if ready {
    println("ready");
} else {
    println("waiting");
}
```

### Loops

```falcon
while count < 10 {
    count = count + 1;
}

loop {
    tick();
}
```

### Return

```falcon
return value;
```

## Expressions

The language surface in this repository includes:

- arithmetic expressions
- comparison expressions
- function calls
- field access
- indexing
- casts
- pattern matching

Example:

```falcon
let total = (a + b) * 2;
```

## Imports

Falcon uses explicit imports:

```falcon
import string;
import random;
import ai;
```

The compiler resolves imports relative to:

- the current source tree
- `library/`
- `stdlib/`

Some modules are profile-routed. In `userland`, an import such as `import string;` resolves to a userland-facing module surface. In freestanding profiles, hosted paths are rejected unless an explicit profile-safe path is used.

## Profiles and Entrypoints

### Userland

Typical entrypoint:

```falcon
func main() {
    println("Hello");
}
```

### Kernel

Current examples use a freestanding entry such as:

```falcon
func kernel_main() {
    loop {}
}
```

### Baremetal

Current examples use:

```falcon
func _start() {
    loop {}
}
```

## Python-Style Falcon

Python-style Falcon keeps the same intended semantics while changing the surface syntax:

```python
import string

def main():
    println("Hello")
```

This path is currently limited to `userland`.

## Current Limitations

This repository includes a real language implementation, but some features remain partial:

- ownership verification is not yet a full borrow checker
- generic inference and specialization are still evolving
- captured closures are not complete
- `stdlib/` is broader than the currently stable implementation surface

For the current repository state, use this document as a practical reference rather than a final language standard.

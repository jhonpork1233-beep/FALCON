# Falcon Language Reference

> **Comprehensive reference for the Falcon programming language.**
> Derived from a line-by-line audit of every compiler source file.
> Version: 0.2.0

---

## 1. Overview

Falcon is a compiled systems programming language with:
- **Rust-inspired syntax** (func, let, match, struct, enum, trait, impl)
- **Three compilation profiles**: userland, kernel, baremetal
- **Two backends**: LLVM (primary, JIT + AOT) and C (legacy)
- **C runtime** per profile (full runtime, kernel stub, baremetal stub)
- **.fpy transpiler** for Python-style syntax (userland only)

### Compilation Pipeline

```
Source (.fc/.fpy) → Lexer → Parser → AST
  → filter_ast_by_profile (compile-time code erasure)
  → resolve_imports (merge imported files)
  → validate_imports (check capabilities)
  → validate_trait_impls (trait conformance)
  → monomorphize_generics (stamp out generic instantiations)
  → ast_to_ir (lower to IR with const inlining)
  → lint_missing_library_imports
  → validate_ir_import_contract
  → apply_profile_passes (profile-specific transforms)
  → verify_ownership (ownership/borrow checks)
  → LLVM codegen (or C codegen)
  → Link (clang for userland, ld.lld for freestanding)
```

---

## 2. CLI Commands

| Command | Description |
|---------|-------------|
| `falcon <file>` | Shortcut for `falcon run <file>` |
| `falcon run <file>` | Build + run (userland only, native mode on Windows) |
| `falcon build <file>` | Compile to binary (all profiles) |
| `falcon check <file>` | Parse + validate without building |
| `falcon fmt <files>` | Format source code |

### Build Flags

| Flag | Description |
|------|-------------|
| `--profile=<P>` | `userland` (default), `kernel`, `baremetal` |
| `--opt=<N>` | Optimization level 0-3 |
| `--emit-ir` | Emit IR as JSON |
| `--emit-c` | Emit C code (legacy) |
| `--emit-llvm` | Emit LLVM IR |
| `--run` | Run after building (userland only) |
| `--target=<triple>` | Cross-compile target (e.g., `x86_64-unknown-none-elf`) |
| `--profiles=all` | Multi-profile build |
| `--strict-imports` | Enforce explicit library imports |
| `--dump-imports` | Print resolved import map |

---

## 3. Types

### Primitive Types

| Type | Size | Description |
|------|------|-------------|
| `i8`, `i16`, `i32`, `i64`, `i128` | 1-16 bytes | Signed integers |
| `u8`, `u16`, `u32`, `u64`, `u128` | 1-16 bytes | Unsigned integers |
| `isize`, `usize` | pointer-width | Platform-sized integers |
| `f32`, `f64` | 4, 8 bytes | Floating point |
| `bool` | 1 byte | `true` or `false` |
| `String` | owned | Owned string (`FalconString` in runtime) |
| `str` | borrowed | String slice (C `const char*` internally) |
| `()` | 0 bytes | Unit type (void) |
| `!` | - | Never type (function never returns) |

> **Important**: In the current compiler, untyped integer literals default to `i64`. All arithmetic is 64-bit unless an explicit smaller type is annotated. The LLVM backend does support sub-64-bit types via `TypeHint` IR instructions.

### Compound Types

```falcon
// Arrays (fixed-size, stack-allocated)
let data = [1, 2, 3, 4, 5];
let x = data[0];        // index access

// Tuples
let pair = (10, 20);

// Structs
struct Point {
    x: i64,
    y: i64,
}
let p = Point { x: 10, y: 20 };
let px = p.x;           // field access

// Enums (tagged unions)
enum Color {
    Red,
    Green,
    Blue,
    Custom(i64),         // variant with payload
}
let c = Color::Red;
let c2 = Color::Custom(255);
```

### Pointer Types

```falcon
*mut T     // mutable raw pointer
*const T   // immutable raw pointer
&T         // immutable reference
&mut T     // mutable reference
```

### Type Casting

```falcon
let x = 0x24 as *mut u8;     // integer to pointer
let y = 42 as u8;            // integer to smaller type
```

---

## 4. Variables and Constants

### Local Variables

```falcon
let x = 42;                  // immutable, type inferred as i64
let mut counter = 0;         // mutable
let y: f64 = 3.14;           // explicit type
```

### Top-Level Constants

Top-level `const` and `let` declarations are **compile-time evaluated** and inlined at all use sites.

```falcon
const MAX_SIZE: i64 = 1024;
let LED_PIN: u8 = 5;
let LED_MASK: u8 = 1 << LED_PIN;        // evaluated at compile time
let DDRB: *mut u8 = 0x24 as *mut u8;    // cast is evaluated at compile time
```

The const evaluator supports: integer literals, float literals, bool literals, string literals, all binary operators (`+`, `-`, `*`, `/`, `%`, `<<`, `>>`, `&`, `|`, `^`), unary operators (`-`, `!`, `~`), `as` type casts, and references to previously-defined constants.

---

## 5. Functions

### Regular Functions

```falcon
func add(a: i64, b: i64) -> i64 {
    return a + b;
}

func greet(name: str) {
    println("Hello " + name);
}
```

### Generic Functions

```falcon
func identity<T>(x: T) -> T {
    return x;
}

// Call with explicit type argument:
let v = identity::<i64>(42);
```

Generics are monomorphized before IR lowering (stamped out into concrete functions like `identity_i64`).

### Extern Functions

Declare external C functions without a body:

```falcon
extern func outb(port: i64, val: i64);
extern func inb(port: i64) -> i64;
extern func read_volatile(addr: i64) -> i64;
extern func write_volatile(addr: i64, val: i64);
extern func compiler_fence();
```

### Unsafe Functions

```falcon
func unsafe raw_access() {
    // Allows pointer dereference, FFI, etc.
}
```

### Never-Returning Functions

```falcon
func _start() -> ! {
    loop {
        // runs forever
    }
}
```

---

## 6. Operators

### Arithmetic
`+`, `-`, `*`, `/`, `%`

### Comparison
`==`, `!=`, `<`, `<=`, `>`, `>=`

### Logical
`&&`, `||`, `!`

### Bitwise
`&` (AND), `|` (OR), `^` (XOR), `~` (NOT), `<<` (shift left), `>>` (shift right)

### Compound Assignment
`+=`, `-=`, `*=`, `/=`, `%=`

### Other
- `as` — type casting: `expr as Type`
- `?` — error propagation (try operator)
- `..` — range: `0..10`

---

## 7. Control Flow

### If / Else

```falcon
if condition {
    // ...
} else if other {
    // ...
} else {
    // ...
}

// If as expression:
let result = if x > 0 { x } else { -x };
```

### While Loop

```falcon
while counter < 10 {
    counter += 1;
}
```

### For Loop

```falcon
for i in 0..10 {
    println(i);
}
```

### Loop (Infinite)

```falcon
loop {
    if done { break; }
    continue;
}
```

### Match

```falcon
match value {
    0 => println("zero"),
    1 => println("one"),
    _ => println("other"),     // wildcard
}

// Match on enum variants:
match color {
    Color::Red => println("red"),
    Color::Custom(n) => println(n),   // binding
    _ => println("other"),
}

// Struct pattern matching:
match point {
    Point { x: 0, y } => println("on y-axis"),
    _ => println("elsewhere"),
}
```

---

## 8. Structs, Enums, Traits

### Structs

```falcon
struct Point {
    x: i64,
    y: i64,
}
```

### Impl Blocks (Methods)

```falcon
impl Point {
    // Associated function (no self)
    func new(x: i64, y: i64) -> Point {
        return Point { x: x, y: y };
    }

    // Instance method
    func distance(self) -> f64 {
        return sqrt(self.x * self.x + self.y * self.y);
    }

    // Mutable method
    func translate(&mut self, dx: i64, dy: i64) {
        self.x += dx;
        self.y += dy;
    }
}

// Usage:
let p = Point::new(3, 4);
let d = p.distance();
```

Self parameter types: `self` (ownership), `&self` (immutable borrow), `&mut self` (mutable borrow).

### Enums

```falcon
enum Shape {
    Circle(f64),          // variant with payload
    Rectangle(f64),       // variant with payload
    Unknown,              // unit variant
}
```

Enum tags are auto-assigned (0, 1, 2, ...).

### Traits

```falcon
trait Printable {
    func display(self);
}

impl Printable for Point {
    func display(self) {
        print(self.x);
        print(self.y);
    }
}
```

Trait conformance is checked at compile time: all declared methods must be implemented with matching signatures.

---

## 9. Imports and Modules

### Import Syntax

```falcon
import math;
import io;
use std::{println, print};       // selective import
import crypto;                    // profile-routed
```

### Module System

```falcon
mod utils {
    pub func helper() -> i64 {
        return 42;
    }
}
```

### Profile-Routed Modules

These modules are profile-aware: `crypto`, `net`, `ai`, `fs`, `sync`, `time`, `math`, `io`, `random`, `string`.

- **Userland**: resolves to `module/mod`
- **Kernel**: must use `module::kernel` or `module::raw`
- **Baremetal**: must use `module::raw`

### Import Resolution

Imports are resolved from `.fc` files relative to the source directory. The resolved file's AST is merged into the main program.

---

## 10. Closures

```falcon
let add = |a: i64, b: i64| -> i64 { a + b };
let result = add(3, 4);

// Closures capture variables:
let factor = 10;
let multiply = |x: i64| -> i64 { x * factor };
```

In IR, closures become `ClosureCreate` + `ClosureCall` instructions. Captured variables are tracked.

---

## 11. Error Handling

### Option and Result Types

```falcon
enum Option {
    Some(T),
    None,
}

enum Result {
    Ok(T),
    Err(E),
}
```

### Try Operator (`?`)

```falcon
func read_file(path: str) -> Result {
    let data = open(path)?;   // returns Err early if failed
    return Result::Ok(data);
}
```

In IR, `?` lowers to: check tag → if Err, return Err → if Ok, extract payload.

---

## 12. Compilation Profiles

### Userland (Default)

- Full runtime: panic, alloc, I/O, math, random, strings, Vec, file I/O, Ollama
- Bounds checking enabled
- Heap allocations allowed
- Panic unwinding allowed
- All capabilities: Heap, Runtime, Panic, Unsafe, Threads, Os

### Kernel

- Freestanding runtime: `falcon_abs`, `falcon_min`, `falcon_max` only
- **No** libc, **no** heap, **no** panic, **no** hosted I/O
- Only `Unsafe` capability allowed
- Entry point: `_start` (via linker script)
- Target: `x86_64-unknown-none-elf`
- Linker: `ld.lld` with `falcon_kernel.ld` (base address: 1MB)
- Output: ELF + flat binary (.bin)
- Use `extern func` for hardware I/O (`outb`, `inb`, etc.)

### Baremetal

- Freestanding runtime: stub I/O (no-op) + `falcon_abs/min/max`
- **No** libc, **no** heap, **no** panic
- Only `Unsafe` capability allowed
- Entry point: `_start` (via linker script)
- Target: `x86_64-unknown-none-elf` (or AVR, RISC-V via `--target`)
- Linker: `ld.lld` with `falcon_baremetal.ld` (base address: 0x8000)
- Use `extern func` for hardware boundary functions

### Profile Attributes

```falcon
#[userland]
func gui_render() { /* only compiled in userland */ }

#[kernel]
func interrupt_handler() { /* only compiled in kernel */ }

#[baremetal]
func mcu_init() { /* only compiled in baremetal */ }

// No attribute = shared (compiled in ALL profiles)
func compute(a: i64, b: i64) -> i64 { return a + b; }
```

Functions with a non-matching profile attribute are **erased at compile time**.

---

## 13. Literals

| Literal | Example | Token |
|---------|---------|-------|
| Decimal integer | `42`, `1_000_000` | `IntLiteral(i64)` |
| Hex integer | `0xFF`, `0x1A2B` | `IntLiteral(i64)` |
| Binary integer | `0b1010`, `0b1111_0000` | `IntLiteral(i64)` |
| Float | `3.14`, `0.5` | `FloatLiteral(f64)` |
| String | `"hello\nworld"` | `StringLiteral` |
| Char | `'a'`, `'\n'` | `CharLiteral` |
| Bool | `true`, `false` | `BoolLiteral` |

### Escape Sequences (strings and chars)
`\n` `\t` `\r` `\\` `\"` `\'` `\0`

### Number Separators
Underscores in numbers are ignored: `1_000_000`, `0xFF_FF`, `0b1111_0000`

---

## 14. Comments

```falcon
// Line comment

/* Block comment */

/* Nested /* block */ comments /* are */ supported */
```

Block comments support unlimited nesting depth.

---

## 15. Keywords (Complete List)

```
func    let     mut     if      else    while   for     loop
break   continue return  match   struct  enum    impl    mod
import  pub     const   as      unsafe  extern  use     in
trait   true    false
```

Total: **25 keywords** + `true`/`false` (parsed as `BoolLiteral`, not keywords in practice).

---

## 16. Runtime Library Functions

### Available in ALL profiles:
- `falcon_abs(value: i64) -> i64`
- `falcon_min(a: i64, b: i64) -> i64`
- `falcon_max(a: i64, b: i64) -> i64`

### Userland only:

**I/O:**
- `falcon_println(s: str)` / `falcon_print(s: str)`
- `falcon_print_int(v: i64)` / `falcon_print_i32(v: i32)` / `falcon_print_float(v: f64)` / `falcon_print_bool(v: bool)`
- `falcon_print_newline()`
- `falcon_input(prompt: str) -> str`

**Math (libm wrappers):**
- `falcon_sin`, `falcon_cos`, `falcon_tan`, `falcon_asin`, `falcon_acos`, `falcon_atan`
- `falcon_sqrt`, `falcon_pow`, `falcon_exp`, `falcon_log`, `falcon_log10`
- `falcon_floor`, `falcon_ceil`, `falcon_round`
- `falcon_pi()`, `falcon_e()`

**Random:**
- `falcon_random_seed(seed: i64)`
- `falcon_random() -> f64`
- `falcon_randint(a: i64, b: i64) -> i64`
- `falcon_randrange(n: i64) -> i64`
- `falcon_time_seed() -> i64`

**Strings:**
- `falcon_str_concat(a: str, b: str) -> str`
- `falcon_str_eq(a: str, b: str) -> bool`
- `falcon_str_is_empty(s: str) -> bool`
- `falcon_str_len(s: str) -> i64`
- `falcon_str_find_from(haystack: str, needle: str, start: i64) -> i64`
- `falcon_str_substr(s: str, start: i64, length: i64) -> str`
- `falcon_str_replace_all(input: str, target: str, replacement: str) -> str`
- `falcon_str_strip_html_tags(input: str) -> str`
- `falcon_str_json_extract_values(json: str, key: str, max_items: i64) -> str`

**Memory:**
- `falcon_alloc(size: i64) -> ptr`
- `falcon_dealloc(ptr: ptr)`
- `falcon_realloc(ptr: ptr, new_size: i64) -> ptr`
- `falcon_bounds_check(index: i64, length: i64)`

**File I/O:**
- `falcon_file_read(filename: str, buffer: ptr, max_size: i64) -> i64`
- `falcon_file_write(filename: str, data: str, size: i64) -> i64`
- `falcon_file_append(filename: str, data: str, size: i64) -> i64`
- `falcon_file_exists(filename: str) -> i64`
- `falcon_file_size(filename: str) -> i64`

**OS Exec:**
- `falcon_os_exec_capture(command: str) -> str`
- `falcon_os_exec_stream(command: str) -> i64`

**Ollama LLM:**
- `falcon_ollama_generate(model: str, prompt: str)`
- `falcon_ollama_list_models()`
- `falcon_ollama_chat(model: str, personality: str)`

---

## 17. IR Instruction Set (Complete)

The IR has ~50 instructions in these categories:

**Memory:** Alloc, Load, Store, VolatileLoad, VolatileStore, Free, StackAlloc, HeapAlloc

**Ownership:** Move, BorrowImm, BorrowMut, Drop, Copy

**Control Flow:** Branch, BranchCond, Call, Return, Label

**Arithmetic:** Add, Sub, Mul, Div, Mod

**Comparison:** Eq, Ne, Lt, Le, Gt, Ge

**Logical:** And, Or, Not

**Bitwise:** BitAnd, BitOr, BitXor, Shl, Shr, BitNot

**Unary:** Neg, AddrOf, PtrDeref

**Data:** Literal, TypeHint, Index, FieldAccess, StructInit, ArrayInit, Range

**Enum:** EnumInit, EnumTag, EnumPayload

**Closure:** ClosureCreate, ClosureCall

**Safety:** BoundsCheck, Panic, Unwrap

---

## 18. Linker Scripts

### Kernel (`falcon_kernel.ld`)
- Format: `elf64-x86-64`
- Entry: `_start`
- Base address: `1MB` (0x100000)
- Sections: `.text`, `.rodata`, `.data`, `.bss` (4K aligned)

### Baremetal (`falcon_baremetal.ld`)
- Format: `elf64-x86-64`
- Entry: `_start`
- Base address: `0x8000`
- Sections: `.text`, `.rodata`, `.data`, `.bss` (4K aligned)

---

## 19. Examples Quick Reference

### Hello World
```falcon
func main() -> i64 {
    println("Hello, World!");
    return 0;
}
```

### Kernel (Serial Output)
```falcon
extern func outb(port: i64, val: i64);
extern func inb(port: i64) -> i64;

func serial_write(byte: i64) {
    loop {
        let status = inb(0x3FD);
        if (status & 0x20) != 0 { break; }
    }
    outb(0x3F8, byte);
}

func _start() {
    serial_write(72); // 'H'
    loop {}
}
```

### Baremetal (Arduino Blink)
```falcon
extern func read_volatile(addr: *const u8) -> u8;
extern func write_volatile(addr: *mut u8, val: u8);
extern func compiler_fence();

let DDRB: *mut u8 = 0x24 as *mut u8;
let PORTB: *mut u8 = 0x25 as *mut u8;
let LED_MASK: u8 = 1 << 5;

func _start() -> ! {
    // set pin 13 as output
    let cur = read_volatile(DDRB);
    write_volatile(DDRB, cur | LED_MASK);

    loop {
        let c = read_volatile(PORTB);
        write_volatile(PORTB, c | LED_MASK);   // ON
        delay_ms(500);
        let c = read_volatile(PORTB);
        write_volatile(PORTB, c & ~LED_MASK);  // OFF
        delay_ms(500);
    }
}
```

---

## 20. File Extensions

| Extension | Description |
|-----------|-------------|
| `.fc` | Falcon source code |
| `.fpy` | Python-style Falcon (transpiled to `.fc`, userland only) |
| `.elf` | Linked ELF binary (kernel/baremetal) |
| `.bin` | Flat binary (objcopy from .elf) |
| `.o` | Object file (intermediate) |
| `.c` | Generated C code (legacy backend) |

---

## 21. Environment Variables

| Variable | Description |
|----------|-------------|
| `FALCON_LINKER` | Override hosted linker (default: clang) |
| `FALCON_FREESTANDING_LINKER` | Override freestanding linker (default: ld.lld) |
| `FALCON_OBJCOPY` | Override objcopy (default: llvm-objcopy) |
| `FALCON_RUNTIME_DIR` | Override runtime directory |

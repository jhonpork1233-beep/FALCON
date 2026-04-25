# Falcon Runtime Boundary Specification

**Version:** 1.0  
**Status:** Frozen (Phase-1 Requirement)  
**Last Updated:** 2024-12-29

This document defines what "runtime" means in Falcon and what runtime components exist in each compilation profile.

---

## Definition of "Runtime"

**Runtime:** Code that is linked into every Falcon program and executes before, during, or after user code. This includes initialization, cleanup, and helper functions.

**Key Principle:** Runtime is **explicit**, not hidden. All runtime components are documented and profile-specific.

---

## Userland Profile Runtime

### Minimal Runtime Components

**1. Panic Handler**
- **Purpose:** Handle unrecoverable errors
- **Behavior:** Unwind stack, run destructors, print message, abort
- **Size:** ~5-10 KB
- **Required:** Yes (can be customized)

**2. Memory Allocator**
- **Purpose:** Heap allocation for `Vec`, `String`, `Box`, etc.
- **Behavior:** Standard allocator (malloc/free wrapper)
- **Size:** ~10-20 KB
- **Required:** Yes (can be replaced with `#[global_allocator]`)

**3. Stack Overflow Detection (Optional)**
- **Purpose:** Detect stack overflow
- **Behavior:** Guard pages or stack canary
- **Size:** ~1-2 KB
- **Required:** No (enabled via flag)

**4. Memory Sanitizers (Optional)**
- **Purpose:** Detect memory errors
- **Behavior:** AddressSanitizer, MemorySanitizer
- **Size:** Varies (10-50% overhead)
- **Required:** No (enabled via `--sanitize` flag)

**5. Unwinding Support**
- **Purpose:** Stack unwinding for panics
- **Behavior:** libunwind or equivalent
- **Size:** ~5-10 KB
- **Required:** Yes (for panic unwinding)

**6. I/O Hooks**
- **Purpose:** Standard input/output
- **Behavior:** Wrappers around libc I/O
- **Size:** ~2-5 KB
- **Required:** Yes (for `println`, file I/O)

### Total Runtime Size

**Minimal (no sanitizers):** ~25-50 KB  
**With sanitizers:** +10-50% overhead

### Runtime Initialization

**Order:**
1. Platform-specific initialization
2. Stack setup
3. Global constructors (if any)
4. Allocator initialization
5. I/O initialization
6. User `main()` function

**Runtime Cleanup:**
1. User code completes
2. Global destructors (if any)
3. I/O cleanup
4. Allocator cleanup
5. Platform-specific cleanup

### Forbidden in Userland Runtime

- ❌ Garbage collector
- ❌ JIT compiler
- ❌ Reflection system
- ❌ Exception handling (beyond panics)
- ❌ Hidden allocations
- ❌ Background threads

---

## Kernel Profile Runtime

### No Runtime

**Definition:** Zero runtime components. All behavior is explicit in user code.

**What This Means:**
- No panic handler (abort only)
- No memory allocator (unless custom)
- No stack overflow detection
- No unwinding support
- No I/O hooks
- No hidden helpers

### Explicit Behavior Only

**Panic Handling:**
- Panics are compile errors
- Must use `Result<T, E>` for errors
- Abort on unrecoverable errors (no unwinding)

**Memory Management:**
- Stack allocations only
- Custom allocators via `#[global_allocator]`
- Explicit memory management

**Error Handling:**
- All errors explicit (`Result`)
- No hidden error paths
- Abort on fatal errors

### Initialization

**Order:**
1. Platform-specific setup (minimal)
2. Stack setup
3. User code (no runtime init)

**Cleanup:**
1. User code completes
2. Abort (no cleanup needed)

### Forbidden in Kernel Profile

- ❌ Any runtime components
- ❌ Hidden helpers
- ❌ Implicit initialization
- ❌ Automatic cleanup
- ❌ Runtime symbols

---

## Baremetal Profile Runtime

### Zero Runtime

**Definition:** Absolutely zero runtime. Program starts at `_start` or `main`, no helpers.

**What This Means:**
- No panic handler
- No memory allocator
- No stack setup (programmer responsible)
- No unwinding
- No I/O
- No libc
- No standard library
- No hidden code

### Explicit Everything

**Entry Point:**
- `#[entry] func _start() -> !` or `func main() -> !`
- Programmer defines entry point
- No automatic initialization

**Stack:**
- Programmer responsible for stack setup
- No automatic stack initialization
- Stack pointer must be set manually (if needed)

**Memory:**
- No allocator
- Manual memory management only
- Direct hardware access

**Errors:**
- No error handling (abort or loop)
- Custom panic handler required (if any)
- No unwinding

### Initialization

**Order:**
1. Hardware-specific setup (programmer code)
2. Stack setup (programmer code)
3. User code

**Cleanup:**
1. User code (infinite loop or halt)
2. No cleanup (system halts)

### Forbidden in Baremetal Profile

- ❌ All runtime components
- ❌ libc
- ❌ Standard library
- ❌ Hidden code
- ❌ Automatic initialization
- ❌ Any helpers

---

## Runtime Symbols

### Userland Profile

**Required Symbols:**
- `falcon_panic` - Panic handler
- `falcon_alloc` - Memory allocator
- `falcon_dealloc` - Memory deallocator
- `falcon_realloc` - Memory reallocator
- `falcon_print` - Print function
- `falcon_unwind` - Unwinding support

**Optional Symbols:**
- `falcon_stack_guard` - Stack overflow detection
- `falcon_sanitizer_*` - Sanitizer hooks

### Kernel Profile

**Required Symbols:**
- None (zero runtime)

**Forbidden Symbols:**
- Any runtime symbols
- Any hidden helpers

### Baremetal Profile

**Required Symbols:**
- None (zero runtime)

**Forbidden Symbols:**
- All runtime symbols
- All standard library symbols
- All libc symbols

---

## Runtime Boundary Enforcement

### Compile-Time Checks

**IR Validation:**
- Verify no runtime symbols in kernel/baremetal
- Verify no implicit runtime calls
- Verify explicit behavior only

**Linker Checks:**
- Kernel/baremetal: Fail if runtime symbols referenced
- Userland: Allow runtime symbols

### Runtime Size Verification

**Userland:**
- Report runtime size in build output
- Allow customization of runtime components

**Kernel/Baremetal:**
- Verify zero runtime size
- Fail if any runtime code detected

---

## Summary

| Component | Userland | Kernel | Baremetal |
|-----------|----------|--------|-----------|
| **Panic Handler** | ✅ | ❌ | ❌ |
| **Allocator** | ✅ | ❌ | ❌ |
| **Unwinding** | ✅ | ❌ | ❌ |
| **I/O Hooks** | ✅ | ❌ | ❌ |
| **Stack Guard** | ⚠️ Optional | ❌ | ❌ |
| **Sanitizers** | ⚠️ Optional | ❌ | ❌ |
| **Runtime Size** | ~25-50 KB | 0 KB | 0 KB |
| **Hidden Code** | Minimal | None | None |

**"Zero runtime" means zero. No exceptions.**

**This specification is frozen. All implementations must conform.**










# Unsafe Guarantees by Profile

This document defines what guarantees (or lack thereof) exist when using `unsafe` blocks in each compilation profile.

## The `unsafe` Keyword

The `unsafe` keyword in Falcon marks code that bypasses the compiler's safety checks. It is **explicit** and **intentional**.

```falcon
unsafe {
    // Code that bypasses safety checks
    let ptr = 0x1000 as *mut u8
    *ptr = 42
}
```

## Profile-Specific Behavior

### userland Profile

**What `unsafe` allows:**
- Raw pointer dereferencing
- Calling unsafe functions
- FFI (Foreign Function Interface)
- Inline assembly
- Mutating static variables

**What safety checks remain:**
- ✅ **Bounds checking** in unsafe blocks (optional, can disable)
- ✅ **Memory sanitizers** available (via `--sanitize` flag)
- ✅ **Stack overflow detection** (always active)
- ✅ **Use-after-free detection** (optional, via sanitizers)
- ✅ **Double-free detection** (optional, via sanitizers)

**Example:**
```falcon
unsafe {
    let ptr = 0x1000 as *mut u8
    *ptr = 42  // Bounds-checked if sanitizers enabled
    // Stack overflow detection still active
}
```

**Compilation flags:**
```bash
falcon build app.fc                    # Default: some checks
falcon build --sanitize=memory app.fc  # Full sanitization
falcon build --sanitize=none app.fc     # Minimal checks
```

**Use cases:**
- FFI with C libraries
- Performance-critical code with manual optimization
- Low-level system calls
- When you need raw pointer access but want some safety

### kernel Profile

**What `unsafe` allows:**
- Raw pointer dereferencing
- Calling unsafe functions
- FFI (with explicit lifetimes)
- Inline assembly
- Mutating static variables
- Direct memory manipulation

**What safety checks remain:**
- ❌ **No bounds checking** (removed for performance)
- ❌ **No memory sanitizers** (not available)
- ❌ **No stack overflow detection** (removed)
- ✅ **Strict aliasing rules** (enforced)
- ✅ **Explicit lifetimes** required at FFI boundaries
- ✅ **No implicit allocations** (compile error)

**Example:**
```falcon
unsafe {
    let ptr = 0x1000 as *mut u8
    *ptr = 42  // No bounds check, no sanitization
    // Programmer responsible for correctness
}

// FFI requires explicit lifetimes
extern "C" {
    func c_function<'a>(data: &'a [u8]) -> *const u8;
}
```

**Restrictions:**
- Cannot use heap allocation functions (unless custom allocator)
- Must handle all errors explicitly (no panics)
- Lifetime annotations required at boundaries

**Use cases:**
- Device drivers
- Kernel modules
- Real-time systems
- Memory allocators
- System-level code

### baremetal Profile

**What `unsafe` allows:**
- **Everything** - no restrictions
- Raw pointer arithmetic
- Undefined behavior (programmer's responsibility)
- Direct hardware access
- Zero runtime overhead

**What safety checks remain:**
- ❌ **NONE** - Programmer is trusted completely
- ❌ **No bounds checking**
- ❌ **No memory sanitizers**
- ❌ **No stack overflow detection**
- ❌ **No use-after-free detection**
- ❌ **No double-free detection**
- ❌ **No undefined behavior detection**

**Example:**
```falcon
unsafe {
    let ptr = 0x4000_0000 as *mut u32  // GPIO register
    *ptr = 0x01  // Direct hardware write, zero checks
    // If you write wrong address, undefined behavior
}
```

**Guarantees:**
- Zero runtime overhead
- Direct hardware access
- Full control over memory layout
- No safety nets whatsoever

**Use cases:**
- Bootloaders
- Firmware
- Microcontrollers
- RTOS (Real-Time Operating Systems)
- Embedded systems
- When every cycle counts

## Unsafe Function Guidelines

### When to mark functions as `unsafe`

```falcon
// ✅ CORRECT: Function that requires unsafe guarantees
unsafe func raw_memory_copy(
    src: *const u8,
    dst: *mut u8,
    len: usize
) {
    for i in range(len) {
        *dst.offset(i) = *src.offset(i)
    }
}

// ❌ WRONG: Safe function marked unsafe
unsafe func add(a: i32, b: i32) -> i32 {
    a + b  // This is safe, no need for unsafe
}
```

### Calling unsafe functions

```falcon
// Must be in unsafe block
unsafe {
    raw_memory_copy(src_ptr, dst_ptr, 100)
}

// OR mark calling function as unsafe
unsafe func wrapper() {
    raw_memory_copy(src_ptr, dst_ptr, 100)  // OK, already in unsafe context
}
```

## Safety Invariants

Even in `unsafe` blocks, you must maintain these invariants:

1. **No use-after-free**: Don't use memory after it's freed
2. **No double-free**: Don't free memory twice
3. **No buffer overflows**: Don't write past allocated bounds
4. **Valid pointers**: Pointers must point to valid memory
5. **No data races**: In concurrent code, use proper synchronization

**Violating these invariants is undefined behavior**, even in `unsafe` blocks.

## Best Practices

### 1. Minimize unsafe code

```falcon
// ❌ BAD: Entire function unsafe
unsafe func process(data: Vec<u8>) {
    // Most of this is safe
    unsafe {
        // Only this part needs unsafe
    }
}

// ✅ GOOD: Only unsafe block where needed
func process(data: Vec<u8>) {
    // Safe code here
    unsafe {
        // Only unsafe part
    }
    // More safe code
}
```

### 2. Document unsafe invariants

```falcon
unsafe func raw_write(addr: usize, value: u8) {
    // SAFETY: Caller must ensure:
    // - addr is valid memory address
    // - addr is writable
    // - addr is properly aligned
    let ptr = addr as *mut u8
    *ptr = value
}
```

### 3. Provide safe wrappers

```falcon
// Unsafe implementation
unsafe func unsafe_operation(ptr: *mut u8) { }

// Safe wrapper
func safe_operation(data: &mut [u8]) -> Result<(), Error> {
    if data.is_empty() {
        return Err(Error::Empty)
    }
    unsafe {
        unsafe_operation(data.as_mut_ptr())
    }
    Ok(())
}
```

## Summary Table

| Feature | userland | kernel | baremetal |
|---------|----------|--------|-----------|
| Bounds checking | ✅ (optional) | ❌ | ❌ |
| Memory sanitizers | ✅ (optional) | ❌ | ❌ |
| Stack overflow | ✅ | ❌ | ❌ |
| Use-after-free | ✅ (optional) | ❌ | ❌ |
| Strict aliasing | ✅ | ✅ | ❌ |
| Explicit lifetimes | ✅ (at boundaries) | ✅ (required) | ❌ |
| Zero overhead | ❌ | ⚠️ (minimal) | ✅ |
| Hardware access | ⚠️ (limited) | ✅ | ✅ |
| Undefined behavior | ⚠️ (detected) | ⚠️ (possible) | ✅ (allowed) |

---

**Last Updated**: 2024-12-29  
**Version**: 1.0


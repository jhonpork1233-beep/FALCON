# Falcon Error Model Specification

**Version:** 1.0  
**Status:** FROZEN (Phase-1 Requirement)  
**Last Updated:** 2024-12-29

This document defines the complete error handling model for Falcon. This specification is **frozen** and cannot be changed without a major version bump.

---

## Error Philosophy

**Single Philosophy:** Errors are values, not exceptions.

**Core Principles:**
1. **Errors → Return Values**: Use `Result<T, E>` for recoverable errors
2. **Panics → Abort**: Use `panic!` for unrecoverable errors (userland only)
3. **No Exceptions**: No exception-based error handling
4. **No Hidden Unwinding**: All error paths are explicit

**This philosophy is immutable. It cannot change.**

---

## Error Types

### Recoverable Errors: `Result<T, E>`

**Definition:** Errors that can be handled and recovered from.

**Type:**
```falcon
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

**Usage:**
```falcon
func divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        return Err("Division by zero");
    }
    Ok(a / b)
}

// Handle error
match divide(10, 2) {
    Ok(result) => println("Result: " + result),
    Err(msg) => println("Error: " + msg),
}
```

**Error Propagation:**
```falcon
func complex_operation() -> Result<i32, String> {
    let a = divide(10, 2)?;  // Returns early if Err
    let b = divide(20, 4)?;
    Ok(a + b)
}
```

**Rules:**
- All recoverable errors use `Result`
- `?` operator propagates errors
- Errors are explicit in function signatures
- No hidden error paths

### Unrecoverable Errors: `panic!`

**Definition:** Errors that cannot be recovered from. Program execution cannot continue.

**Usage:**
```falcon
if data.is_empty() {
    panic!("Data cannot be empty!");
}
```

**Behavior by Profile:**

**Userland Profile:**
- ✅ `panic!` allowed
- ✅ Stack unwinding (runs destructors)
- ✅ Error message printed
- ✅ Program aborts after unwinding

**Kernel Profile:**
- ❌ `panic!` **forbidden** (compile error)
- ✅ Must use `Result` for all errors
- ✅ Abort on fatal errors (no unwinding)

**Baremetal Profile:**
- ❌ `panic!` **forbidden** (compile error)
- ✅ Custom panic handler required (if any)
- ✅ Abort or infinite loop on fatal errors

### Optional Values: `Option<T>`

**Definition:** Values that may or may not exist.

**Type:**
```falcon
enum Option<T> {
    Some(T),
    None,
}
```

**Usage:**
```falcon
func find_item(items: Vec<String>, name: &str) -> Option<String> {
    for item in items {
        if item == name {
            return Some(item);
        }
    }
    None
}
```

**Not an Error:** `Option` is for optional values, not errors. Use `Result` for errors.

---

## Error Handling by Profile

### Userland Profile

**Allowed:**
- ✅ `Result<T, E>` for recoverable errors
- ✅ `panic!` for unrecoverable errors
- ✅ `Option<T>` for optional values
- ✅ `?` operator for error propagation
- ✅ `unwrap()` and `expect()` (panic on error)

**Panic Behavior:**
1. Stack unwinding begins
2. Destructors run (RAII cleanup)
3. Error message printed
4. Program aborts

**Example:**
```falcon
func process(data: Vec<u8>) {
    if data.is_empty() {
        panic!("Empty data");  // OK in userland
    }
    // Process data
}
```

### Kernel Profile

**Allowed:**
- ✅ `Result<T, E>` for all errors
- ✅ `Option<T>` for optional values
- ✅ `?` operator for error propagation
- ❌ `panic!` **forbidden** (compile error)
- ❌ `unwrap()` **forbidden** (compile error, uses panic)
- ❌ `expect()` **forbidden** (compile error, uses panic)

**Error Handling:**
- All errors must be explicit
- Must handle all `Result` values
- Abort on fatal errors (no unwinding)

**Example:**
```falcon
func process(data: Vec<u8>) -> Result<(), Error> {
    if data.is_empty() {
        return Err(Error::Empty);  // Must use Result
    }
    // Process data
    Ok(())
}

// Caller must handle error
match process(data) {
    Ok(()) => { /* success */ },
    Err(e) => { /* handle error */ },
}
```

### Baremetal Profile

**Allowed:**
- ✅ `Result<T, E>` (compile-time only, no runtime)
- ✅ `Option<T>` (compile-time only, no runtime)
- ❌ `panic!` **forbidden** (compile error)
- ❌ `unwrap()` **forbidden** (compile error)
- ❌ `expect()` **forbidden** (compile error)

**Error Handling:**
- All errors explicit
- Custom panic handler required (if any)
- Abort or infinite loop on fatal errors
- No unwinding support

**Example:**
```falcon
#[panic_handler]
func panic(_info: &PanicInfo) -> ! {
    // Custom panic handler
    loop {}  // Halt forever
}

func process(data: &[u8]) -> Result<(), Error> {
    if data.is_empty() {
        return Err(Error::Empty);
    }
    Ok(())
}
```

---

## Error Propagation

### The `?` Operator

**Definition:** Propagates errors automatically.

**Syntax:**
```falcon
func operation() -> Result<i32, String> {
    let a = may_fail()?;  // Returns early if Err
    let b = may_fail()?;
    Ok(a + b)
}
```

**Behavior:**
- If `Result` is `Ok`, unwrap and continue
- If `Result` is `Err`, return early with error

**Rules:**
- Only works in functions returning `Result`
- Cannot use in functions returning other types
- Explicit error propagation (visible in code)

### Manual Error Handling

**Pattern Matching:**
```falcon
match operation() {
    Ok(value) => {
        // Handle success
    }
    Err(error) => {
        // Handle error
    }
}
```

**If-Let:**
```falcon
if let Ok(value) = operation() {
    // Use value
} else {
    // Handle error
}
```

---

## Compile-Time vs Runtime Errors

### Compile-Time Errors

**Definition:** Errors detected by the compiler before execution.

**Examples:**
- Type mismatches
- Ownership violations
- Undefined variables
- Profile violations (panic in kernel, allocation in kernel/baremetal)
- Missing error handling (in kernel profile)

**Enforcement:**
- Compiler rejects invalid programs
- No runtime execution of invalid code
- Clear error messages

### Runtime Errors

**Definition:** Errors that occur during program execution.

**Userland Profile:**
- Panics (unrecoverable)
- Division by zero (panic)
- Out of bounds (panic)
- Stack overflow (abort)

**Kernel Profile:**
- Division by zero (abort, no unwinding)
- Out of bounds (undefined behavior, no check)
- Stack overflow (abort, no unwinding)

**Baremetal Profile:**
- All errors are undefined behavior
- No runtime checks
- Programmer responsible for correctness

---

## Error Model Summary

| Aspect | Userland | Kernel | Baremetal |
|--------|----------|--------|-----------|
| **Result<T, E>** | ✅ | ✅ | ✅ |
| **panic!** | ✅ | ❌ Forbidden | ❌ Forbidden |
| **unwrap()** | ✅ | ❌ Forbidden | ❌ Forbidden |
| **Error Propagation** | ✅ `?` operator | ✅ `?` operator | ✅ `?` operator |
| **Unwinding** | ✅ | ❌ | ❌ |
| **Abort** | ✅ After unwinding | ✅ Immediate | ✅ Immediate |
| **Runtime Checks** | ✅ | ❌ | ❌ |

---

## Frozen Decisions

**These decisions cannot change:**

1. ✅ Errors are values (`Result`), not exceptions
2. ✅ Panics are for unrecoverable errors only
3. ✅ No exception-based error handling
4. ✅ Kernel/baremetal: No panics
5. ✅ All error paths are explicit
6. ✅ `?` operator for error propagation
7. ✅ No hidden unwinding

**Changing these requires Falcon 2.0.**

---

## Implementation Requirements

### IR Representation

**Errors must be explicit in IR:**
- `Result` types represented explicitly
- `?` operator generates explicit error checks
- Panic calls marked explicitly
- Profile validation rejects invalid panics

### Profile Enforcement

**Compile-time checks:**
- Kernel: Reject `panic!` calls
- Kernel: Reject `unwrap()` / `expect()`
- Baremetal: Reject `panic!` calls
- Baremetal: Reject `unwrap()` / `expect()`

**IR Validation:**
- Verify no panic calls in kernel/baremetal
- Verify all errors handled in kernel profile
- Verify explicit error paths

---

**This specification is FROZEN. All implementations must conform.**

**Violating this specification breaks the language.**










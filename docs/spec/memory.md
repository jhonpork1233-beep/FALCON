# Falcon Memory Model Specification

**Version:** 1.0  
**Status:** Frozen (Phase-1 Requirement)  
**Last Updated:** 2024-12-29

This document defines the complete memory model for Falcon. This is a **design specification**, not implementation notes. All implementations must conform to this specification.

---

## Memory Regions

### Stack

**Definition:** Function-local, lifetime-bound memory region.

**Characteristics:**
- Automatically allocated on function entry
- Automatically deallocated on function exit
- Non-escaping (cannot outlive function)
- Fixed size at compile time
- Zero runtime overhead (baremetal)

**What lives on stack:**
- Local variables
- Function parameters
- Temporary values
- Fixed-size arrays `[T; N]`

**Lifetime Rules:**
- Stack values live until end of scope
- References to stack values cannot outlive the value
- Moving stack values transfers ownership

**Profile Behavior:**
- **userland**: Stack with overflow detection (optional)
- **kernel**: Stack only, no overflow detection
- **baremetal**: Stack only, zero overhead

### Heap

**Definition:** Dynamically allocated, manually managed memory region.

**Characteristics:**
- Explicit allocation required
- Explicit deallocation (RAII)
- Can outlive allocating function
- Size determined at runtime
- Requires allocator (userland only)

**What lives on heap:**
- `Vec<T>` (userland only)
- `String` (userland only)
- `Box<T>` (userland only)
- Any explicitly allocated memory

**Allocation Rules:**
- **userland**: Heap allocation allowed via standard library
- **kernel**: Heap allocation **forbidden** (compile error)
- **baremetal**: Heap allocation **forbidden** (compile error)

**Deallocation:**
- Automatic via RAII (destructors)
- Explicit `drop()` when needed
- No garbage collection

### Globals

**Definition:** Program-wide, static memory region.

**Characteristics:**
- Lifetime: entire program execution
- Single instance
- Mutable only with `unsafe` (userland) or forbidden (kernel/baremetal)

**What lives in globals:**
- Constants (`const`)
- Static variables (`static`)
- Static mutables (`static mut` - requires `unsafe`)

**Profile Behavior:**
- **userland**: Allowed, mutable with `unsafe`
- **kernel**: Allowed, mutable with `unsafe`, no runtime init
- **baremetal**: Allowed, mutable with `unsafe`, no runtime init

---

## Semantics

### Move vs Copy

**Copy Semantics:**
- Types that implement `Copy` trait
- Integers, floats, bool, char
- Small fixed-size types
- Copy is implicit (no move)

**Move Semantics:**
- Default for all types
- Transfers ownership
- Original value becomes invalid
- No implicit copying

**Move Rules:**
1. Assignment moves: `let y = x;` (x moved to y)
2. Function call moves: `func(x)` (x moved to function)
3. Return moves: `return x;` (x moved to caller)
4. After move, original is invalid (compile error to use)

**Copy Rules:**
1. Only `Copy` types can be copied
2. Copy is implicit (no `clone()` needed)
3. Both original and copy are valid

### Mutability Guarantees

**Immutable by Default:**
- All bindings are immutable unless `mut`
- Immutable references: `&T`
- Cannot modify through immutable reference

**Mutable Bindings:**
- `let mut x = ...` creates mutable binding
- Can reassign: `x = new_value`
- Can take mutable reference: `&mut x`

**Mutable References:**
- `&mut T` provides exclusive write access
- Only one mutable reference at a time
- No immutable references while mutable reference exists
- Mutable reference must not outlive the value

**Mutability Rules:**
1. Immutable binding → cannot reassign
2. Mutable binding → can reassign
3. Immutable reference → cannot modify
4. Mutable reference → can modify, exclusive access

### Alias Rules

**Aliasing Definition:**
- Two references pointing to the same memory location
- At the same time (overlapping lifetimes)

**Aliasing Rules:**
1. **Immutable aliasing allowed**: Multiple `&T` to same value
2. **Mutable aliasing forbidden**: Cannot have `&mut T` while other references exist
3. **Mixed aliasing forbidden**: Cannot have `&T` and `&mut T` simultaneously

**Enforcement:**
- Compile-time (ownership system)
- No runtime checks
- Violations are compile errors

### Borrow Rules

**Borrow Definition:**
- Temporary reference to a value
- Does not transfer ownership
- Must not outlive the value

**Immutable Borrow (`&T`):**
- Read-only access
- Multiple immutable borrows allowed
- Cannot modify through immutable borrow
- Lifetime: until borrow goes out of scope

**Mutable Borrow (`&mut T`):**
- Exclusive write access
- Only one mutable borrow at a time
- No other borrows (immutable or mutable) allowed
- Lifetime: until borrow goes out of scope

**Borrow Rules:**
1. Borrow must not outlive the value
2. Cannot borrow moved value
3. Cannot move while borrowed
4. Mutable borrow is exclusive

**Borrow Checker:**
- Compile-time analysis
- Tracks all borrows and their lifetimes
- Ensures no violations

---

## Undefined Behavior (UB)

### Definition

**Undefined Behavior:** Program behavior that is not specified by the language specification. The compiler and runtime make no guarantees about what happens.

### UB in Baremetal Profile

**Allowed UB (Programmer Responsibility):**
- Raw pointer dereference without bounds check
- Uninitialized memory read
- Integer overflow (wraps, no check)
- Out-of-bounds array access
- Use-after-free (if manual memory management)
- Double-free (if manual memory management)

**Rationale:** Baremetal profile trusts the programmer completely. Zero overhead means zero safety nets.

### Forbidden in Kernel Profile

**Compile-Time Errors:**
- Implicit heap allocation
- Panics (must use `Result`)
- Uninitialized memory (must initialize)
- Unbounded loops without explicit control

**Runtime Errors (Abort):**
- Stack overflow (abort, no unwinding)
- Division by zero (abort, no unwinding)
- Null pointer dereference (abort)

**Rationale:** Kernel profile enforces safety at compile time, aborts on runtime errors (no recovery).

### Impossible in Userland Profile

**Guaranteed Safe:**
- Bounds checking on array access
- Stack overflow detection (optional)
- Use-after-free detection (optional, sanitizers)
- Double-free detection
- Panic unwinding (runs destructors)

**Rationale:** Userland profile prioritizes safety. Runtime checks ensure correctness.

---

## Memory Safety Guarantees

### Userland Profile

**Guarantees:**
- ✅ No use-after-free (ownership system + optional sanitizers)
- ✅ No double-free (ownership system)
- ✅ No buffer overflows (bounds checking)
- ✅ No data races (ownership system)
- ⚠️ Stack overflow (detectable, optional)
- ⚠️ Memory leaks (possible with cycles, use `Weak`)

**Runtime Checks:**
- Bounds checking (arrays, vectors)
- Stack overflow detection (optional)
- Memory sanitizers (optional, via flags)

### Kernel Profile

**Guarantees:**
- ✅ No implicit heap allocation (compile-time)
- ✅ No panics (compile-time, must use `Result`)
- ✅ Explicit memory management
- ❌ No bounds checking (removed for performance)
- ❌ No stack overflow detection
- ❌ No memory sanitizers

**Compile-Time Checks:**
- Ownership verification
- Lifetime checking
- No-panic verification
- No-allocation verification

### Baremetal Profile

**Guarantees:**
- ❌ **NONE** - Programmer is trusted completely
- ❌ No bounds checking
- ❌ No overflow detection
- ❌ No use-after-free detection
- ❌ No double-free detection
- ❌ No undefined behavior detection

**Compile-Time Checks:**
- Ownership verification (compile-time only)
- Lifetime checking (compile-time only)
- No runtime checks whatsoever

---

## Memory Layout

### Stack Layout

```
[High Address]
  Return Address
  Saved Registers
  Local Variables
  Parameters
[Low Address]
```

**Growth:** Platform-dependent (usually down)

### Heap Layout

**Managed by allocator (userland only):**
- Free list or similar structure
- Allocation headers
- Alignment padding

### Global Layout

**In data segment:**
- Constants: read-only data segment
- Statics: initialized data segment
- Static mut: initialized data segment (mutable)

---

## Allocation Intent

### Explicit in IR

**IR Instructions:**
- `Alloc { ty, size, dest }` - Explicit heap allocation
- `StackAlloc { ty, size, dest }` - Explicit stack allocation
- `GlobalAlloc { name, ty, init }` - Global allocation

**Profile Enforcement:**
- **userland**: `Alloc` allowed
- **kernel**: `Alloc` forbidden (compile error)
- **baremetal**: `Alloc` forbidden (compile error)

**Forbidden:**
- Implicit allocation in IR
- Backend inferring allocation
- C compiler deciding allocation

---

## Summary

| Aspect | Userland | Kernel | Baremetal |
|--------|----------|--------|-----------|
| **Stack** | ✅ + overflow detection | ✅ | ✅ |
| **Heap** | ✅ Explicit | ❌ Forbidden | ❌ Forbidden |
| **Globals** | ✅ + mutable (unsafe) | ✅ + mutable (unsafe) | ✅ + mutable (unsafe) |
| **Bounds Check** | ✅ Optional | ❌ | ❌ |
| **Overflow Check** | ✅ Optional | ❌ | ❌ |
| **UB Detection** | ✅ Optional (sanitizers) | ❌ | ❌ |
| **Runtime** | ✅ Minimal | ❌ None | ❌ None |

**This specification is frozen. All implementations must conform.**










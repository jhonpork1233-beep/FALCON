# Falcon IR Semantics Specification

**Version:** 1.0  
**Status:** Frozen (Phase-1 Requirement)  
**Last Updated:** 2024-12-29

This document defines that **IR is the single source of truth** for all Falcon language semantics. The backend (C, LLVM, or others) MUST NOT infer, decide, or alter semantics.

---

## Core Principle

**IR Is the Single Source of Truth**

All language semantics MUST be explicitly represented in Falcon IR.

The backend MUST be a mechanical translation only.

---

## Explicit Semantics in IR

### Ownership Moves

**IR Instruction:**
```ir
Move {
    src: IrValue,
    dest: IrValue,
}
```

**Semantics:**
- Ownership transferred from `src` to `dest`
- `src` becomes invalid after move
- No implicit copying
- Backend MUST respect this (no C copy semantics)

**Forbidden:**
- Backend inferring copy instead of move
- C compiler deciding semantics
- Implicit copying

### Borrow Rules

**IR Instructions:**
```ir
BorrowImm {
    src: IrValue,
    dest: IrValue,
    lifetime: String,
}

BorrowMut {
    src: IrValue,
    dest: IrValue,
    lifetime: String,
}
```

**Semantics:**
- `BorrowImm`: Immutable borrow, multiple allowed
- `BorrowMut`: Mutable borrow, exclusive access
- Lifetime tracked explicitly
- Backend MUST respect borrow rules

**Forbidden:**
- Backend ignoring borrow rules
- C pointer semantics overriding borrows
- Implicit aliasing

### Mutability Guarantees

**IR Representation:**
- Mutable bindings: Explicit in variable metadata
- Mutable references: `BorrowMut` instruction
- Immutable references: `BorrowImm` instruction

**Semantics:**
- Mutability is explicit in IR
- Backend MUST respect mutability
- No implicit mutability

### Profile Constraints

**IR Metadata:**
- Profile stored in `IrModule`
- Profile-specific instructions marked
- Profile violations checked at IR validation

**Semantics:**
- Profile rules enforced at IR level
- Backend MUST NOT override profile rules
- Invalid programs rejected before codegen

### Panic vs Abort Behavior

**IR Instruction:**
```ir
Panic {
    message: String,
}
```

**Semantics by Profile:**
- **Userland**: Unwind stack, run destructors, print message, abort
- **Kernel**: Compile error (forbidden)
- **Baremetal**: Compile error (forbidden)

**IR Representation:**
- Panic calls explicit in IR
- Profile validation rejects invalid panics
- Backend translates to appropriate behavior

**Forbidden:**
- Backend deciding panic behavior
- C abort() semantics overriding IR
- Implicit panic handling

### Allocation Intent

**IR Instructions:**
```ir
StackAlloc {
    ty: IrType,
    size: usize,
    dest: IrValue,
}

HeapAlloc {
    ty: IrType,
    size: usize,
    dest: IrValue,
}
```

**Semantics:**
- `StackAlloc`: Stack allocation, lifetime-bound
- `HeapAlloc`: Heap allocation, explicit deallocation
- Allocation intent explicit in IR
- Profile validation enforces rules

**Profile Rules:**
- **Userland**: `HeapAlloc` allowed
- **Kernel**: `HeapAlloc` forbidden (compile error)
- **Baremetal**: `HeapAlloc` forbidden (compile error)

**Forbidden:**
- Backend inferring allocation
- C compiler deciding stack vs heap
- Implicit allocations

---

## Backend Requirements

### Mechanical Translation Only

**Backend MUST:**
- Translate IR instructions to target code
- Preserve all IR semantics
- Respect profile constraints
- Maintain ownership rules

**Backend MUST NOT:**
- Infer semantics
- Decide behavior
- Override IR semantics
- Add implicit behavior
- Use target language semantics to define behavior

### Example: Move Semantics

**IR:**
```ir
Move { src: %x, dest: %y }
```

**Correct C Translation:**
```c
// Move: transfer ownership, original invalid
y = x;
// x is now invalid (compiler ensures no use)
```

**Forbidden:**
```c
// WRONG: Implicit copy
y = x;  // C copy semantics - violates move
// x still valid - violates IR semantics
```

### Example: Allocation

**IR:**
```ir
HeapAlloc { ty: Int32, size: 10, dest: %vec }
```

**Correct Translation:**
```c
// Explicit heap allocation
vec = malloc(10 * sizeof(int32_t));
```

**Forbidden:**
```c
// WRONG: Stack allocation
int32_t vec[10];  // C decides stack - violates IR
```

---

## IR Validation

### Profile Enforcement

**Validation MUST:**
- Check all profile constraints
- Reject invalid programs
- Fail before code generation
- Provide clear error messages

**Validation MUST NOT:**
- Rely on backend to catch errors
- Defer to linker errors
- Use runtime crashes as enforcement

### Ownership Verification

**Validation MUST:**
- Verify ownership rules
- Check borrow lifetimes
- Ensure no use-after-move
- Reject invalid programs

---

## Forbidden Patterns

### ❌ Backend Decides Semantics

```ir
// IR says move
Move { src: %x, dest: %y }
```

```c
// WRONG: C copy semantics
y = x;  // x still valid - violates IR
```

### ❌ Implicit Behavior

```ir
// IR: No allocation specified
Load { addr: %ptr, dest: %val }
```

```c
// WRONG: Backend allocates
int* ptr = malloc(...);  // Implicit allocation
```

### ❌ Profile Rules in Backend

```ir
// IR: HeapAlloc in kernel profile
HeapAlloc { ty: Int32, size: 10, dest: %vec }
```

```c
// WRONG: Backend allows it
vec = malloc(...);  // Should have been rejected at IR validation
```

---

## Summary

**IR is the single source of truth.**

**All semantics are explicit in IR.**

**Backend is mechanical translation only.**

**Profile enforcement happens at IR validation.**

**This specification is frozen. All implementations must conform.**










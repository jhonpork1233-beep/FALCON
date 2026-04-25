# Falcon IR Ownership Specification

This document defines the formal ownership model at the IR (Intermediate Representation) level.

## Ownership Invariant

**For any value V at any point in the program, EXACTLY ONE of these must be true:**

1. **V has exactly ONE owner**
   - Move semantics, exclusive access
   - Owner can transfer ownership (move)
   - Owner can be dropped (destroyed)

2. **V has N immutable borrows (N ≥ 1)**
   - Shared read access
   - No owner modification allowed
   - All borrows must outlive the data

3. **V has exactly ONE mutable borrow**
   - Exclusive write access
   - Owner cannot access V while mutably borrowed
   - No other borrows (immutable or mutable) allowed

**VIOLATIONS are compile-time errors.**

This invariant is preserved across ALL IR passes.

## IR Ownership Instructions

### Move Operations

```
move %v1 → %v2
```
- Transfers ownership from `%v1` to `%v2`
- `%v1` becomes invalid after this instruction
- Compiler verifies `%v1` is not used after move

### Borrow Operations

```
borrow_imm %v1 → %r1 lifetime[L1]
```
- Creates immutable borrow of `%v1`
- Result stored in `%r1`
- Lifetime `L1` tracks borrow validity
- Multiple immutable borrows allowed

```
borrow_mut %v1 → %r2 lifetime[L2]
```
- Creates mutable borrow of `%v1`
- Result stored in `%r2`
- Lifetime `L2` tracks borrow validity
- Only one mutable borrow allowed at a time

### Drop Operations

```
drop %v1
```
- Destroys owner `%v1`
- All borrows of `%v1` must be dead (out of scope)
- Compiler verifies no active borrows before drop

### Copy Operations

```
copy %v1 → %v2
```
- Creates independent copy (for `Copy` types)
- Both `%v1` and `%v2` are valid
- Only for types that implement `Copy` trait

## Lifetime Tracking

Lifetimes in IR are represented as:

```
lifetime[L1] = { start: block_1, end: block_5 }
```

The compiler tracks:
- Where lifetime starts (borrow created)
- Where lifetime ends (borrow goes out of scope)
- All uses of borrowed value within lifetime

## Ownership Verification Pass

The compiler runs an ownership verification pass that checks:

1. **No use-after-move**
   ```ir
   move %v1 → %v2
   load %v1  // ERROR: %v1 used after move
   ```

2. **No simultaneous mut + imm borrows**
   ```ir
   borrow_imm %v1 → %r1 lifetime[L1]
   borrow_mut %v1 → %r2 lifetime[L2]  // ERROR: cannot mutably borrow
   ```

3. **All borrows outlive their data**
   ```ir
   borrow_imm %v1 → %r1 lifetime[L1]
   drop %v1  // ERROR: %v1 dropped while borrowed
   // Lifetime L1 must end before drop
   ```

4. **Mutable borrow exclusivity**
   ```ir
   borrow_mut %v1 → %r1 lifetime[L1]
   borrow_mut %v1 → %r2 lifetime[L2]  // ERROR: cannot mutably borrow twice
   ```

## IR Example

### Source Code
```falcon
func process(data: Vec<i32>) {
    let ref = &data
    println(ref[0])
}
```

### IR (Simplified)
```ir
func process(%data: Vec<i32>) {
    // Borrow data
    %ref = borrow_imm %data lifetime[L1]
    
    // Load element (uses borrow)
    %elem = load_index %ref, 0
    
    // Print (consumes element)
    call println(%elem)
    
    // Lifetime L1 ends here
    // Now we can drop %data
    drop %data
}
```

## Profile-Specific Ownership

### userland Profile
- Full ownership checking
- Bounds checking on borrows
- Stack overflow detection
- Memory sanitizers available

### kernel Profile
- Full ownership checking
- Explicit lifetimes at boundaries
- No implicit allocations
- Strict aliasing rules

### baremetal Profile
- Ownership checking (compile-time only)
- No runtime checks
- Programmer trusted for correctness
- Zero overhead

## Mathematical Formalism

For a value V at program point P:

```
Owner(V, P) ∈ {0, 1}           // Exactly one owner
ImmBorrows(V, P) ∈ {0, 1, ...}  // Zero or more immutable borrows
MutBorrows(V, P) ∈ {0, 1}       // Zero or one mutable borrow

Invariant:
  (Owner(V, P) = 1 AND ImmBorrows(V, P) = 0 AND MutBorrows(V, P) = 0)
  OR
  (Owner(V, P) = 0 AND ImmBorrows(V, P) ≥ 1 AND MutBorrows(V, P) = 0)
  OR
  (Owner(V, P) = 0 AND ImmBorrows(V, P) = 0 AND MutBorrows(V, P) = 1)
```

This invariant must hold at every program point.

---

**Last Updated**: 2024-12-29  
**Version**: 1.0


# Falcon Ownership Rules

This is a one-page reference for Falcon's ownership system. All memory safety derives from these 5 rules.

## The 5 Rules

### 1. Every value has exactly ONE owner

```falcon
let x = 5        // x owns the value 5
let y = x        // y now owns the value, x no longer owns it
// x is invalid here
```

### 2. Owner can be moved (transferred)

```falcon
let data = vec![1, 2, 3]
process(data)    // Ownership moved to process()
// data is no longer accessible
```

### 3. Owner can be borrowed (temporarily)

```falcon
let data = vec![1, 2, 3]
let ref = &data  // Borrowed, not moved
process(&data)   // Borrowed, data still accessible
println(data[0]) // OK, still own data
```

### 4. Borrows must not outlive owner

```falcon
let ref;
{
    let data = vec![1, 2, 3]
    ref = &data  // ERROR: data will be dropped before ref
}
// ref would point to invalid memory
```

### 5. Either N immutable OR 1 mutable borrow (never both)

```falcon
let mut data = vec![1, 2, 3]

// ✅ Multiple immutable borrows
let r1 = &data
let r2 = &data
let r3 = &data

// ✅ One mutable borrow
let r1 = &mut data
// let r2 = &data      // ERROR: cannot borrow while mutably borrowed
// let r2 = &mut data  // ERROR: cannot mutably borrow twice

// ✅ After mutable borrow ends, can borrow again
{
    let r = &mut data
    r.push(4)
}  // r goes out of scope
let r2 = &data  // OK now
```

## Quick Reference

| Operation | Ownership | Borrows |
|-----------|-----------|---------|
| `let y = x` | Moved | None |
| `let r = &x` | Unchanged | 1 immutable |
| `let r = &mut x` | Unchanged | 1 mutable |
| `process(x)` | Moved | None |
| `process(&x)` | Unchanged | 1 immutable |
| `process(&mut x)` | Unchanged | 1 mutable |

## Common Patterns

### Moving into function
```falcon
func take_ownership(v: Vec<i32>) { }
let data = vec![1, 2, 3]
take_ownership(data)  // data moved
```

### Borrowing for function
```falcon
func borrow_only(v: &Vec<i32>) { }
let data = vec![1, 2, 3]
borrow_only(&data)  // data still accessible
```

### Returning ownership
```falcon
func create() -> Vec<i32> {
    vec![1, 2, 3]  // Ownership returned to caller
}
let data = create()  // data owns the vector
```

### Cloning when needed
```falcon
func needs_owned(v: Vec<i32>) { }
let data = vec![1, 2, 3]
needs_owned(data.clone())  // Clone creates new owner
// data still accessible
```

---

**That's it. All safety derives from these 5 rules.**

---

**Last Updated**: 2024-12-29  
**Version**: 1.0


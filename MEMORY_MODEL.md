# Falcon Memory Model

This document defines how memory is managed in Falcon across all compilation profiles.

## Ownership Model

Falcon uses **move semantics by default**. Every value has exactly one owner at any given time.

### Move Semantics

```falcon
let data = vec![1, 2, 3]
process(data)  // data is MOVED to process()
// data is no longer accessible here
```

### Borrowing

**Immutable Borrows** (`&T`):
- Multiple immutable borrows allowed simultaneously
- No mutation allowed
- Borrows must not outlive the owner

```falcon
let data = vec![1, 2, 3]
let ref1 = &data  // immutable borrow
let ref2 = &data  // another immutable borrow (OK)
println(ref1[0])  // OK
println(ref2[0])  // OK
```

**Mutable Borrows** (`&mut T`):
- Exactly one mutable borrow allowed at a time
- No other borrows (immutable or mutable) allowed simultaneously
- Exclusive write access

```falcon
let mut data = vec![1, 2, 3]
let ref1 = &mut data  // mutable borrow
// let ref2 = &data     // ERROR: cannot borrow while mutably borrowed
// let ref2 = &mut data // ERROR: cannot mutably borrow twice
ref1.push(4)  // OK
```

### Ownership Rules

1. Every value has exactly **ONE** owner
2. Owner can be **moved** (transferred to another owner)
3. Owner can be **borrowed** (temporarily shared or exclusively accessed)
4. Borrows must **not outlive** the owner
5. Either **N immutable** OR **1 mutable** borrow (never both)

## Reference Counting

For cases where shared ownership is needed, Falcon provides reference counting types.

### `Rc<T>` - Single-Threaded Reference Counting

```falcon
import std::rc::Rc

let data = Rc::new(vec![1, 2, 3])
let copy1 = data.clone()  // reference count +1
let copy2 = data.clone()  // reference count +1
// data, copy1, copy2 all point to same data
```

**Characteristics:**
- **NOT thread-safe** (panics if sent across threads)
- Cheaper than `Arc` (no atomic operations)
- **NO cycle detection** (use `Weak<T>` to break cycles)
- Single-threaded use only

### `Arc<T>` - Thread-Safe Reference Counting

```falcon
import std::sync::Arc

let data = Arc::new(vec![1, 2, 3])
let copy1 = data.clone()  // atomic reference count +1
let copy2 = data.clone()  // atomic reference count +1
// Can be sent across threads safely
```

**Characteristics:**
- **Thread-safe** (atomic operations)
- Can be sent across threads
- Slight performance cost (atomic operations)
- **NO cycle detection** (use `Weak<T>` to break cycles)

### `Weak<T>` - Weak References (Break Cycles)

```falcon
import std::rc::{Rc, Weak}

// ❌ WRONG - creates memory leak
let a = Rc::new(Node { next: None })
let b = Rc::new(Node { next: Some(a.clone()) })
a.next = Some(b.clone())  // CYCLE! Memory leak

// ✅ CORRECT - breaks cycle with Weak
let a = Rc::new(Node { 
    next: Some(Rc::downgrade(&b))  // Use Weak
})
let b = Rc::new(Node { 
    next: Some(Rc::downgrade(&a))  // Use Weak
})

// Access weak reference
match a.next.unwrap().upgrade() {
    Some(strong_ref) => {
        // Use strong_ref
    }
    None => {
        // Value was dropped
    }
}
```

**Characteristics:**
- Does not keep value alive
- `upgrade()` returns `Option<Rc<T>>` or `Option<Arc<T>>`
- Used to break reference cycles
- Returns `None` if value was dropped

## Profile-Specific Behavior

### userland Profile (Default)

**Memory Safety:**
- Bounds checking in arrays/vectors
- Stack overflow detection
- Memory sanitizers available (optional flags)
- Use-after-free detection (optional)
- Double-free detection

**Allocations:**
- Heap allocations allowed
- Automatic memory management (RAII)
- Panic unwinding runs destructors

**Example:**
```falcon
let data = vec![1, 2, 3]
let value = data[10]  // Panic: index out of bounds
```

### kernel Profile

**Memory Safety:**
- No implicit heap allocations (compile error)
- Explicit memory management required
- No panics allowed (must use `Result`)
- Strict aliasing rules enforced
- Explicit lifetimes at FFI boundaries

**Allocations:**
- Stack allocations only (unless explicitly unsafe)
- Custom allocators via `#[global_allocator]`
- No automatic heap allocation

**Example:**
```falcon
// ❌ ERROR in kernel profile
let data = vec![1, 2, 3]  // Implicit heap allocation

// ✅ CORRECT in kernel profile
let data: [i32; 3] = [1, 2, 3]  // Stack allocation
// OR
unsafe {
    let data = heap_alloc(3 * size_of::<i32>())
    // Manual memory management
}
```

### baremetal Profile

**Memory Safety:**
- **NONE** - Programmer is trusted completely
- Zero runtime checks
- No bounds checking
- No overflow detection
- Direct hardware access

**Allocations:**
- No standard allocator
- Manual memory management only
- Stack allocations only (unless custom allocator)
- Zero runtime overhead

**Example:**
```falcon
unsafe {
    let ptr = 0x4000_0000 as *mut u32
    *ptr = 42  // Direct memory write, no checks
}
```

## Memory Layout

### Stack Allocation

All local variables are stack-allocated by default:

```falcon
func example() {
    let x: i32 = 42        // Stack
    let arr: [i32; 10]     // Stack
    let s = "hello"        // Stack (string literal)
}
```

### Heap Allocation

Heap allocation is explicit in userland, forbidden in kernel (unless unsafe):

```falcon
// userland profile
let vec = vec![1, 2, 3]           // Heap allocation
let string = String::from("hi")   // Heap allocation

// kernel profile
// ❌ vec![1, 2, 3]  // Compile error
// ✅ Use stack arrays or unsafe heap allocation
```

### Zero-Copy Operations

Falcon supports zero-copy operations where possible:

```falcon
let data = vec![1, 2, 3, 4, 5]
let slice = &data[1..4]  // Zero-copy slice, no allocation
```

## Lifetime System

Lifetimes ensure references don't outlive their data:

```falcon
// Explicit lifetime annotation (when needed)
func longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// Most cases: lifetime elision (compiler infers)
func first_word(s: &str) -> &str {
    // Compiler infers lifetime
    s.split(' ').next().unwrap()
}
```

**Lifetime Rules:**
1. Each reference has a lifetime
2. Lifetime must not exceed the data it references
3. Compiler enforces lifetime validity
4. Explicit annotations required only at boundaries (FFI, complex cases)

## Garbage Collection

**Falcon does NOT have garbage collection.**

Memory is managed through:
- Ownership (move semantics)
- RAII (Resource Acquisition Is Initialization)
- Reference counting (`Rc`/`Arc`) when needed
- Manual management (in unsafe/baremetal contexts)

## Memory Safety Guarantees

| Profile | Bounds Checks | Overflow Detection | Use-After-Free | Double-Free | Panic Unwinding |
|---------|--------------|-------------------|----------------|-------------|-----------------|
| userland | ✅ | ✅ (optional) | ✅ (optional) | ✅ | ✅ |
| kernel | ❌ | ❌ | ❌ | ❌ | ❌ (abort only) |
| baremetal | ❌ | ❌ | ❌ | ❌ | ❌ |

---

**Last Updated**: 2024-12-29  
**Version**: 1.0


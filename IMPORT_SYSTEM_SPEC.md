# Falcon Import System — Full Specification

## 0. Design Goals (Non-Negotiable)

1. Imports must be **beautiful and simple** for users
2. Imports must be **profile-aware**
3. Imports must **never bypass safety rules**
4. Imports must **not imply behavior**
5. Imports must scale across **userland / kernel / baremetal**
6. Imports must keep **IR as the single source of truth**

**Additional clarification (IMPORTANT):**

> Falcon libraries may wrap **native C/C++ core libraries that are also used by Python**, but Falcon never embeds or depends on the Python VM. Python is just another consumer of those native libraries.

---

## 1. Core Principle (Locked)

> **Importing a module does not grant permission.**
> **Profiles determine legality. Imports only select symbols.**

This means:

* `import` ≠ enable feature
* `import` ≠ allow allocation
* `import` ≠ allow runtime

Legality is always enforced **after import resolution**.

---

## 2. Import Syntax

```falcon
import crypto
import net::http
import ai::onnx
```

```falcon
import crypto::raw
import crypto::{sha256, hmac}
import ai::onnx as onnx
```

---

## 3. Mandatory Library Layout

```
library/<name>/
├── raw.fc        # Unsafe ABI-level bindings (C/C++)
├── mod.fc        # Safe userland API
├── kernel.fc     # Optional kernel-safe API
├── baremetal.fc  # Optional baremetal API
└── LICENSE
```

**Note:**
Most libraries in `library/` are **thin Falcon bindings over native C/C++ libraries** (the same cores commonly wrapped by Python), with no Python runtime involved.

---

## 4. Profile-Based Import Resolution

When a user writes:

```falcon
import crypto
```

Resolution depends on profile:

| Profile   | Resolution    | Result           |
| --------- | ------------- | ---------------- |
| userland  | `crypto::mod` | ✅ allowed        |
| kernel    | ❌ error       | must be explicit |
| baremetal | ❌ error       | must be explicit |

Kernel:

```falcon
import crypto::kernel
```

Baremetal:

```falcon
import crypto::raw
```

---

## 5. Raw Imports (Universal)

```falcon
import crypto::raw
```

Rules:

* Allowed in **all profiles**
* Requires `unsafe`
* No allocation assumptions
* No runtime assumptions
* Direct mapping to native C ABI

Raw APIs represent the **true capability boundary**.

---

## 6. Language-Level Contracts vs Raw Imports

This distinction is **fundamental** to Falcon's design.

### Raw C Imports (Mechanism Only)

```falcon
import crypto::raw
```

What you get:
- Raw symbols and ABI calls
- No guarantees or semantics
- No memory ownership model
- No profile awareness
- "Here is a function pointer. Don't screw it up."

This is **low-level plumbing** — necessary but not expressive.

### Language-Level Contracts (e.g., `ai::llm`)

```falcon
import ai::llm
```

What you get:
- **Domain-level semantics** the compiler understands
- Memory ownership rules
- Streaming behavior
- Concurrency guarantees
- Profile-aware execution
- Deployment awareness

**The direction is reversed:**

| Type | Flow |
|------|------|
| Raw C import | ABI → You figure it out |
| `ai::llm` | Falcon semantics → backed by C |

### Which Modules Deserve Contract Status?

Very few. Only **domain-defining modules**, not convenience wrappers:

| Module | Why it's primary |
|--------|------------------|
| `ai::llm` | LLM execution, streaming, inference |
| `ai::tensor` | Zero-copy tensor views |
| `net::io` | Sockets, async I/O |
| `fs::io` | File descriptors, mmap |
| `sync` | Atomics, channels, locks |
| `time` | Monotonic clocks, timers |

Everything else (crypto, compression, databases, image processing) lives as **normal libraries built on C**.

### Why `ai::llm` Earns First-Class Treatment

- AI is the **dominant workload** of this era
- It touches memory, compute, I/O, concurrency
- Mistakes here are catastrophic
- Python is failing here at scale

### Internal Structure

```
ai/
├── llm/
│   ├── raw.fc      ← C bindings (llama.cpp, onnxruntime)
│   └── mod.fc      ← Falcon contract (Model, stream, inference)
```

`raw.fc` is the **mechanism** (unsafe C calls).
`mod.fc` is the **meaning** (what a model IS).

> **C is the foundation. Falcon defines meaning.**

---

## 7. Compile-Time Errors (Required)

Example:

```falcon
import crypto
```

Kernel error:

```
KERNEL PROFILE VIOLATION:
Module 'crypto' resolves to a userland API.
Use 'crypto::kernel' or 'crypto::raw'.
```

Baremetal error:

```
BAREMETAL PROFILE VIOLATION:
Only raw bindings are allowed.
```

---

## 8. IR Representation

```text
Import {
  module: "crypto",
  resolved_to: "crypto::raw",
  profile: "kernel"
}
```

* Resolution before IR validation
* Enforcement during IR validation
* Backend never infers behavior

---

## 9. No Implicit Fallbacks (Strict)

❌ Forbidden:

```falcon
import crypto   // silently falls back to raw
```

All fallbacks must be **explicit**.

---

## 10. Standard Library Rules

Userland:

```falcon
import std::io
import std::collections
```

Kernel:

```falcon
import std::core
```

Baremetal:

```falcon
// std not allowed
```

---

## 11. Contributor Rules (Mandatory)

1. Only permissive licenses (MIT, BSD, Apache, zlib)
2. Native libraries may be the same C/C++ cores used by Python
3. No Python VM code allowed
4. `raw.fc` is required
5. Allocation behavior must be documented
6. Profile support must be explicit

---

## 12. Final Locked Rules Summary

1. Imports select names, not permissions
2. Profiles enforce legality
3. Native C libraries are the foundation
4. Python is not a dependency, only a parallel consumer
5. Raw APIs are universal
6. No silent behavior
7. IR records final resolution
8. Backend is mechanical only

---

## Final Verdict

Falcon achieves:

* Python-level ergonomics
* Rust-level safety discipline
* C-level control
* Kernel and baremetal correctness

**By reusing proven native C/C++ cores (often the same ones Python wraps), Falcon saves years of ecosystem work without inheriting Python's runtime costs.**

This is a **foundational, long-term-safe design**.

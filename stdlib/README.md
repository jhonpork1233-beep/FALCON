# Falcon Standard Library

The Falcon standard library provides core functionality for all Falcon programs.

## Structure

```
stdlib/
├── core/          # Core types and traits
├── collections/   # Vec, HashMap, HashSet
├── io/           # File I/O, stdin/stdout
├── net/          # HTTP, WebSocket, TCP/UDP
├── ai/           # Tensor, LLM integration
├── time/         # Time and duration
├── string/        # String utilities
└── math/         # Math functions
```

## Usage

Standard library modules are automatically available:

```falcon
// No import needed for core types
let x: i32 = 42
let s: String = "hello"

// Import specific modules
import std::collections::HashMap
import std::io::println
```

## Profile Support

- **userland**: Full standard library available
- **kernel**: Limited stdlib (no heap allocations)
- **baremetal**: Minimal stdlib (no runtime dependencies)










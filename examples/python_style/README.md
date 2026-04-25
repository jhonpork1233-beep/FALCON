# Python-Style Falcon Examples

This directory contains Falcon examples written in Python-style syntax.

## Syntax Differences

| Feature | Current Syntax | Python-Style |
|---------|----------------|--------------|
| Functions | `func name() { }` | `def name():` |
| If | `if x { }` | `if x:` |
| Else | `} else {` | `else:` |
| While | `while x { }` | `while x:` |
| For | `for i in range(n) { }` | `for i in range(n):` |
| Statements | `x = 5;` | `x = 5` |

## Note

Both syntaxes compile to the same IR and produce identical binaries.
Python-style is **userland only** - kernel/baremetal use Rust-style for systems clarity.

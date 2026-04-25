# Falcon Standard Library Status

The `stdlib/` directory represents the broader standard-library direction for Falcon.

## Current Status

`stdlib/` is not yet a finished, uniformly implemented standard library. Some modules are more like design direction or early scaffolding than stable public surface.

That means:

- not every module is equally wired into the current compiler flow
- not every API in `stdlib/` should be treated as mature
- `library/` is generally the better place to look for runtime-backed functionality that is already connected to the current compiler/runtime implementation

## Existing Areas

The tree currently includes areas such as:

- `core`
- `collections`
- `io`
- `net`
- `ai`

## Recommendation

For public-facing documentation, treat `stdlib/` as active work in progress rather than as a finished standard library.

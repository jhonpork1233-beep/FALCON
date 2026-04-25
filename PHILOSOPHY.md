# Falcon Philosophy

Falcon is motivated by a small number of engineering priorities.

## One Language Across Multiple Execution Contexts

Falcon is trying to keep one language surface across hosted and freestanding targets while letting the compiler enforce what each environment allows.

## Semantics Before Backend

The compiler should determine legality and meaning before code generation. AST and IR exist to make those decisions explicit rather than leaving them to backend behavior.

## Explicit Runtime Boundaries

Falcon prefers explicit imports and profile-aware validation over hidden runtime facilities.

## Conservative Feature Growth

The project is willing to leave features incomplete rather than presenting them as finished before the underlying semantics are strong enough.

## Honest Documentation

The public documentation should describe what the repository actually implements. Falcon benefits more from clear status reporting than from aggressive claims.

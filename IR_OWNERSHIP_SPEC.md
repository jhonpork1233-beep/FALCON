# Falcon IR Ownership Specification

This document describes the ownership-related model currently represented in Falcon IR.

## Purpose

Falcon lowers source code into IR before code generation. Ownership-related instructions in IR are used to make value movement and borrowing explicit enough for verification passes and backend lowering.

## Core IR Concepts

The current IR uses explicit instructions for:

- ownership transfer (`Move`)
- immutable borrow (`BorrowImm`)
- mutable borrow (`BorrowMut`)
- destruction (`Drop`)

These instructions give the compiler a place to reason about value flow after parsing and before backend emission.

## Verification Goals

The ownership pass is intended to catch:

- use of moved values
- conflicting immutable and mutable borrows
- multiple mutable borrows
- drop while still borrowed

## Current Scope

The current implementation is useful, but it should not be described as a complete borrow checker. In particular, the repository does not yet provide:

- full lifetime inference
- complete destructor insertion
- a finished `Copy` classification model
- Rust-equivalent alias analysis

## Why Keep Ownership in IR

Representing movement and borrowing in IR helps Falcon:

- separate source syntax from semantic verification
- keep profile and ownership checks backend-independent
- make later compiler stages work from explicit operations rather than inferred intent

## Practical Interpretation

This document should be read as the current IR ownership model used by the repository, not as a statement that Falcon's ownership system is already final.

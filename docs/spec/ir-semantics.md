# Falcon IR Semantics

This document describes the role IR plays in the current Falcon compiler.

## Purpose

Falcon lowers parsed source into IR before backend code generation. The goal is to make important semantic decisions visible before machine-code or C emission.

## What IR Carries

The current IR is used to represent:

- control flow
- value movement
- borrow-related operations
- calls
- profile-sensitive instructions
- import-resolution results carried forward for validation

## Backend Contract

Backends are expected to implement verified intent rather than invent semantics. In practical terms, Falcon tries to reject invalid profile use and import misuse before backend lowering.

## Why This Matters

Keeping semantics explicit in IR helps Falcon:

- separate parsing from semantic validation
- keep profile rules backend-independent
- make ownership-related checks possible before code generation

## Status

The IR is a real part of the compiler architecture and not just a temporary translation format. At the same time, this repository should not be described as having a fully finalized IR specification across every future version.

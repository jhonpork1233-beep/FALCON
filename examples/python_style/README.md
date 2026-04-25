# Python-Style Falcon Examples

This directory contains `.fpy` examples for Falcon's Python-style userland front end.

## Purpose

These examples demonstrate a friendlier userland syntax that still targets Falcon's normal compiler pipeline after transpilation.

## Key Differences from `.fc`

| Feature | `.fc` style | `.fpy` style |
| --- | --- | --- |
| function declaration | `func main() {}` | `def main():` |
| blocks | braces | indentation |
| statement terminators | semicolons | newline-oriented |

## Scope

`.fpy` is currently intended for `userland` only. Kernel and baremetal examples continue to use the standard Falcon source form.

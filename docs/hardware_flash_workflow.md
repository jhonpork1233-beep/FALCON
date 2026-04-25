# Falcon Hardware Flash Workflow

This document outlines a practical workflow for building and testing current baremetal examples. It is a supporting guide, not a guarantee that every board and target combination is already turnkey.

## Prerequisites

- Falcon compiler with the LLVM path available
- LLVM tools such as `llvm-objcopy`
- target-specific flashing or emulation tools

## Build a Baremetal Example

```bash
falcon build examples/blink_hardware.fc --profile baremetal
```

For some targets you may also want a flat binary:

```bash
llvm-objcopy -O binary blink_hardware.elf blink_hardware.bin
```

## Test in QEMU

```bash
qemu-system-x86_64 -kernel blink_hardware.elf -nographic
```

## Cross-Compile Examples

ARM:

```bash
falcon build examples/blink_hardware.fc --profile baremetal --target aarch64-unknown-none
```

Thumb/Cortex-M style target:

```bash
falcon build examples/blink_hardware.fc --profile baremetal --target thumbv7em-none-eabihf
```

RISC-V:

```bash
falcon build examples/blink_hardware.fc --profile baremetal --target riscv64gc-unknown-none-elf
```

## Flashing

Flashing is target- and board-specific. Typical external tools include:

- OpenOCD
- probe-rs
- vendor-specific tooling

Example OpenOCD flow:

```bash
openocd -f board/st_nucleo_f4.cfg -c "program blink_hardware.bin verify reset exit"
```

Example probe-rs flow:

```bash
probe-rs run --chip STM32F401RE blink_hardware.elf
```

## Verification

It is often useful to verify that freestanding builds are not accidentally pulling in hosted symbols:

```bash
llvm-readelf -S blink_hardware.elf
nm blink_hardware.elf
```

## Important Note

The freestanding pipeline is real, but hardware bring-up remains inherently target-specific. Treat this document as a starting point for experimentation rather than a universal flashing manual.

# Falcon Hardware Flash Workflow

## Prerequisites

- Falcon compiler (with `--features llvm`)
- LLVM toolchain (`llvm-objcopy`)
- Target hardware or QEMU for testing

## Build for x86_64 (QEMU testing)

```bash
# Build baremetal ELF
falcon build examples/blink_hardware.fc --profile baremetal

# Convert to flat binary (if needed)
llvm-objcopy -O binary blink_hardware.elf blink_hardware.bin

# Test in QEMU
qemu-system-x86_64 -kernel blink_hardware.elf -nographic
```

## Cross-compile for ARM (STM32/Cortex-M)

```bash
# Build for ARM target
falcon build blink_hardware.fc --profile baremetal --target aarch64-unknown-none

# Or for Cortex-M (thumb)
falcon build blink_hardware.fc --profile baremetal --target thumbv7em-none-eabihf

# Convert to binary
llvm-objcopy -O binary blink_hardware.elf blink_hardware.bin

# Flash via OpenOCD
openocd -f board/st_nucleo_f4.cfg -c "program blink_hardware.bin verify reset exit"

# Or via probe-rs (Rust tool)
probe-rs run --chip STM32F401RE blink_hardware.elf
```

## Cross-compile for RISC-V

```bash
# Build for RISC-V 64-bit
falcon build blink_hardware.fc --profile baremetal --target riscv64gc-unknown-none-elf

# Flash via J-Link
JLinkExe -device RISCV -if JTAG -speed 4000 -autoconnect 1 -CommanderScript flash.jlink
```

## Profile Requirements

| Profile | Entry Point | Hosted I/O | Heap | Runtime |
|---------|------------|------------|------|---------|
| Userland | `main()` | ✅ Yes | ✅ Yes | Full |
| Kernel | `kernel_main()` | ❌ Rejected | ❌ Rejected | Freestanding |
| Baremetal | `_start()` | ❌ Rejected | ❌ Rejected | Zero |

## Interrupt Handlers

Mark functions with `#[interrupt]` attribute:

```falcon
#[interrupt]
func timer_isr() {
    // Handle timer interrupt
    // Must be in kernel or baremetal profile
}
```

## Verification

After building, verify no libc symbols leaked:

```bash
# Check for libc symbols (should return empty for freestanding)
nm blink_hardware.o | grep -E "printf|puts|malloc|fopen"

# Check ELF sections
llvm-readelf -S blink_hardware.elf
```

# Kernel Profile Scope

This document defines what the kernel profile is designed for and what it is **not** designed for (at least initially).

## Honest Positioning

**Falcon kernel profile is designed for kernel components and drivers, not (initially) for writing entire kernels from scratch. Complete OS development is a long-term goal.**

## ✅ SUPPORTED (v1.0)

### Kernel Modules
- Loadable kernel modules (LKM)
- Device drivers
- File system drivers
- Network drivers
- Character/block device drivers

**Example:**
```falcon
#![no_std]
#![profile=kernel]

#[kernel_module]
func init() -> Result<(), Error> {
    register_device("my_device", &MyDeviceOps)
    Ok(())
}

#[kernel_module]
func cleanup() {
    unregister_device("my_device")
}
```

### Device Drivers
- Hardware abstraction layers
- Driver interfaces
- Interrupt handlers (via unsafe + inline asm)
- DMA operations
- Register access

**Example:**
```falcon
#![profile=kernel]

struct GPIO {
    base: usize,
}

impl GPIO {
    unsafe func set_pin(&self, pin: u8, value: bool) {
        let reg = (self.base + 0x00) as *mut u32
        if value {
            *reg |= 1 << pin
        } else {
            *reg &= !(1 << pin)
        }
    }
}
```

### System Daemons
- Background services
- System utilities
- Real-time components
- Memory allocators (with unsafe)

**Example:**
```falcon
#![profile=kernel]

func system_daemon() -> ! {
    loop {
        handle_events()
        // No panics, must handle all errors
    }
}
```

### Real-Time Components
- Real-time schedulers
- Deterministic systems
- Low-latency components

## ⚠️ LIMITED SUPPORT (v1.0)

### Interrupt Handlers
- **Status**: Possible but requires unsafe + inline assembly
- **Limitation**: No automatic interrupt vector setup
- **Future**: Full interrupt support planned for v2.0

**Example (v1.0 - limited):**
```falcon
#![profile=kernel]

unsafe extern "C" func interrupt_handler() {
    // Manual interrupt handling
    asm!("
        push rax
        // handler code
        pop rax
        iretq
    ")
}
```

### Custom Allocators
- **Status**: Supported via `#[global_allocator]` attribute
- **Limitation**: Must implement full allocator interface
- **Future**: Easier allocator API in v2.0

**Example:**
```falcon
#![profile=kernel]

struct MyAllocator;

unsafe impl GlobalAlloc for MyAllocator {
    unsafe func alloc(&self, layout: Layout) -> *mut u8 {
        // Custom allocation logic
    }
    
    unsafe func dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Custom deallocation logic
    }
}

#[global_allocator]
static ALLOCATOR: MyAllocator = MyAllocator;
```

## ❌ NOT INITIAL FOCUS (Future Goals)

### Complete Kernel Replacement
- **Status**: Long-term goal (v3.0+)
- **Reason**: Requires extensive runtime, boot code, architecture support
- **Current**: Focus on components first

### Architecture-Specific Boot Code
- **Status**: Use baremetal profile instead
- **Reason**: Boot code needs zero runtime, direct hardware
- **Current**: Baremetal profile handles this

**Example (use baremetal for boot):**
```falcon
#![profile=baremetal]

#[entry]
func _start() -> ! {
    // Boot code here
    // Zero runtime, direct hardware
}
```

### Direct Interrupt Vector Control
- **Status**: Future (v2.0)
- **Reason**: Requires architecture-specific code generation
- **Current**: Manual setup via unsafe + asm

## Future Roadmap

### v2.0 Goals
- ✅ Full interrupt handler support
- ✅ Easier custom allocator API
- ✅ More architecture support (ARM, RISC-V)
- ✅ Better FFI for kernel APIs

### v3.0 Goals
- ✅ Bootloader capabilities
- ✅ Initialization code generation
- ✅ Architecture abstraction layer

### Long-Term Vision
- ✅ Complete OS in Falcon
- ✅ Self-hosting kernel
- ✅ Full POSIX compatibility layer
- ✅ Driver framework

## What This Means for Users

### ✅ Use Kernel Profile For:
- Writing Linux kernel modules
- Creating device drivers
- Building system daemons
- Real-time components
- Memory allocators
- Kernel-level utilities

### ⚠️ Use Kernel Profile With Caution For:
- Interrupt handlers (possible but manual)
- Custom allocators (requires full implementation)
- Architecture-specific code (may need baremetal)

### ❌ Don't Use Kernel Profile For (Yet):
- Complete kernel from scratch (use baremetal or wait for v3.0)
- Boot code (use baremetal profile)
- Very low-level hardware init (use baremetal)

### ✅ Use Baremetal Profile For:
- Bootloaders
- Firmware
- Microcontrollers
- RTOS
- Direct hardware access
- Zero-runtime requirements

## Migration Path

If you're building something that doesn't fit kernel profile yet:

1. **Start with userland** - Get it working
2. **Profile and optimize** - Find bottlenecks
3. **Move to kernel** - For components that need it
4. **Use baremetal** - For lowest-level code
5. **Wait for future versions** - For full kernel support

## Community Expectations

**We are honest about limitations:**

- ✅ Kernel profile is production-ready for modules/drivers
- ⚠️ Kernel profile is limited for full OS development
- ✅ Baremetal profile handles boot/low-level
- ✅ Future versions will expand capabilities
- ❌ We won't claim full OS support until it's ready

**This honesty builds trust and sets proper expectations.**

---

**Last Updated**: 2024-12-29  
**Version**: 1.0


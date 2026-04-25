# Baremetal Blink Preview

This file is a forward-looking syntax preview for a more feature-complete baremetal workflow.

It is not intended to represent the current guaranteed parser surface. Instead, it shows the direction of a more explicit board- and target-aware baremetal program once more low-level pieces are fully implemented.

```falcon
#![profile = "baremetal"]
#![target = "thumbv7em-none-eabihf"]
#![linker_script = "boards/stm32f4.ld"]
#![panic = "abort"]

type u32 = i32;

extern func volatile_load32(addr: *const u32) -> u32;
extern func volatile_store32(addr: *mut u32, value: u32);
extern func compiler_fence();

const RCC_AHB1ENR: *mut u32 = 0x40023830 as *mut u32;
const GPIOD_MODER: *mut u32 = 0x40020C00 as *mut u32;
const GPIOD_ODR: *mut u32 = 0x40020C14 as *mut u32;

const GPIOD_ENABLE_BIT: u32 = 1 << 3;
const PIN12_MODE_SHIFT: u32 = 24;
const PIN12_MODE_MASK: u32 = 0b11 << PIN12_MODE_SHIFT;
const PIN12_OUTPUT_MODE: u32 = 0b01 << PIN12_MODE_SHIFT;
const PIN12_ODR_BIT: u32 = 1 << 12;

unsafe func set_bits(addr: *mut u32, mask: u32) {
    let current = volatile_load32(addr as *const u32);
    volatile_store32(addr, current | mask);
}

unsafe func write_masked(addr: *mut u32, clear_mask: u32, set_mask: u32) {
    let current = volatile_load32(addr as *const u32);
    let updated = (current & ~clear_mask) | set_mask;
    volatile_store32(addr, updated);
}

func delay(cycles: u32) {
    let mut remaining = cycles;
    while remaining > 0 {
        compiler_fence();
        remaining = remaining - 1;
    }
}

#[no_mangle]
#[link_section = ".text.start"]
extern "C" func _start() -> ! {
    unsafe {
        set_bits(RCC_AHB1ENR, GPIOD_ENABLE_BIT);
        write_masked(GPIOD_MODER, PIN12_MODE_MASK, PIN12_OUTPUT_MODE);
    }

    loop {
        unsafe {
            set_bits(GPIOD_ODR, PIN12_ODR_BIT);
        }
        delay(2_000_000);

        unsafe {
            write_masked(GPIOD_ODR, PIN12_ODR_BIT, 0);
        }
        delay(2_000_000);
    }
}
```

/**
 * Falcon Arduino Runtime (ATmega328P)
 *
 * Zero-overhead baremetal runtime for Arduino Uno/Nano.
 * Provides:
 * - Interrupt vector table (RESET only, rest = reti)
 * - Stack initialization
 * - BSS zeroing + .data copy from flash
 * - 8-bit volatile register access
 * - Busy-wait delay calibrated for 16 MHz
 */

#include <stdint.h>
#include <stdbool.h>

/* ============================================
 * AVR Register Definitions (ATmega328P)
 * These are ACTUAL hardware addresses
 * ============================================ */

/* I/O register base (AVR uses memory-mapped I/O starting at 0x20) */
#define _SFR_IO8(addr) (*(volatile uint8_t *)((addr) + 0x20))
#define _SFR_MEM8(addr) (*(volatile uint8_t *)(addr))

/* Port B (Pin 8-13 on Arduino Uno) */
#define DDRB   _SFR_IO8(0x04)   /* 0x24 - Data Direction Register B */
#define PORTB  _SFR_IO8(0x05)   /* 0x25 - Output Register B         */
#define PINB   _SFR_IO8(0x03)   /* 0x23 - Input Register B          */

/* Port D (Pin 0-7 on Arduino Uno) */
#define DDRD   _SFR_IO8(0x0A)   /* 0x2A - Data Direction Register D */
#define PORTD  _SFR_IO8(0x0B)   /* 0x2B - Output Register D         */
#define PIND   _SFR_IO8(0x09)   /* 0x29 - Input Register D          */

/* Stack Pointer */
#define SPH _SFR_IO8(0x3E)
#define SPL _SFR_IO8(0x3D)

/* Status Register */
#define SREG _SFR_IO8(0x3F)

/* Pin masks */
#define PB5 (1 << 5)  /* Arduino Pin 13 (built-in LED) */
#define PB0 (1 << 0)  /* Arduino Pin 8 */

/* ============================================
 * Startup Code
 * ============================================ */

/* External symbols from linker script */
extern uint8_t __data_start, __data_end;
extern uint8_t __data_load_start;
extern uint8_t __bss_start, __bss_end;
extern uint8_t __stack_top;

/* Forward declaration of user's _start function */
extern void _start(void);

/* Reset vector handler — called on power-on/reset */
void __reset(void) __attribute__((naked, section(".vectors")));

void __reset(void) {
    /* Initialize stack pointer to top of SRAM */
    SPH = (uint16_t)(&__stack_top) >> 8;
    SPL = (uint16_t)(&__stack_top) & 0xFF;

    /* Copy .data from flash to SRAM */
    uint8_t *src = &__data_load_start;
    uint8_t *dst = &__data_start;
    while (dst < &__data_end) {
        *dst++ = *src++;
    }

    /* Zero .bss */
    dst = &__bss_start;
    while (dst < &__bss_end) {
        *dst++ = 0;
    }

    /* Call user's _start (never returns) */
    _start();

    /* If _start somehow returns, hang */
    while (1) {}
}

/* ============================================
 * Volatile I/O Primitives
 * These are what Falcon's read_volatile /
 * write_volatile call into for baremetal
 * ============================================ */

/* 8-bit volatile read — for AVR I/O registers */
int64_t falcon_read_volatile(int64_t addr) {
    volatile uint8_t *reg = (volatile uint8_t *)((uint16_t)addr);
    return (int64_t)(*reg);
}

/* 8-bit volatile write — for AVR I/O registers */
void falcon_write_volatile(int64_t addr, int64_t value) {
    volatile uint8_t *reg = (volatile uint8_t *)((uint16_t)addr);
    *reg = (uint8_t)(value & 0xFF);
}

/* ============================================
 * Stub I/O (no-op on bare AVR)
 * ============================================ */

void falcon_println(const char* s) { (void)s; }
void falcon_print(const char* s)   { (void)s; }
void falcon_print_int(int64_t v)   { (void)v; }
void falcon_print_i32(int32_t v)   { (void)v; }
void falcon_print_newline(void)    { }

/* Math helpers */
int64_t falcon_abs(int64_t v) { return v < 0 ? -v : v; }
int64_t falcon_min(int64_t a, int64_t b) { return a < b ? a : b; }
int64_t falcon_max(int64_t a, int64_t b) { return a > b ? a : b; }

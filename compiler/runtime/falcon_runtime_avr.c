/* ============================================================
 * Falcon AVR Runtime Stub — Arduino Uno (ATmega328P)
 * ============================================================
 * This is the minimal C runtime for Falcon baremetal on AVR.
 * - No libc
 * - No stdio
 * - No heap
 * Only volatile MMIO access functions.
 *
 * Compile:
 *   avr-gcc -mmcu=atmega328p -Os -c falcon_runtime_avr.c -o falcon_runtime_avr.o
 * ============================================================ */

#include <stdint.h>

/* Volatile write to a memory-mapped I/O register.
 * Falcon passes i64 because it lacks pointer types;
 * we truncate to the AVR-correct 8-bit values. */
void write_volatile(int64_t addr, int64_t val) {
    *(volatile uint8_t *)(uint16_t)addr = (uint8_t)val;
}

/* Volatile read from a memory-mapped I/O register. */
int64_t read_volatile(int64_t addr) {
    return (int64_t)(*(volatile uint8_t *)(uint16_t)addr);
}

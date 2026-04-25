/**
 * Falcon Baremetal Runtime
 * 
 * Zero runtime for baremetal profile.
 * These are stub functions that can be overridden.
 * In true baremetal, you would replace these with
 * hardware-specific implementations (UART, etc.)
 */

#include <stdint.h>
#include <stdbool.h>

/* ============================================
 * Stub I/O Functions
 * Replace these with hardware-specific code
 * ============================================ */

// Platform-specific output - override for your hardware
// Default: no-op (silent)
void falcon_println(const char* s) {
    // No-op in baremetal stub
    // Override with UART/serial output for real hardware
    (void)s;
}

void falcon_print(const char* s) {
    (void)s;
}

void falcon_print_int(int64_t value) {
    (void)value;
}

void falcon_print_i32(int32_t value) {
    (void)value;
}

void falcon_print_newline(void) {
    // No-op
}

/* ============================================
 * Math Functions (pure, inline-able)
 * ============================================ */

int64_t falcon_abs(int64_t value) {
    return value < 0 ? -value : value;
}

int64_t falcon_min(int64_t a, int64_t b) {
    return a < b ? a : b;
}

int64_t falcon_max(int64_t a, int64_t b) {
    return a > b ? a : b;
}

/* ============================================
 * FORBIDDEN in Baremetal Profile
 * These do NOT exist - compiler rejects any
 * code that would reference them
 * ============================================ */

// NO falcon_panic
// NO falcon_alloc / falcon_dealloc
// NO falcon_bounds_check
// NO heap functions of any kind

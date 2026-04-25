/**
 * Falcon Kernel Runtime
 * 
 * Freestanding runtime for kernel profile.
 * NO libc. NO heap. NO panics. NO unwinding.
 * 
 * Only freestanding C11 headers allowed:
 *   <stdint.h>, <stdbool.h>, <stddef.h>, <float.h>, <limits.h>, <stdarg.h>
 * 
 * Debug output is available ONLY when FALCON_KERNEL_DEBUG is defined.
 * Default: OFF (silent, zero hosted behavior).
 */

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/* ============================================
 * Debug Output (off by default)
 * Define FALCON_KERNEL_DEBUG to enable debug
 * output via a user-provided debug_putchar().
 * ============================================ */

#ifdef FALCON_KERNEL_DEBUG
/* User must provide: void debug_putchar(char c); */
extern void debug_putchar(char c);

static void debug_print_str(const char* s) {
    if (s == NULL) return;
    while (*s) {
        debug_putchar(*s++);
    }
}

static void debug_print_int(int64_t value) {
    char buf[21]; /* max i64 decimal digits + sign + null */
    int i = 0;
    uint64_t abs_val;

    if (value < 0) {
        debug_putchar('-');
        abs_val = (uint64_t)(-(value + 1)) + 1;
    } else {
        abs_val = (uint64_t)value;
    }

    if (abs_val == 0) {
        debug_putchar('0');
        return;
    }

    while (abs_val > 0) {
        buf[i++] = '0' + (char)(abs_val % 10);
        abs_val /= 10;
    }
    while (i > 0) {
        debug_putchar(buf[--i]);
    }
}
#endif /* FALCON_KERNEL_DEBUG */

/* ============================================
 * Math Functions (pure, no side effects)
 * Freestanding — no libc dependency
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
 * FORBIDDEN in Kernel Profile
 * These are NOT provided. The compiler rejects
 * any code that would call them.
 * ============================================ */

// NO falcon_panic        — kernel must use Result
// NO falcon_alloc        — kernel must use stack
// NO falcon_dealloc      — no heap
// NO falcon_bounds_check — kernel is trusted
// NO falcon_println      — no hosted I/O
// NO falcon_print        — no hosted I/O
// NO falcon_print_int    — no hosted I/O
// NO falcon_print_float  — no hosted I/O
// NO falcon_print_bool   — no hosted I/O
// NO falcon_input        — no hosted I/O
// NO falcon_random*      — no time() / srand()
// NO falcon_file_*       — no fopen / fread
// NO falcon_os_exec_*    — no popen
// NO falcon_ollama_*     — no hosted process exec

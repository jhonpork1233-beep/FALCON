/**
 * Falcon Runtime Header
 * 
 * Minimal runtime for Falcon userland profile.
 * Size target: ~25-50 KB
 * 
 * Phase 2 implementation.
 */

#ifndef FALCON_RUNTIME_H
#define FALCON_RUNTIME_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================
 * Panic Handler
 * ============================================ */

/**
 * Falcon panic handler.
 * Prints error message and aborts program.
 * This function does not return.
 */
void falcon_panic(const char* message);

/* ============================================
 * Memory Allocator
 * ============================================ */

/**
 * Allocate memory.
 * Returns NULL on failure (does not panic).
 */
void* falcon_alloc(size_t size);

/**
 * Deallocate memory.
 * Safe to call with NULL.
 */
void falcon_dealloc(void* ptr);

/**
 * Reallocate memory.
 * Returns NULL on failure (does not panic).
 */
void* falcon_realloc(void* ptr, size_t new_size);

/* ============================================
 * I/O Functions
 * ============================================ */

/**
 * Print string to stdout with newline.
 */
void falcon_println(const char* s);

/**
 * Print string to stdout without newline.
 */
void falcon_print(const char* s);

/**
 * Print formatted integer.
 */
void falcon_print_int(int64_t value);

/**
 * Print an i32 integer (no newline)
 */
void falcon_print_i32(int32_t value);

/**
 * Print formatted float.
 */
void falcon_print_float(double value);

/**
 * Print boolean as "true" or "false".
 */
void falcon_print_bool(bool value);

/**
 * Print newline only.
 */
void falcon_print_newline(void);

/**
 * Read a line from stdin (like Python's input())
 */
const char* falcon_input(const char* prompt);

/* ============================================
 * Math Functions
 * ============================================ */

/**
 * Get absolute value.
 */
int64_t falcon_abs(int64_t value);

/**
 * Get minimum of two values.
 */
int64_t falcon_min(int64_t a, int64_t b);

/**
 * Get maximum of two values.
 */
int64_t falcon_max(int64_t a, int64_t b);

/**
 * Bounds check for userland profile.
 * - Panics if index is negative.
 * - Panics if length >= 0 and index >= length.
 * - length < 0 means upper bound is unknown (only lower bound check is applied).
 */
void falcon_bounds_check(int64_t index, int64_t length);

/* ============================================
 * Native C Math Library (libm)
 * Same functions Python math module uses!
 * ============================================ */

// Trigonometric
double falcon_sin(double x);
double falcon_cos(double x);
double falcon_tan(double x);
double falcon_asin(double x);
double falcon_acos(double x);
double falcon_atan(double x);

// Exponential and logarithmic
double falcon_sqrt(double x);
double falcon_pow(double base, double exp);
double falcon_exp(double x);
double falcon_log(double x);
double falcon_log10(double x);

// Rounding
double falcon_floor(double x);
double falcon_ceil(double x);
double falcon_round(double x);

// Constants
double falcon_pi(void);
double falcon_e(void);

/* ============================================
 * Python Random Module (Ported)
 * Same API as Python's random module!
 * ============================================ */

/** Set random seed (like Python's random.seed()) */
void falcon_random_seed(int64_t seed);

/** Random float 0.0 to 1.0 (like Python's random.random()) */
double falcon_random(void);

/** Random int a to b inclusive (like Python's random.randint(a, b)) */
int64_t falcon_randint(int64_t a, int64_t b);

/** Random int 0 to n-1 (like Python's random.randrange(n)) */
int64_t falcon_randrange(int64_t n);

/** Get current time as seed */
int64_t falcon_time_seed(void);

/* ============================================
 * String Type (Owned)
 * ============================================ */

/**
 * Falcon owned string.
 * 
 * Rules (per Phase-2 spec):
 * - Owned only
 * - No slicing
 * - No formatting
 * - No UTF tricks
 */
typedef struct {
    char* data;       /* Owned pointer to data */
    size_t len;       /* Length in bytes (not including null terminator) */
    size_t capacity;  /* Allocated capacity */
} FalconString;

/**
 * Create a new string from a C string literal.
 * The input is copied (not borrowed).
 */
FalconString falcon_string_new(const char* data);

/**
 * Create an empty string with given capacity.
 */
FalconString falcon_string_with_capacity(size_t capacity);

/**
 * Drop (free) a string.
 * Safe to call multiple times (will set data to NULL).
 */
void falcon_string_drop(FalconString* s);

/**
 * Get string length.
 */
size_t falcon_string_len(const FalconString* s);

/**
 * Get string capacity.
 */
size_t falcon_string_capacity(const FalconString* s);

/**
 * Get C string pointer (for passing to C functions).
 * Returns empty string if NULL.
 */
const char* falcon_string_as_ptr(const FalconString* s);

/* ============================================
 * Option Type
 * ============================================ */

/**
 * Option type for nullable values.
 * is_some: true if value is present
 */
typedef struct {
    bool is_some;
    /* Actual value stored inline by user code */
} FalconOption;

#define FALCON_SOME(val) ((FalconOption){.is_some = true})
#define FALCON_NONE ((FalconOption){.is_some = false})

/* ============================================
 * Result Type
 * ============================================ */

/**
 * Result type for error handling.
 * is_ok: true if success, false if error
 */
typedef struct {
    bool is_ok;
    /* Actual value/error stored inline by user code */
} FalconResult;

#define FALCON_OK(val) ((FalconResult){.is_ok = true})
#define FALCON_ERR(err) ((FalconResult){.is_ok = false})

/* ============================================
 * Vec Type (minimal)
 * ============================================ */

/**
 * Falcon Vec (dynamic array).
 * 
 * Phase 2 operations:
 * - push
 * - len
 * - drop
 */
typedef struct {
    void* data;       /* Pointer to element data */
    size_t len;       /* Number of elements */
    size_t capacity;  /* Capacity in elements */
    size_t elem_size; /* Size of each element */
} FalconVec;

/**
 * Create a new empty Vec.
 */
FalconVec falcon_vec_new(size_t elem_size);

/**
 * Push an element to the Vec.
 * element is copied (not moved).
 */
void falcon_vec_push(FalconVec* vec, const void* element);

/**
 * Get Vec length.
 */
size_t falcon_vec_len(const FalconVec* vec);

/**
 * Drop (free) a Vec.
 */
void falcon_vec_drop(FalconVec* vec);

/**
 * Get element at index (returns NULL if out of bounds).
 */
void* falcon_vec_get(const FalconVec* vec, size_t index);

/* ============================================
 * Ollama LLM Integration
 * ============================================ */

/**
 * Generate text using local Ollama model.
 * Calls ollama CLI and streams output to stdout.
 * 
 * @param model - model name (e.g., "phi3:mini", "qwen2.5-coder:7b")
 * @param prompt - the prompt to send to the model
 */
void falcon_ollama_generate(const char* model, const char* prompt);

/**
 * Interactive chat with Ollama - reads input and sends with personality
 */
void falcon_ollama_chat(const char* model, const char* personality);

#ifdef __cplusplus
}
#endif

#endif /* FALCON_RUNTIME_H */

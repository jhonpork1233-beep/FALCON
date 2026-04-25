/**
 * Falcon Runtime Implementation
 * 
 * Minimal runtime for Falcon userland profile.
 * Size target: ~25-50 KB
 * 
 * Phase 2 implementation.
 */

#include "falcon_runtime.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* ============================================
 * Panic Handler
 * ============================================ */

void falcon_panic(const char* message) {
    fprintf(stderr, "PANIC: %s\n", message ? message : "(no message)");
    fflush(stderr);
    abort();
}

/* ============================================
 * Memory Allocator
 * ============================================ */

void* falcon_alloc(size_t size) {
    if (size == 0) {
        return NULL;
    }
    void* ptr = malloc(size);
    if (ptr == NULL) {
        /* Allocation failed - return NULL, let caller handle */
        return NULL;
    }
    /* Zero-initialize for safety */
    memset(ptr, 0, size);
    return ptr;
}

void falcon_dealloc(void* ptr) {
    if (ptr != NULL) {
        free(ptr);
    }
}

void* falcon_realloc(void* ptr, size_t new_size) {
    if (new_size == 0) {
        falcon_dealloc(ptr);
        return NULL;
    }
    if (ptr == NULL) {
        return falcon_alloc(new_size);
    }
    return realloc(ptr, new_size);
}

/* ============================================
 * I/O Functions
 * ============================================ */

void falcon_println(const char* s) {
    if (s != NULL) {
        printf("%s\n", s);
    } else {
        printf("\n");
    }
    fflush(stdout);
}

void falcon_print(const char* s) {
    if (s != NULL) {
        printf("%s", s);
    }
    fflush(stdout);
}

void falcon_print_int(int64_t value) {
    printf("%lld", (long long)value);
    fflush(stdout);
}

void falcon_print_i32(int32_t value) {
    printf("%d", value);
    fflush(stdout);
}

void falcon_print_float(double value) {
    printf("%g", value);
    fflush(stdout);
}

void falcon_print_bool(bool value) {
    printf("%s", value ? "true" : "false");
    fflush(stdout);
}

void falcon_print_newline(void) {
    printf("\n");
    fflush(stdout);
}

/* ============================================
 * Input Functions
 * ============================================ */

static char input_buffer[4096];

// Read a line from stdin (like Python's input())
const char* falcon_input(const char* prompt) {
    if (prompt != NULL) {
        printf("%s", prompt);
        fflush(stdout);
    }
    
    if (fgets(input_buffer, sizeof(input_buffer), stdin) == NULL) {
        input_buffer[0] = '\0';
        return input_buffer;
    }
    
    // Remove trailing newline
    size_t len = strlen(input_buffer);
    if (len > 0 && input_buffer[len - 1] == '\n') {
        input_buffer[len - 1] = '\0';
    }
    
    return input_buffer;
}

/* ============================================
 * Math Functions
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
 * Native C Math Library (libm)
 * Same functions Python uses - but native speed!
 * ============================================ */

#include <math.h>

// Trigonometric functions
double falcon_sin(double x) { return sin(x); }
double falcon_cos(double x) { return cos(x); }
double falcon_tan(double x) { return tan(x); }

// Inverse trig
double falcon_asin(double x) { return asin(x); }
double falcon_acos(double x) { return acos(x); }
double falcon_atan(double x) { return atan(x); }

// Exponential and logarithmic
double falcon_sqrt(double x) { return sqrt(x); }
double falcon_pow(double base, double exp) { return pow(base, exp); }
double falcon_exp(double x) { return exp(x); }
double falcon_log(double x) { return log(x); }
double falcon_log10(double x) { return log10(x); }

// Rounding
double falcon_floor(double x) { return floor(x); }
double falcon_ceil(double x) { return ceil(x); }
double falcon_round(double x) { return round(x); }

// Constants
double falcon_pi(void) { return 3.14159265358979323846; }
double falcon_e(void) { return 2.71828182845904523536; }

void falcon_bounds_check(int64_t index, int64_t length) {
    if (index < 0) {
        falcon_panic("Array index out of bounds: negative index");
    }
    if (length >= 0 && index >= length) {
        falcon_panic("Array index out of bounds: index exceeds length");
    }
}

/* ============================================
 * Python Random Module (Ported)
 * Same API as Python's random module - native speed!
 * ============================================ */

#include <time.h>

static int random_initialized = 0;

// Initialize random seed (like Python's random.seed())
void falcon_random_seed(int64_t seed) {
    srand((unsigned int)seed);
    random_initialized = 1;
}

// Random float between 0.0 and 1.0 (like Python's random.random())
double falcon_random(void) {
    if (!random_initialized) {
        srand((unsigned int)time(NULL));
        random_initialized = 1;
    }
    return (double)rand() / (double)RAND_MAX;
}

// Random integer between a and b inclusive (like Python's random.randint(a, b))
int64_t falcon_randint(int64_t a, int64_t b) {
    if (!random_initialized) {
        srand((unsigned int)time(NULL));
        random_initialized = 1;
    }
    if (a > b) {
        int64_t tmp = a;
        a = b;
        b = tmp;
    }
    return a + (rand() % (b - a + 1));
}

// Random choice from range 0 to n-1 (like Python's random.randrange(n))
int64_t falcon_randrange(int64_t n) {
    return falcon_randint(0, n - 1);
}

// Get current time as seed value
int64_t falcon_time_seed(void) {
    return (int64_t)time(NULL);
}

/* ============================================
 * String Type (Owned)
 * ============================================ */

FalconString falcon_string_new(const char* data) {
    FalconString s;
    if (data == NULL) {
        s.data = NULL;
        s.len = 0;
        s.capacity = 0;
        return s;
    }
    
    s.len = strlen(data);
    s.capacity = s.len + 1; /* +1 for null terminator */
    s.data = (char*)falcon_alloc(s.capacity);
    
    if (s.data != NULL) {
        memcpy(s.data, data, s.len);
        s.data[s.len] = '\0';
    } else {
        s.len = 0;
        s.capacity = 0;
    }
    
    return s;
}

FalconString falcon_string_with_capacity(size_t capacity) {
    FalconString s;
    s.len = 0;
    s.capacity = capacity > 0 ? capacity : 1;
    s.data = (char*)falcon_alloc(s.capacity);
    
    if (s.data != NULL) {
        s.data[0] = '\0';
    } else {
        s.capacity = 0;
    }
    
    return s;
}

void falcon_string_drop(FalconString* s) {
    if (s != NULL && s->data != NULL) {
        falcon_dealloc(s->data);
        s->data = NULL;
        s->len = 0;
        s->capacity = 0;
    }
}

size_t falcon_string_len(const FalconString* s) {
    if (s == NULL || s->data == NULL) {
        return 0;
    }
    return s->len;
}

size_t falcon_string_capacity(const FalconString* s) {
    if (s == NULL) {
        return 0;
    }
    return s->capacity;
}

const char* falcon_string_as_ptr(const FalconString* s) {
    if (s == NULL || s->data == NULL) {
        return "";
    }
    return s->data;
}

/* ============================================
 * Vec Type (minimal)
 * ============================================ */

#define VEC_INITIAL_CAPACITY 4
#define VEC_GROWTH_FACTOR 2

FalconVec falcon_vec_new(size_t elem_size) {
    FalconVec vec;
    vec.data = NULL;
    vec.len = 0;
    vec.capacity = 0;
    vec.elem_size = elem_size > 0 ? elem_size : 1;
    return vec;
}

static void falcon_vec_grow(FalconVec* vec) {
    size_t new_capacity;
    void* new_data;
    
    if (vec->capacity == 0) {
        new_capacity = VEC_INITIAL_CAPACITY;
    } else {
        new_capacity = vec->capacity * VEC_GROWTH_FACTOR;
    }
    
    new_data = falcon_realloc(vec->data, new_capacity * vec->elem_size);
    if (new_data != NULL) {
        vec->data = new_data;
        vec->capacity = new_capacity;
    }
}

void falcon_vec_push(FalconVec* vec, const void* element) {
    if (vec == NULL || element == NULL) {
        return;
    }
    
    if (vec->len >= vec->capacity) {
        falcon_vec_grow(vec);
    }
    
    if (vec->len < vec->capacity) {
        char* dest = (char*)vec->data + (vec->len * vec->elem_size);
        memcpy(dest, element, vec->elem_size);
        vec->len++;
    }
}

size_t falcon_vec_len(const FalconVec* vec) {
    if (vec == NULL) {
        return 0;
    }
    return vec->len;
}

void falcon_vec_drop(FalconVec* vec) {
    if (vec != NULL) {
        falcon_dealloc(vec->data);
        vec->data = NULL;
        vec->len = 0;
        vec->capacity = 0;
    }
}

void* falcon_vec_get(const FalconVec* vec, size_t index) {
    if (vec == NULL || vec->data == NULL || index >= vec->len) {
        return NULL;
    }
    return (char*)vec->data + (index * vec->elem_size);
}

/* ============================================
 * Ollama LLM Integration
 * ============================================ */

/**
 * Generate text using local Ollama model
 * Uses popen to call ollama CLI directly
 * 
 * @param model - model name (e.g., "phi3:mini", "qwen2.5-coder:7b")
 * @param prompt - the prompt to send to the model
 */
void falcon_ollama_generate(const char* model, const char* prompt) {
    if (model == NULL || prompt == NULL) {
        printf("[Ollama] Error: NULL model or prompt\n");
        return;
    }
    
    // Build command: ollama run <model> "<prompt>"
    // Escape quotes in prompt for safety
    char cmd[4096];
    snprintf(cmd, sizeof(cmd), "ollama run %s \"%s\"", model, prompt);
    
    printf("[Ollama] Model: %s\n", model);
    printf("[Ollama] Prompt: %s\n", prompt);
    printf("[Ollama] Response:\n");
    fflush(stdout);
    
    // Execute and stream output
    FILE* pipe = _popen(cmd, "r");
    if (pipe == NULL) {
        printf("[Ollama] Error: Failed to run ollama command\n");
        return;
    }
    
    char buffer[256];
    while (fgets(buffer, sizeof(buffer), pipe) != NULL) {
        printf("%s", buffer);
        fflush(stdout);
    }
    
    int status = _pclose(pipe);
    if (status != 0) {
        printf("\n[Ollama] Command exited with code: %d\n", status);
    }
    printf("\n");
    fflush(stdout);
}

/**
 * Helper function to escape special characters for shell command
 * Escapes: " \ and removes newlines
 */
static void escape_for_shell(const char* src, char* dest, size_t dest_size) {
    size_t j = 0;
    for (size_t i = 0; src[i] != '\0' && j < dest_size - 2; i++) {
        char c = src[i];
        if (c == '"') {
            dest[j++] = '\\';
            dest[j++] = '"';
        } else if (c == '\\') {
            dest[j++] = '\\';
            dest[j++] = '\\';
        } else if (c == '\n') {
            dest[j++] = ' ';  // Replace newlines with spaces
        } else if (c == '\r') {
            // Skip carriage returns
        } else {
            dest[j++] = c;
        }
    }
    dest[j] = '\0';
}

/**
 * Interactive chat with Ollama - reads user input and sends with system personality
 * 
 * @param model - model name (e.g., "llama3.1:8b")
 * @param personality - system prompt defining AI personality
 */
void falcon_ollama_chat(const char* model, const char* personality) {
    if (model == NULL) {
        printf("[Ollama] Error: NULL model\n");
        return;
    }
    
    // Read user input
    printf("You: ");
    fflush(stdout);
    
    char user_input[2048];
    if (fgets(user_input, sizeof(user_input), stdin) == NULL) {
        return;
    }
    
    // Remove trailing newline
    size_t len = strlen(user_input);
    if (len > 0 && user_input[len - 1] == '\n') {
        user_input[len - 1] = '\0';
    }
    
    // Check for quit
    if (strcmp(user_input, "quit") == 0 || strcmp(user_input, "exit") == 0) {
        printf("\n[Adwaith's AI] Goodbye! 👋\n");
        exit(0);
    }
    
    // Escape the personality and user input for shell safety
    char escaped_personality[4096];
    char escaped_input[2048];
    
    if (personality != NULL) {
        escape_for_shell(personality, escaped_personality, sizeof(escaped_personality));
    } else {
        escaped_personality[0] = '\0';
    }
    escape_for_shell(user_input, escaped_input, sizeof(escaped_input));
    
    // Build full prompt with personality
    char full_prompt[8192];
    if (strlen(escaped_personality) > 0) {
        snprintf(full_prompt, sizeof(full_prompt), "%s User: %s", escaped_personality, escaped_input);
    } else {
        snprintf(full_prompt, sizeof(full_prompt), "%s", escaped_input);
    }
    
    // Build command
    char cmd[16384];
    snprintf(cmd, sizeof(cmd), "ollama run %s \"%s\"", model, full_prompt);
    
    printf("\n🤖 Adwaith's AI: ");
    fflush(stdout);
    
    // Execute and stream output
    FILE* pipe = _popen(cmd, "r");
    if (pipe == NULL) {
        printf("[Error] Failed to run ollama\n");
        return;
    }
    
    char buffer[256];
    while (fgets(buffer, sizeof(buffer), pipe) != NULL) {
        printf("%s", buffer);
        fflush(stdout);
    }
    
    _pclose(pipe);
    printf("\n");
    fflush(stdout);
}



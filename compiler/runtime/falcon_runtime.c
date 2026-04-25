/**
 * Falcon Runtime Implementation
 *
 * Minimal runtime for Falcon userland profile.
 * Size target: ~25-50 KB
 *
 * Phase 2 implementation.
 */

#include "falcon_runtime.h"
#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* ============================================
 * Panic Handler
 * ============================================ */

void falcon_panic(const char *message) {
  fprintf(stderr, "PANIC: %s\n", message ? message : "(no message)");
  fflush(stderr);
  abort();
}

/* ============================================
 * Memory Allocator
 * ============================================ */

void *falcon_alloc(size_t size) {
  if (size == 0) {
    return NULL;
  }
  void *ptr = malloc(size);
  if (ptr == NULL) {
    /* Allocation failed - return NULL, let caller handle */
    return NULL;
  }
  /* Zero-initialize for safety */
  memset(ptr, 0, size);
  return ptr;
}

void falcon_dealloc(void *ptr) {
  if (ptr != NULL) {
    free(ptr);
  }
}

void *falcon_realloc(void *ptr, size_t new_size) {
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

void falcon_println(const char *s) {
  if (s != NULL) {
    printf("%s\n", s);
  } else {
    printf("\n");
  }
  fflush(stdout);
}

void falcon_print(const char *s) {
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

// Keep a small ring of input buffers so sequential falcon_input calls
// do not overwrite previously returned pointers immediately.
#define FALCON_INPUT_SLOTS 16
#define FALCON_INPUT_BUF_SIZE 4096
static char input_buffers[FALCON_INPUT_SLOTS][FALCON_INPUT_BUF_SIZE];
static size_t input_slot_index = 0;

#define FALCON_STR_TMP_SLOTS 16
#define FALCON_STR_TMP_BUF_SIZE 16384
static char str_tmp_buffers[FALCON_STR_TMP_SLOTS][FALCON_STR_TMP_BUF_SIZE];
static size_t str_tmp_slot_index = 0;

static char *next_str_tmp_buffer(void);
static void trim_in_place(char *s);

// Read a line from stdin (like Python's input())
const char *falcon_input(const char *prompt) {
  char *input_buffer = input_buffers[input_slot_index];
  input_slot_index = (input_slot_index + 1) % FALCON_INPUT_SLOTS;

  if (prompt != NULL) {
    printf("%s", prompt);
    fflush(stdout);
  }

  if (fgets(input_buffer, FALCON_INPUT_BUF_SIZE, stdin) == NULL) {
    input_buffer[0] = '\0';
    return input_buffer;
  }

  // Remove trailing CR/LF
  size_t len = strlen(input_buffer);
  while (len > 0 &&
         (input_buffer[len - 1] == '\n' || input_buffer[len - 1] == '\r')) {
    input_buffer[len - 1] = '\0';
    len--;
  }

  trim_in_place(input_buffer);
  return input_buffer;
}

const char *falcon_str_concat(const char *a, const char *b) {
  const char *lhs = a != NULL ? a : "";
  const char *rhs = b != NULL ? b : "";
  char *out = next_str_tmp_buffer();
  snprintf(out, FALCON_STR_TMP_BUF_SIZE, "%s%s", lhs, rhs);
  return out;
}

bool falcon_str_eq(const char *a, const char *b) {
  const char *lhs = a != NULL ? a : "";
  const char *rhs = b != NULL ? b : "";
  return strcmp(lhs, rhs) == 0;
}

bool falcon_str_is_empty(const char *s) { return s == NULL || s[0] == '\0'; }

int64_t falcon_str_len(const char *s) {
  if (s == NULL) {
    return 0;
  }
  return (int64_t)strlen(s);
}

int64_t falcon_str_find_from(const char *haystack, const char *needle,
                             int64_t start) {
  if (haystack == NULL || needle == NULL) {
    return -1;
  }

  size_t haystack_len = strlen(haystack);
  if (start < 0) {
    start = 0;
  }
  if ((size_t)start > haystack_len) {
    return -1;
  }
  if (needle[0] == '\0') {
    return start;
  }

  const char *found = strstr(haystack + start, needle);
  if (found == NULL) {
    return -1;
  }
  return (int64_t)(found - haystack);
}

const char *falcon_str_substr(const char *s, int64_t start, int64_t length) {
  if (s == NULL) {
    return "";
  }

  size_t s_len = strlen(s);
  if (start < 0) {
    start = 0;
  }
  if ((size_t)start > s_len) {
    start = (int64_t)s_len;
  }
  if (length < 0) {
    length = 0;
  }

  size_t remaining = s_len - (size_t)start;
  size_t copy_len = (size_t)length;
  if (copy_len > remaining) {
    copy_len = remaining;
  }
  if (copy_len > FALCON_STR_TMP_BUF_SIZE - 1) {
    copy_len = FALCON_STR_TMP_BUF_SIZE - 1;
  }

  char *out = next_str_tmp_buffer();
  if (copy_len > 0) {
    memcpy(out, s + start, copy_len);
  }
  out[copy_len] = '\0';
  return out;
}

const char *falcon_str_replace_all(const char *input, const char *target,
                                   const char *replacement) {
  const char *source = input != NULL ? input : "";
  const char *needle = target != NULL ? target : "";
  const char *repl = replacement != NULL ? replacement : "";

  char *out = next_str_tmp_buffer();
  out[0] = '\0';

  size_t needle_len = strlen(needle);
  if (needle_len == 0) {
    snprintf(out, FALCON_STR_TMP_BUF_SIZE, "%s", source);
    return out;
  }

  size_t repl_len = strlen(repl);
  const char *scan = source;
  size_t written = 0;

  while (scan[0] != '\0' && written + 1 < FALCON_STR_TMP_BUF_SIZE) {
    const char *hit = strstr(scan, needle);
    if (hit == NULL) {
      size_t tail_len = strlen(scan);
      if (tail_len > FALCON_STR_TMP_BUF_SIZE - written - 1) {
        tail_len = FALCON_STR_TMP_BUF_SIZE - written - 1;
      }
      memcpy(out + written, scan, tail_len);
      written += tail_len;
      break;
    }

    size_t prefix_len = (size_t)(hit - scan);
    if (prefix_len > FALCON_STR_TMP_BUF_SIZE - written - 1) {
      prefix_len = FALCON_STR_TMP_BUF_SIZE - written - 1;
    }
    if (prefix_len > 0) {
      memcpy(out + written, scan, prefix_len);
      written += prefix_len;
    }

    size_t repl_copy_len = repl_len;
    if (repl_copy_len > FALCON_STR_TMP_BUF_SIZE - written - 1) {
      repl_copy_len = FALCON_STR_TMP_BUF_SIZE - written - 1;
    }
    if (repl_copy_len > 0) {
      memcpy(out + written, repl, repl_copy_len);
      written += repl_copy_len;
    }

    scan = hit + needle_len;
  }

  out[written] = '\0';
  return out;
}

const char *falcon_str_strip_html_tags(const char *input) {
  const char *source = input != NULL ? input : "";
  char *out = next_str_tmp_buffer();

  size_t written = 0;
  int in_tag = 0;

  for (size_t i = 0; source[i] != '\0' && written + 1 < FALCON_STR_TMP_BUF_SIZE;
       i++) {
    char c = source[i];
    if (c == '<') {
      in_tag = 1;
      continue;
    }
    if (c == '>') {
      in_tag = 0;
      continue;
    }
    if (!in_tag) {
      out[written++] = c;
    }
  }

  out[written] = '\0';
  return out;
}

const char *falcon_str_json_extract_values(const char *json, const char *key,
                                           int64_t max_items) {
  const char *source = json != NULL ? json : "";
  const char *field = key != NULL ? key : "";
  char *out = next_str_tmp_buffer();
  out[0] = '\0';

  if (max_items <= 0 || field[0] == '\0') {
    return out;
  }

  char marker[256];
  snprintf(marker, sizeof(marker), "\"%s\":\"", field);
  size_t marker_len = strlen(marker);

  const char *scan = source;
  int64_t count = 0;
  size_t written = 0;

  while (count < max_items && scan[0] != '\0' &&
         written + 1 < FALCON_STR_TMP_BUF_SIZE) {
    const char *start = strstr(scan, marker);
    if (start == NULL) {
      break;
    }

    const char *value = start + marker_len;
    if (written + 2 < FALCON_STR_TMP_BUF_SIZE) {
      out[written++] = '-';
      out[written++] = ' ';
    }

    int escaped = 0;
    while (value[0] != '\0' && written + 1 < FALCON_STR_TMP_BUF_SIZE) {
      char c = value[0];
      value++;

      if (escaped) {
        if (c == 'n' || c == 'r' || c == 't') {
          out[written++] = ' ';
        } else {
          out[written++] = c;
        }
        escaped = 0;
        continue;
      }

      if (c == '\\') {
        escaped = 1;
        continue;
      }

      if (c == '"') {
        break;
      }

      out[written++] = c;
    }

    if (written + 1 < FALCON_STR_TMP_BUF_SIZE) {
      out[written++] = '\n';
    }

    out[written] = '\0';
    count++;
    scan = value;
  }

  out[written] = '\0';
  return out;
}

static void trim_in_place(char *s) {
  if (s == NULL) {
    return;
  }

  size_t len = strlen(s);
  while (len > 0 && isspace((unsigned char)s[len - 1])) {
    s[len - 1] = '\0';
    len--;
  }

  size_t start = 0;
  while (s[start] != '\0' && isspace((unsigned char)s[start])) {
    start++;
  }

  if (start > 0) {
    memmove(s, s + start, strlen(s + start) + 1);
  }
}

static char *next_str_tmp_buffer(void) {
  char *slot = str_tmp_buffers[str_tmp_slot_index];
  str_tmp_slot_index = (str_tmp_slot_index + 1) % FALCON_STR_TMP_SLOTS;
  slot[0] = '\0';
  return slot;
}

/* ============================================
 * Math Functions
 * ============================================ */

int64_t falcon_abs(int64_t value) { return value < 0 ? -value : value; }

int64_t falcon_min(int64_t a, int64_t b) { return a < b ? a : b; }

int64_t falcon_max(int64_t a, int64_t b) { return a > b ? a : b; }

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

/* Array length for iterator protocol.
 * Arrays are represented as a pointer with length stored at offset -1.
 * For now, returns a sentinel; actual array metadata tracking is TODO. */
int64_t falcon_array_len(void *arr) {
  if (arr == NULL) {
    return 0;
  }
  /* For tagged arrays, length is stored in the 8 bytes before the data pointer
   */
  int64_t *meta = (int64_t *)arr;
  return meta[-1];
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
int64_t falcon_randrange(int64_t n) { return falcon_randint(0, n - 1); }

// Get current time as seed value
int64_t falcon_time_seed(void) { return (int64_t)time(NULL); }

/* ============================================
 * String Type (Owned)
 * ============================================ */

FalconString falcon_string_new(const char *data) {
  FalconString s;
  if (data == NULL) {
    s.data = NULL;
    s.len = 0;
    s.capacity = 0;
    return s;
  }

  s.len = strlen(data);
  s.capacity = s.len + 1; /* +1 for null terminator */
  s.data = (char *)falcon_alloc(s.capacity);

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
  s.data = (char *)falcon_alloc(s.capacity);

  if (s.data != NULL) {
    s.data[0] = '\0';
  } else {
    s.capacity = 0;
  }

  return s;
}

void falcon_string_drop(FalconString *s) {
  if (s != NULL && s->data != NULL) {
    falcon_dealloc(s->data);
    s->data = NULL;
    s->len = 0;
    s->capacity = 0;
  }
}

size_t falcon_string_len(const FalconString *s) {
  if (s == NULL || s->data == NULL) {
    return 0;
  }
  return s->len;
}

size_t falcon_string_capacity(const FalconString *s) {
  if (s == NULL) {
    return 0;
  }
  return s->capacity;
}

const char *falcon_string_as_ptr(const FalconString *s) {
  if (s == NULL || s->data == NULL) {
    return "";
  }
  return s->data;
}

/* ============================================
 * String Utility Functions (const char*)
 * Callable from Falcon as extern functions
 * ============================================ */

int64_t falcon_strlen(const char *s) {
  if (s == NULL) return 0;
  return (int64_t)strlen(s);
}

const char *falcon_str_slice(const char *s, int64_t start, int64_t end) {
  if (s == NULL) return "";
  int64_t len = (int64_t)strlen(s);
  if (start < 0) start = 0;
  if (end > len) end = len;
  if (start >= end) return "";
  size_t slice_len = (size_t)(end - start);
  char *result = (char *)falcon_alloc(slice_len + 1);
  if (result) {
    memcpy(result, s + start, slice_len);
    result[slice_len] = '\0';
  }
  return result;
}

int64_t falcon_str_contains(const char *haystack, const char *needle) {
  if (haystack == NULL || needle == NULL) return 0;
  return strstr(haystack, needle) != NULL ? 1 : 0;
}

int64_t falcon_str_equals(const char *a, const char *b) {
  if (a == NULL && b == NULL) return 1;
  if (a == NULL || b == NULL) return 0;
  return strcmp(a, b) == 0 ? 1 : 0;
}

int64_t falcon_str_char_at(const char *s, int64_t index) {
  if (s == NULL) return 0;
  int64_t len = (int64_t)strlen(s);
  if (index < 0 || index >= len) return 0;
  return (int64_t)(unsigned char)s[index];
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

static void falcon_vec_grow(FalconVec *vec) {
  size_t new_capacity;
  void *new_data;

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

void falcon_vec_push(FalconVec *vec, const void *element) {
  if (vec == NULL || element == NULL) {
    return;
  }

  if (vec->len >= vec->capacity) {
    falcon_vec_grow(vec);
  }

  if (vec->len < vec->capacity) {
    char *dest = (char *)vec->data + (vec->len * vec->elem_size);
    memcpy(dest, element, vec->elem_size);
    vec->len++;
  }
}

size_t falcon_vec_len(const FalconVec *vec) {
  if (vec == NULL) {
    return 0;
  }
  return vec->len;
}

void falcon_vec_drop(FalconVec *vec) {
  if (vec != NULL) {
    falcon_dealloc(vec->data);
    vec->data = NULL;
    vec->len = 0;
    vec->capacity = 0;
  }
}

void *falcon_vec_get(const FalconVec *vec, size_t index) {
  if (vec == NULL || vec->data == NULL || index >= vec->len) {
    return NULL;
  }
  return (char *)vec->data + (index * vec->elem_size);
}

/* ============================================
 * HashMap Type (open-addressing, string keys)
 * ============================================ */

#define HASHMAP_INITIAL_CAPACITY 16
#define HASHMAP_LOAD_FACTOR 0.75
#define HASHMAP_GROWTH_FACTOR 2
#define HASHMAP_TOMBSTONE ((const char *)(uintptr_t)1)

typedef struct {
  const char *key;
  int64_t value;
} FalconHashEntry;

typedef struct {
  FalconHashEntry *entries;
  size_t capacity;
  size_t len;
  size_t occupied;
} FalconHashMap;

static uint64_t falcon_hash_string(const char *key) {
  uint64_t hash = 14695981039346656037ULL;
  while (*key) {
    hash ^= (uint8_t)*key++;
    hash *= 1099511628211ULL;
  }
  return hash;
}

FalconHashMap *falcon_hashmap_new(void) {
  FalconHashMap *map = (FalconHashMap *)falcon_alloc(sizeof(FalconHashMap));
  if (map == NULL)
    return NULL;
  map->entries = (FalconHashEntry *)falcon_alloc(HASHMAP_INITIAL_CAPACITY *
                                                 sizeof(FalconHashEntry));
  map->capacity = HASHMAP_INITIAL_CAPACITY;
  map->len = 0;
  map->occupied = 0;
  return map;
}

static void falcon_hashmap_resize(FalconHashMap *map) {
  size_t new_cap = map->capacity * HASHMAP_GROWTH_FACTOR;
  FalconHashEntry *new_entries =
      (FalconHashEntry *)falcon_alloc(new_cap * sizeof(FalconHashEntry));
  if (new_entries == NULL)
    return;
  for (size_t i = 0; i < map->capacity; i++) {
    if (map->entries[i].key != NULL &&
        map->entries[i].key != HASHMAP_TOMBSTONE) {
      uint64_t h = falcon_hash_string(map->entries[i].key);
      size_t idx = h % new_cap;
      while (new_entries[idx].key != NULL) {
        idx = (idx + 1) % new_cap;
      }
      new_entries[idx] = map->entries[i];
    }
  }
  falcon_dealloc(map->entries);
  map->entries = new_entries;
  map->capacity = new_cap;
  map->occupied = map->len;
}

void falcon_hashmap_insert(FalconHashMap *map, const char *key, int64_t value) {
  if (map == NULL || key == NULL)
    return;
  if ((double)map->occupied / (double)map->capacity > HASHMAP_LOAD_FACTOR) {
    falcon_hashmap_resize(map);
  }
  uint64_t h = falcon_hash_string(key);
  size_t idx = h % map->capacity;
  while (map->entries[idx].key != NULL) {
    if (map->entries[idx].key != HASHMAP_TOMBSTONE &&
        strcmp(map->entries[idx].key, key) == 0) {
      map->entries[idx].value = value;
      return;
    }
    idx = (idx + 1) % map->capacity;
  }
  map->entries[idx].key = key;
  map->entries[idx].value = value;
  map->len++;
  map->occupied++;
}

int64_t falcon_hashmap_get(const FalconHashMap *map, const char *key) {
  if (map == NULL || key == NULL || map->len == 0)
    return 0;
  uint64_t h = falcon_hash_string(key);
  size_t idx = h % map->capacity;
  while (map->entries[idx].key != NULL) {
    if (map->entries[idx].key != HASHMAP_TOMBSTONE &&
        strcmp(map->entries[idx].key, key) == 0) {
      return map->entries[idx].value;
    }
    idx = (idx + 1) % map->capacity;
  }
  return 0;
}

bool falcon_hashmap_contains(const FalconHashMap *map, const char *key) {
  if (map == NULL || key == NULL || map->len == 0)
    return false;
  uint64_t h = falcon_hash_string(key);
  size_t idx = h % map->capacity;
  while (map->entries[idx].key != NULL) {
    if (map->entries[idx].key != HASHMAP_TOMBSTONE &&
        strcmp(map->entries[idx].key, key) == 0) {
      return true;
    }
    idx = (idx + 1) % map->capacity;
  }
  return false;
}

bool falcon_hashmap_remove(FalconHashMap *map, const char *key) {
  if (map == NULL || key == NULL || map->len == 0)
    return false;
  uint64_t h = falcon_hash_string(key);
  size_t idx = h % map->capacity;
  while (map->entries[idx].key != NULL) {
    if (map->entries[idx].key != HASHMAP_TOMBSTONE &&
        strcmp(map->entries[idx].key, key) == 0) {
      map->entries[idx].key = HASHMAP_TOMBSTONE;
      map->entries[idx].value = 0;
      map->len--;
      return true;
    }
    idx = (idx + 1) % map->capacity;
  }
  return false;
}

size_t falcon_hashmap_len(const FalconHashMap *map) {
  if (map == NULL)
    return 0;
  return map->len;
}

void falcon_hashmap_drop(FalconHashMap *map) {
  if (map != NULL) {
    falcon_dealloc(map->entries);
    falcon_dealloc(map);
  }
}

/* ============================================
 * Ollama LLM Integration
 * ============================================ */

static void escape_for_shell(const char *src, char *dest, size_t dest_size);
static const char *resolve_ollama_command(void);
static const char *quote_command_if_needed(const char *command);
static int write_prompt_temp_file(const char *prompt, char *out_path,
                                  size_t out_path_size);
static char *next_exec_result_buffer(void);
static void run_ollama_stream(const char *model, const char *full_prompt);

#define FALCON_EXEC_RESULT_SLOTS 4
#define FALCON_EXEC_RESULT_BUF_SIZE 65536
static char exec_result_buffers[FALCON_EXEC_RESULT_SLOTS]
                               [FALCON_EXEC_RESULT_BUF_SIZE];
static size_t exec_result_slot_index = 0;

/**
 * Generate text using local Ollama model
 * Uses popen to call ollama CLI directly
 *
 * @param model - model name (e.g., "phi3:mini", "qwen2.5-coder:7b")
 * @param prompt - the prompt to send to the model
 */
void falcon_ollama_generate(const char *model, const char *prompt) {
  if (model == NULL || prompt == NULL) {
    printf("[Ollama] Error: NULL model or prompt\n");
    return;
  }

  printf("[Ollama] Model: %s\n", model);
  printf("[Ollama] Prompt: %s\n", prompt);
  printf("[Ollama] Response:\n");
  fflush(stdout);

  run_ollama_stream(model, prompt);
  printf("\n");
  fflush(stdout);
}

/**
 * List local models available in Ollama.
 */
void falcon_ollama_list_models(void) {
  printf("[Ollama] Available local models:\n");
  fflush(stdout);

  const char *ollama_cmd = resolve_ollama_command();
  char cmd[512];
  snprintf(cmd, sizeof(cmd), "%s list", ollama_cmd);
  FILE *pipe = _popen(cmd, "r");
  if (pipe == NULL) {
    printf("[Ollama] Error: Failed to run `ollama list`\n");
    printf("[Ollama] Hint: set FALCON_OLLAMA_CMD to full executable path if "
           "needed.\n");
    return;
  }

  char buffer[256];
  while (fgets(buffer, sizeof(buffer), pipe) != NULL) {
    printf("%s", buffer);
  }

  int status = _pclose(pipe);
  if (status != 0) {
    printf("[Ollama] `ollama list` exited with code: %d\n", status);
    printf("[Ollama] If models are missing, run: ollama pull <model>\n");
  }

  printf("\n");
  fflush(stdout);
}

const char *falcon_os_exec_capture(const char *command) {
  if (command == NULL || command[0] == '\0') {
    return "";
  }

  char *out = next_exec_result_buffer();
  out[0] = '\0';

  FILE *pipe = _popen(command, "r");
  if (pipe == NULL) {
    return out;
  }

  size_t written = 0;
  while (written + 1 < FALCON_EXEC_RESULT_BUF_SIZE) {
    size_t remaining = FALCON_EXEC_RESULT_BUF_SIZE - written - 1;
    size_t chunk = fread(out + written, 1, remaining, pipe);
    if (chunk == 0) {
      break;
    }
    written += chunk;
  }
  out[written] = '\0';

  _pclose(pipe);
  return out;
}

int64_t falcon_os_exec_stream(const char *command) {
  if (command == NULL || command[0] == '\0') {
    return -1;
  }

  FILE *pipe = _popen(command, "r");
  if (pipe == NULL) {
    return -1;
  }

  char buffer[256];
  while (fgets(buffer, sizeof(buffer), pipe) != NULL) {
    printf("%s", buffer);
    fflush(stdout);
  }

  return (int64_t)_pclose(pipe);
}

/**
 * Helper function to escape special characters for shell command.
 * Escapes: " and \ ; removes newlines.
 */
static void escape_for_shell(const char *src, char *dest, size_t dest_size) {
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
      dest[j++] = ' '; // Replace newlines with spaces
    } else if (c == '\r') {
      // Skip carriage returns
    } else {
      dest[j++] = c;
    }
  }
  dest[j] = '\0';
}

/**
 * Resolve the Ollama executable command.
 * You can override with environment variable:
 *   FALCON_OLLAMA_CMD=C:\Path\To\ollama.exe
 */
static const char *quote_command_if_needed(const char *command) {
  static char quoted[1024];
  if (command == NULL || command[0] == '\0') {
    return "ollama";
  }

  size_t len = strlen(command);
  int already_quoted = len >= 2 && command[0] == '"' && command[len - 1] == '"';
  int has_space = strchr(command, ' ') != NULL || strchr(command, '\t') != NULL;

  if (has_space && !already_quoted) {
    snprintf(quoted, sizeof(quoted), "\"%s\"", command);
    return quoted;
  }

  snprintf(quoted, sizeof(quoted), "%s", command);
  return quoted;
}

static int write_prompt_temp_file(const char *prompt, char *out_path,
                                  size_t out_path_size) {
  if (prompt == NULL || out_path == NULL || out_path_size == 0) {
    return 0;
  }

  char tmp_name[L_tmpnam];
  if (tmpnam(tmp_name) == NULL) {
    return 0;
  }

  int copied = snprintf(out_path, out_path_size, "%s", tmp_name);
  if (copied <= 0 || copied >= (int)out_path_size) {
    return 0;
  }

  FILE *fp = fopen(out_path, "wb");
  if (fp == NULL) {
    return 0;
  }

  size_t prompt_len = strlen(prompt);
  size_t written = fwrite(prompt, 1, prompt_len, fp);
  fclose(fp);

  return written == prompt_len;
}

static const char *resolve_ollama_command(void) {
  const char *env_cmd = getenv("FALCON_OLLAMA_CMD");
  if (env_cmd != NULL && env_cmd[0] != '\0') {
    return quote_command_if_needed(env_cmd);
  }

#ifdef _WIN32
  const char *local_app_data = getenv("LOCALAPPDATA");
  if (local_app_data != NULL && local_app_data[0] != '\0') {
    static char candidate_path[1024];
    int written = snprintf(candidate_path, sizeof(candidate_path),
                           "%s\\Programs\\Ollama\\ollama.exe", local_app_data);
    if (written > 0 && written < (int)sizeof(candidate_path) &&
        falcon_file_exists(candidate_path)) {
      return quote_command_if_needed(candidate_path);
    }
  }
#endif

  return "ollama";
}

static char *next_exec_result_buffer(void) {
  char *slot = exec_result_buffers[exec_result_slot_index];
  exec_result_slot_index =
      (exec_result_slot_index + 1) % FALCON_EXEC_RESULT_SLOTS;
  slot[0] = '\0';
  return slot;
}

static void run_ollama_stream(const char *model, const char *full_prompt) {
  if (model == NULL || full_prompt == NULL) {
    printf("[Ollama] Error: NULL model or prompt payload\n");
    return;
  }

  char escaped_model[256];
  escape_for_shell(model, escaped_model, sizeof(escaped_model));

  char prompt_path[1024];
  if (!write_prompt_temp_file(full_prompt, prompt_path, sizeof(prompt_path))) {
    printf("[Ollama] Error: Failed to create temp prompt file\n");
    return;
  }

  const char *ollama_cmd = resolve_ollama_command();
  char cmd[4096];
  snprintf(cmd, sizeof(cmd), "%s run \"%s\" < \"%s\"", ollama_cmd,
           escaped_model, prompt_path);

  FILE *pipe = _popen(cmd, "r");
  if (pipe == NULL) {
    printf("[Ollama] Error: Failed to run Ollama\n");
    printf("[Ollama] Hint: set FALCON_OLLAMA_CMD to full executable path if "
           "needed.\n");
    remove(prompt_path);
    return;
  }

  char buffer[256];
  while (fgets(buffer, sizeof(buffer), pipe) != NULL) {
    printf("%s", buffer);
    fflush(stdout);
  }

  int status = _pclose(pipe);
  remove(prompt_path);
  if (status != 0) {
    printf("\n[Ollama] Command exited with code: %d\n", status);
  }
}

/**
 * Interactive chat with Ollama - reads user input and sends with system
 * personality
 *
 * @param model - model name (e.g., "llama3.1:8b")
 * @param personality - system prompt defining AI personality
 */
void falcon_ollama_chat(const char *model, const char *personality) {
  if (model == NULL) {
    printf("[Ollama] Error: NULL model\n");
    return;
  }

  printf("You: ");
  fflush(stdout);

  char user_input[2048];
  if (fgets(user_input, sizeof(user_input), stdin) == NULL) {
    if (feof(stdin)) {
      printf("\n[Adwaith's AI] Input closed. Exiting.\n");
      exit(0);
    }
    return;
  }

  size_t len = strlen(user_input);
  while (len > 0 &&
         (user_input[len - 1] == '\n' || user_input[len - 1] == '\r')) {
    user_input[len - 1] = '\0';
    len--;
  }
  trim_in_place(user_input);

  if (strcmp(user_input, "quit") == 0 || strcmp(user_input, "exit") == 0) {
    printf("\n[Adwaith's AI] Goodbye! bye\n");
    exit(0);
  }

  char full_prompt[8192];
  if (personality != NULL && personality[0] != '\0') {
    snprintf(full_prompt, sizeof(full_prompt), "%s User: %s", personality,
             user_input);
  } else {
    snprintf(full_prompt, sizeof(full_prompt), "%s", user_input);
  }

  printf("\n[AI] ");
  fflush(stdout);
  run_ollama_stream(model, full_prompt);
  printf("\n");
  fflush(stdout);
}

/* ============================================
 * File I/O Functions
 * ============================================ */

long long falcon_file_read(const char *filename, char *buffer,
                           long long max_size) {
  if (filename == NULL || buffer == NULL || max_size <= 0) {
    return -1;
  }

  FILE *fp = fopen(filename, "rb");
  if (fp == NULL) {
    return -1;
  }

  long long bytes_read = (long long)fread(buffer, 1, (size_t)max_size - 1, fp);
  buffer[bytes_read] = '\0'; // Null-terminate
  fclose(fp);

  return bytes_read;
}

long long falcon_file_write(const char *filename, const char *data,
                            long long size) {
  if (filename == NULL || data == NULL) {
    return -1;
  }

  FILE *fp = fopen(filename, "wb");
  if (fp == NULL) {
    return -1;
  }

  long long bytes_written;
  if (size < 0) {
    // Treat as null-terminated string
    bytes_written = (long long)fwrite(data, 1, strlen(data), fp);
  } else {
    bytes_written = (long long)fwrite(data, 1, (size_t)size, fp);
  }

  fclose(fp);
  return bytes_written;
}

long long falcon_file_append(const char *filename, const char *data,
                             long long size) {
  if (filename == NULL || data == NULL) {
    return -1;
  }

  FILE *fp = fopen(filename, "ab");
  if (fp == NULL) {
    return -1;
  }

  long long bytes_written;
  if (size < 0) {
    bytes_written = (long long)fwrite(data, 1, strlen(data), fp);
  } else {
    bytes_written = (long long)fwrite(data, 1, (size_t)size, fp);
  }

  fclose(fp);
  return bytes_written;
}

long long falcon_file_exists(const char *filename) {
  if (filename == NULL) {
    return 0;
  }

  FILE *fp = fopen(filename, "r");
  if (fp != NULL) {
    fclose(fp);
    return 1;
  }
  return 0;
}

long long falcon_file_size(const char *filename) {
  if (filename == NULL) {
    return -1;
  }

  FILE *fp = fopen(filename, "rb");
  if (fp == NULL) {
    return -1;
  }

  fseek(fp, 0, SEEK_END);
  long long size = (long long)ftell(fp);
  fclose(fp);

  return size;
}

// Gaut C runtime helpers used by generated C code.
#ifndef GAUT_RUNTIME_H
#define GAUT_RUNTIME_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

#define GAUT_DEFAULT_ARENA_CAP 65536

typedef struct {
    uint8_t* buf;
    size_t cap;
    size_t off;
} gaut_arena;

typedef struct {
    size_t mark;
} gaut_scope;

typedef struct {
    uint8_t* ptr;
    size_t len;
} gaut_bytes;

gaut_arena gaut_arena_from_buffer(uint8_t* buf, size_t cap);
gaut_scope gaut_scope_enter(gaut_arena* arena);
void gaut_scope_leave(gaut_arena* arena, gaut_scope scope);
void* gaut_arena_alloc(gaut_arena* arena, size_t size);

char* gaut_str_concat_arena(gaut_arena* arena, const char* a, const char* b);
char* gaut_str_concat_heap(const char* a, const char* b);
gaut_bytes gaut_bytes_concat_arena(gaut_arena* arena, const gaut_bytes* a, const gaut_bytes* b);
gaut_bytes gaut_bytes_concat_heap(const gaut_bytes* a, const gaut_bytes* b);
void gaut_print(const char* s);
void gaut_println(const char* s);
char* gaut_read_file(const char* path);
int gaut_write_file(const char* path, const char* data);
gaut_bytes gaut_args(void);

#endif // GAUT_RUNTIME_H

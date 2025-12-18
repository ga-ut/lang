// Minimal C runtime for Gaut-generated programs.
#include "runtime.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int gaut_argc = 0;
static char** gaut_argv = NULL;

gaut_arena gaut_arena_from_buffer(uint8_t* buf, size_t cap) {
    gaut_arena arena = {.buf = buf, .cap = cap, .off = 0};
    return arena;
}

gaut_scope gaut_scope_enter(gaut_arena* arena) {
    gaut_scope scope = {.mark = arena ? arena->off : 0};
    return scope;
}

void gaut_scope_leave(gaut_arena* arena, gaut_scope scope) {
    if (!arena || !arena->buf) {
        return;
    }
    if (scope.mark <= arena->cap) {
        arena->off = scope.mark;
    } else {
        arena->off = arena->cap;
    }
}

void* gaut_arena_alloc(gaut_arena* arena, size_t size) {
    if (!arena || !arena->buf) {
        return NULL;
    }
    if (size > arena->cap || arena->off + size > arena->cap) {
        return NULL;
    }
    void* ptr = arena->buf + arena->off;
    arena->off += size;
    return ptr;
}

static size_t gaut_strlen(const char* s) {
    if (!s) {
        return 0;
    }
    return strlen(s);
}

static void* gaut_alloc_bytes(gaut_arena* arena, size_t size) {
    if (size == 0) {
        return NULL;
    }
    if (arena) {
        void* ptr = gaut_arena_alloc(arena, size);
        if (ptr) {
            return ptr;
        }
    }
    return malloc(size);
}

static char* gaut_str_concat_inner(gaut_arena* arena, const char* a, const char* b) {
    const size_t len_a = gaut_strlen(a);
    const size_t len_b = gaut_strlen(b);
    char* out = (char*)gaut_alloc_bytes(arena, len_a + len_b + 1);
    if (!out) {
        return NULL;
    }
    if (a) {
        memcpy(out, a, len_a);
    }
    if (b) {
        memcpy(out + len_a, b, len_b);
    }
    out[len_a + len_b] = '\0';
    return out;
}

char* gaut_str_concat_arena(gaut_arena* arena, const char* a, const char* b) {
    return gaut_str_concat_inner(arena, a, b);
}

char* gaut_str_concat_heap(const char* a, const char* b) {
    return gaut_str_concat_inner(NULL, a, b);
}

static gaut_bytes gaut_bytes_concat_inner(gaut_arena* arena, const gaut_bytes* a, const gaut_bytes* b) {
    const size_t len_a = a ? a->len : 0;
    const size_t len_b = b ? b->len : 0;
    gaut_bytes out = {.ptr = NULL, .len = len_a + len_b};
    if (out.len == 0) {
        return out;
    }
    out.ptr = (uint8_t*)gaut_alloc_bytes(arena, out.len);
    if (!out.ptr) {
        out.len = 0;
        return out;
    }
    if (a && a->ptr) {
        memcpy(out.ptr, a->ptr, len_a);
    }
    if (b && b->ptr) {
        memcpy(out.ptr + len_a, b->ptr, len_b);
    }
    return out;
}

gaut_bytes gaut_bytes_concat_arena(gaut_arena* arena, const gaut_bytes* a, const gaut_bytes* b) {
    return gaut_bytes_concat_inner(arena, a, b);
}

gaut_bytes gaut_bytes_concat_heap(const gaut_bytes* a, const gaut_bytes* b) {
    return gaut_bytes_concat_inner(NULL, a, b);
}

void gaut_print(const char* s) {
    if (s) {
        fputs(s, stdout);
    }
    fflush(stdout);
}

void gaut_println(const char* s) {
    if (s) {
        fputs(s, stdout);
    }
    fputc('\n', stdout);
    fflush(stdout);
}

char* gaut_read_file(const char* path) {
    if (!path) {
        return NULL;
    }
    FILE* f = fopen(path, "rb");
    if (!f) {
        return NULL;
    }
    if (fseek(f, 0, SEEK_END) != 0) {
        fclose(f);
        return NULL;
    }
    long len = ftell(f);
    if (len < 0) {
        fclose(f);
        return NULL;
    }
    if (fseek(f, 0, SEEK_SET) != 0) {
        fclose(f);
        return NULL;
    }
    char* buf = (char*)malloc((size_t)len + 1);
    if (!buf) {
        fclose(f);
        return NULL;
    }
    size_t read = fread(buf, 1, (size_t)len, f);
    fclose(f);
    buf[read] = '\0';
    return buf;
}

int gaut_write_file(const char* path, const char* data) {
    if (!path || !data) {
        return -1;
    }
    FILE* f = fopen(path, "wb");
    if (!f) {
        return -1;
    }
    size_t len = strlen(data);
    size_t written = fwrite(data, 1, len, f);
    fclose(f);
    return written == len ? 0 : -1;
}

void gaut_args_init(int argc, char** argv) {
    gaut_argc = argc;
    gaut_argv = argv;
}

gaut_bytes gaut_args(void) {
    gaut_bytes out = {.ptr = NULL, .len = 0};
    if (gaut_argc <= 0 || !gaut_argv) {
        return out;
    }
    // Encode argv as UTF-8 bytes joined by '\n' (including argv[0]).
    size_t total = 0;
    for (int i = 0; i < gaut_argc; i++) {
        const char* s = gaut_argv[i] ? gaut_argv[i] : "";
        total += strlen(s);
        if (i + 1 < gaut_argc) {
            total += 1;
        }
    }
    if (total == 0) {
        return out;
    }
    uint8_t* buf = (uint8_t*)malloc(total);
    if (!buf) {
        return out;
    }
    size_t off = 0;
    for (int i = 0; i < gaut_argc; i++) {
        const char* s = gaut_argv[i] ? gaut_argv[i] : "";
        const size_t len = strlen(s);
        if (len > 0) {
            memcpy(buf + off, s, len);
            off += len;
        }
        if (i + 1 < gaut_argc) {
            buf[off++] = (uint8_t)'\n';
        }
    }
    out.ptr = buf;
    out.len = off;
    return out;
}

char* gaut_bytes_to_str(gaut_bytes b) {
    // Best-effort conversion: assume UTF-8 and ensure NUL termination.
    size_t len = b.len;
    char* out = (char*)malloc(len + 1);
    if (!out) {
        return NULL;
    }
    if (len > 0 && b.ptr) {
        memcpy(out, b.ptr, len);
    }
    out[len] = '\0';
    return out;
}

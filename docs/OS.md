# Gaut OS contract

## Current machine

- architecture: AArch64
- reference board: QEMU `virt`
- entry privilege: EL1 from QEMU direct-kernel boot
- RAM: 128 MiB beginning at `0x40000000`
- console: PL011 UART at `0x09000000`
- build input: one checked request packet at `0x47000000`
- network device: absent
- persistent disk: absent

QEMU is a replaceable hardware model, not part of Gaut semantics.

## Current compile-and-run flow

`dist/gaut-os.img` is the readable Gaut compiler built with external profile
`2`. `os/run.sh` places that profile ID, an eight-byte little-endian source
length, and the exact source bytes in the input packet and boots the compiler.

The compiler:

1. copies the packet into its fixed source region;
2. parses and validates Gaut;
3. emits a raw AArch64 child image into its output region; and
4. uses `platform.run(address)` to transfer control to the child entry.

The child acceptance program is `os/examples/compiled.gaut` and must emit:

```text
gaut-os: compiled
```

The guest path contains no Linux, libc, dynamic loader, Python, C, Rust, LLVM,
assembler, linker, or foreign runtime.

## Current hard boundary

The compiler is not yet a resident service:

- `platform.run` performs a non-returning branch;
- the child entry establishes the same fixed stack and Gaut arenas used by the
  compiler;
- a child with fixed memory can overwrite compiler state;
- input is one launch packet rather than a command channel; and
- there is no persistent workspace or fault isolation.

Therefore the current machine truthfully compiles and executes one program per
boot, but cannot safely perform a second edit/build/run cycle.

## Next acceptance contract: resident compiler

The next implementation must keep one compiler alive across child execution:

1. compiler and child receive non-overlapping stack, evaluation, return, and
   fixed-memory regions;
2. the compiler engine receives explicit buffers instead of owning platform
   input, output, and process lifetime in its `main` function;
3. a returnable child entry preserves and restores the compiler continuation;
4. the compiler compiles and runs two child programs without rebooting; and
5. serial output ends with one `gaut-os: ready` marker after both returns.

No editor, file system, scheduler, MMU, or optimizer is added to satisfy this
gate. Serial source input follows only after resident execution is proven.

## Later kernel boundary

After the resident loop, Gaut OS adds explicit exception vectors, EL0 task
isolation, an MMU-backed page allocator, timer interrupts, scheduling,
persistent storage, rendering, networking, and higher-level services in that
order as concrete programs demand them.

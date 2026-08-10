# Gaut OS contract

## Current machine

- architecture: AArch64
- reference board: QEMU `virt`
- entry privilege: EL1 from QEMU direct-kernel boot
- RAM: 128 MiB beginning at `0x40000000`
- console: PL011 UART at `0x09000000`
- build input: one checked request sequence at `0x47000000`
- network device: absent
- persistent disk: absent

QEMU is a replaceable hardware model, not part of Gaut semantics.

## Current compile-and-run flow

`dist/gaut-os.img` is the readable Gaut compiler built with external profile
`2`. `os/run.sh` places consecutive profile `3` child requests in memory and
terminates the sequence with sixteen zero bytes. Each request still contains
its profile ID, eight-byte little-endian source length, and exact source bytes.
The next request begins at the following eight-byte boundary; zero alignment
bytes are sequence framing and are not part of either request.

The resident compiler:

1. copies one request into its fixed source region;
2. parses and validates Gaut;
3. emits a raw AArch64 child image into its output region; and
4. uses `platform.run(address)` to call the child entry;
5. restores its own runtime after the child returns; and
6. repeats until the zero terminator, then emits `gaut-os: ready`.

The child acceptance programs are `os/examples/child1.gaut` and
`os/examples/child2.gaut`. A complete boot must emit exactly:

```text
gaut-os: child 1
gaut-os: child 2
gaut-os: ready
```

The guest path contains no Linux, libc, dynamic loader, Python, C, Rust, LLVM,
assembler, linker, or foreign runtime.

## Resident memory contract

- the profile `2` supervisor owns `0x47f00000` through `0x47ffffff`;
- a profile `3` child owns `0x47e00000` through `0x47efffff`;
- the supervisor output image remains in the supervisor region while executing;
- `platform.run` preserves the supervisor stack, local base, evaluation stack,
  return stack, and request cursor across the child call; and
- one child finishes before the next image replaces the output buffer.

The two regions prevent ordinary Gaut locals and fixed memory in a child from
overwriting compiler state. There is still no MMU, fault containment,
persistent workspace, or malicious-child isolation.

## Next acceptance contract: serial development loop

The next implementation replaces the fixed launch sequence with a checked
serial command channel while preserving the resident compiler and memory
contract. No editor, file system, scheduler, MMU, or optimizer is added before
repeated serial upload, compile, run, result, and recovery are proven.

## Later kernel boundary

After the resident loop, Gaut OS adds explicit exception vectors, EL0 task
isolation, an MMU-backed page allocator, timer interrupts, scheduling,
persistent storage, rendering, networking, and higher-level services in that
order as concrete programs demand them.

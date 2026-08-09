# Gaut

Gaut is a small readable systems language that directly emits AArch64 machine
code, followed by an operating system written in the same language.

The maintained compiler is [`gaut/compiler.gaut`](gaut/compiler.gaut). It has
one runtime value type (`word`), named functions and values, structured
conditions and loops, fixed memory, explicit platform effects, and no hidden
allocation or foreign runtime.

```text
Gaut source -> Gaut compiler -> AArch64 program
```

The compiler directly writes either a static AArch64 Linux bootstrap ELF or a
raw QEMU `virt` image. It does not invoke C, Rust, LLVM, an assembler, a linker,
libc, or a dynamic loader.

## Run inside Gaut OS

`dist/gaut-os.img` is the compiler built as a freestanding program. The launcher
places one readable Gaut source in RAM and boots a networkless AArch64 machine.
Inside that machine Gaut reads, compiles, and executes the program without
Linux:

```sh
./os/run.sh os/examples/compiled.gaut
```

Expected output:

```text
gaut-os: compiled
```

QEMU is a replaceable host-side hardware emulator. The launcher creates only a
temporary source packet and deletes it on exit. The current machine compiles
and runs one program per boot.

## Current files

- `gaut/compiler.gaut`: readable self-hosted compiler
- `gaut/examples/`: language regression programs
- `dist/gaut.elf`: retained hosted AArch64 compiler for fixed-point rebuilds
- `dist/gaut-os.img`: retained freestanding compiler image
- `docs/GAUT-SPEC.md`: language semantics and grammar
- `docs/PRIMITIVES.tsv`: canonical value and memory primitives
- `docs/EFFECTS.tsv`: canonical platform effects
- `docs/OS.md`: current machine and runtime boundary
- `docs/ROADMAP.md`: next implementation gates
- `os/run.sh`: disposable QEMU launcher
- `os/VERIFIED`: current fixed-point and machine verification
- `lab/`: on-demand offline ARM64 rebuild environment for compiler changes

Completed bootstrap implementations, transitional compilers, and phase plans
remain available through Git history but are not part of the current tree.

## Development rule

A compiler change is complete only when the retained native compiler builds
the readable compiler twice to byte-identical generations in the offline ARM64
lab. A freestanding change additionally requires a clean networkless QEMU boot.
Source implementation, fixed point, boot proof, commit, and remote push remain
separate gates.

The immediate next boundary is making the compiler a resident service that can
survive a child program and accept another source without rebooting. See
`docs/ROADMAP.md`.

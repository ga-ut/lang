# Gaut ARM64 language and OS bootstrap

The project now targets a self-hosted systems language followed by a minimal
operating system written in that language. See `docs/LANGUAGE.md`,
`docs/GAUT-SPEC.md`, `docs/OS.md`, and `docs/ROADMAP.md` for the authoritative
boundaries.

M0 below is frozen historical bootstrap machinery. It is not the product
language and will not be expanded into a renamed AArch64 assembler.

## Current result: readable direct compiler

`gaut/compiler.gaut` is a human-readable Gaut 0.4 compiler. It accepts
named values, decimal literals, call-shaped expressions, braces, semicolons,
functions, branches, loops, and fixed-memory declarations. It directly writes
a complete static AArch64 Linux ELF:

```text
Gaut source -> gaut compiler -> AArch64 ELF
```

It does not invoke Python, Core-0, C, Rust, LLVM, an assembler, a linker, libc,
or a dynamic loader. In the isolated offline AArch64 VM, the first direct
compiler rebuilt its F0-capable source through byte-identical generations:

```text
413a2e8af4b6267ec3472df68d4d267174b3ae11fa8d19489a49b494fb075649
```

Use the retained 52 KiB compiler inside the isolated AArch64 Linux VM:

```sh
./gaut.elf < program.gaut > program.elf
chmod +x program.elf
./program.elf
```

Verified readable examples return 42 for arithmetic/functions, 55 for a loop,
42 for fixed memory, and 42 for `if`/`else`. Invalid calls, arity, and nested
early returns are rejected with status 2 and zero artifact bytes.

The same compiler also supports the explicit freestanding QEMU target.

## Current result: Gaut OS F0 direct boot

`os/boot.gaut` begins with `target qemu_virt;`. The compiler emits a 4 KiB raw
AArch64 image instead of a Linux ELF:

```sh
./gaut.elf < boot.gaut > gaut-os.img
```

The image was built twice to identical bytes and booted twice in clean,
networkless QEMU `virt` processes. Its Gaut-written PL011 driver produced:

```text
gaut-os: boot
```

The image contains no Linux, libc, dynamic loader, foreign runtime, assembler,
or linker output. F0 proves direct boot and UART MMIO; running the Gaut compiler
inside Gaut OS is the next F1 gate.

## Frozen Core-0 recovery compiler

`core0/compiler.core` is a compiler written in Core-0. In the offline AArch64
VM, the audit-constructed generation 0 compiled it to generation 1, and
generation 1 compiled it to generation 2. All three files were byte-identical:

```text
be6c623f15602c051fe664289659c1cdd83265506e05f7b5e34d694280fbb293
```

The retained recovery compiler is `dist/core0.elf` (20 KiB). It directly emits
static AArch64 Linux ELF files and needs no Python, C, Rust, LLVM, assembler,
linker, libc, or dynamic loader. `core0/reference_compiler.py` is retained only
to audit the one-time construction.

Verified self-hosted examples:

- function call and arithmetic: exit 42
- structured loop and local slots: exit 55
- explicit arena store/load: exit 42
- unknown operation: rejected with status 2 and no emitted bytes

Core-0 is no longer in the active build path. It remains a dependency-free,
reviewable recovery seed for reconstructing the readable compiler if needed.

## Frozen M0 trust root

M0 v0.1 is a native, self-hosting ARM64 Linux word assembler. It has no C,
Rust, LLVM, assembler, linker, libc, interpreter, or dynamic-loader dependency.
It reads source from standard input and writes a fixed-size native ELF to
standard output.

## Verified bootstrap chain

```text
hex0 -> M0 stage0 -> M0 stage1 -> M0 stage2
```

The two self-hosted M0 generations were byte-identical in the offline ARM64
VM. The retained final compiler is `dist/m0.elf`; redundant stage binaries and
the temporary VM workspace were removed after verification.

## v0.1 source syntax

All integer arguments are hexadecimal. Commands are whitespace separated.

```text
word  d2800540   # emit one 32-bit word, little-endian
byte  7f         # emit one byte
movz  0 2a       # movz x0, #42
svc              # svc #0
jump  3          # emit B with raw signed imm26 value
zero  10         # emit 0x10 zero bytes
```

The current parser identifies commands by their first character, so comments
are not part of v0.1 source. The annotations above are documentation only and
must not be copied into source yet.

M0 emits one deterministic 4096-byte ELF with a 3976-byte code capacity. The
small fixed format keeps the compiler and self-host comparison auditable.

## Example

`dist/exit42.m0`:

```text
movz 0 2a
movz 8 5d
svc
```

Compile inside the ARM64 Linux VM:

```sh
./m0.elf < exit42.m0 > exit42.elf
chmod +x exit42.elf
./exit42.elf
echo $?
```

The verified result is `42`.

## Retained files

- `dist/m0.elf`: final self-hosted compiler
- `dist/m0.m0`: compiler represented in its own source language
- `dist/exit42.m0` and `dist/exit42.elf`: verified example
- `dist/M0_VERIFIED`: isolated-VM verification record
- `dist/hex0.hex`: tiny historical trust-root source
- `bootstrap/` and `m0/build_m0.py`: audit-only seed constructors
- `core0/compiler.core`: self-hosted compiler source
- `dist/core0.elf`: retained self-hosted Core-0 compiler
- `dist/CORE0_VERIFIED`: isolated-VM fixed-point and language-test record
- `core0/reference_compiler.py`: audit-only Core-0 constructor
- `gaut/compiler.gaut`: maintained readable direct compiler source
- `dist/gaut.elf`: retained self-hosted direct Gaut compiler
- `gaut/DIRECT_VERIFIED`: isolated-VM direct fixed-point and test record
- `os/boot.gaut`: first freestanding Gaut OS source
- `dist/gaut-os.img`: retained deterministic QEMU `virt` boot image
- `os/F0_VERIFIED`: direct boot verification record

The Python constructors are not part of the active build path. Keeping them
allows the original hand-encoded AArch64 instructions to be reviewed.

# Core ARM64 language and OS bootstrap

The project now targets a self-hosted systems language followed by a minimal
operating system written in that language. See `docs/LANGUAGE.md`,
`docs/RAW-SPEC.md`, `docs/OS.md`, and `docs/ROADMAP.md` for the authoritative
boundaries.

M0 below is frozen historical bootstrap machinery. It is not the product
language and will not be expanded into a renamed AArch64 assembler.

## Current result: Core-0 self-hosted

`core0/compiler.core` is a compiler written in Core-0. In the offline AArch64
VM, the audit-constructed generation 0 compiled it to generation 1, and
generation 1 compiled it to generation 2. All three files were byte-identical:

```text
be6c623f15602c051fe664289659c1cdd83265506e05f7b5e34d694280fbb293
```

The retained active compiler is `dist/core0.elf` (20 KiB). It directly emits
static AArch64 Linux ELF files and needs no Python, C, Rust, LLVM, assembler,
linker, libc, or dynamic loader. `core0/reference_compiler.py` is retained only
to audit the one-time construction.

Verified self-hosted examples:

- function call and arithmetic: exit 42
- structured loop and local slots: exit 55
- explicit arena store/load: exit 42
- unknown operation: rejected with status 2 and no emitted bytes

Core-0 is the dependency-free recovery language, not the source people should
use to maintain the OS. The next gate is the readable named Raw Core grammar in
`RAW-SPEC.md`, followed by a freestanding backend.

## Current readable self-hosted frontend

`raw0/compiler.raw` now accepts the Raw Core 0.4 grammar with named values,
decimal literals, call-shaped expressions, braces, semicolons, functions,
branches, loops, and fixed-memory declarations. It emits validated Core-0
source, which the retained native Core-0 compiler lowers to AArch64 ELF.

The readable compiler rebuilt its own source through two byte-identical native
generations in the offline VM. The remaining bootstrap boundary is explicit:
the readable compiler still emits Core-0 source, and the frozen Core-0 compiler
performs AArch64 lowering.

Use the retained compiler inside the isolated AArch64 VM:

```sh
./raw0.elf < program.raw > program.core
./core0.elf < program.core > program.elf
chmod +x program.elf
./program.elf
```

`dist/raw0.elf` is the VM-built, readable self-hosted compiler. The next
language gate moves the existing AArch64 lowering into readable Raw Core so the
compiler can emit the final artifact directly.

Compile inside the isolated AArch64 Linux VM:

```sh
./core0.elf < program.core > program.elf
chmod +x program.elf
./program.elf
```

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

The Python constructors are not part of the active build path. Keeping them
allows the original hand-encoded AArch64 instructions to be reviewed.

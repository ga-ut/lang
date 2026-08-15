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
raw QEMU `virt` image. The source does not name either target. A small external
build request selects the output profile, so the same Gaut bytes can be built
for both without changing the language or parser. The compiler does not invoke
C, Rust, LLVM, an assembler, a linker, libc, or a dynamic loader.

```sh
./gaut/request.sh linux program.gaut program.request
./dist/gaut.elf < program.request > program.elf
```

`gaut/request.sh` is only a host-side request framer. Profile `1` means the
current Linux ELF adapter and profile `2` means the current Gaut OS raw-image
adapter. Platform, ABI, board, and container names are not Gaut keywords.
Passing a directory frames every `.gaut` file below it as one named source
unit. A relative path such as `math/vector.gaut` becomes module `math.vector`;
source still contains no `module` or `import` declaration.

## Run inside Gaut OS

`dist/gaut-os.img` is the compiler built as a freestanding program. The launcher
sends readable Gaut sources through the emulated PL011 serial device and boots
a networkless AArch64 machine. Inside that machine one resident Gaut compiler
reads, compiles, and executes each program without Linux or rebooting between
them. With no arguments the launcher sends the two acceptance programs; one to
nine source files or source roots may be supplied explicitly:

```sh
./os/run.sh
```

Expected output:

```text
gaut-os: ready
gaut-os: received 1
gaut-os: child 1
gaut-os: ready
gaut-os: received 2
gaut-os: child 2
gaut-os: ready
```

QEMU is a replaceable host-side hardware emulator. The launcher creates only a
temporary unpadded serial request stream and deletes it on exit. The compiler
and child use separate 1 MiB runtime arenas, and the compiler survives each
child return. After a sixteen-byte zero terminator, Gaut requests PSCI machine
shutdown and QEMU exits by itself; manual interruption is not part of the run
contract.

## Current files

- `gaut/compiler.gaut`: readable self-hosted compiler
- `gaut/request.sh`: external build-profile request framer
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

Adding another platform means adding an external profile and its backend
adapter. It must not add target syntax, reserve a platform name, or create a
second parser.

The launcher still supplies a deliberately bounded session, but the resident
compiler can now wait for delayed serial input through `WFI` without consuming
a host CPU core. A rejected Gaut source emits the canonical compiler diagnostic
and returns to `ready` without rebooting. The immediate next boundary is a
minimal named in-memory source workspace, so upload and execution no longer
have to be the same command. See `docs/ROADMAP.md`.

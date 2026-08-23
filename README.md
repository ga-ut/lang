# Gaut

Gaut is a small readable systems language that directly emits AArch64 machine
code, followed by an operating system written in the same language.

The maintained compiler is the seven readable source units in
[`gaut/compiler/`](gaut/compiler/). It has
one runtime value type (`word`), named functions and values, structured
conditions and loops, fixed memory, explicit platform effects, and no hidden
allocation or foreign runtime.

Unsigned `word` literals use ordinary decimal notation for quantities and a
lowercase `0x` hexadecimal notation for addresses, masks, flags, and instruction
bit patterns. Both spellings produce the same runtime value; hexadecimal
notation adds no type or operator.

```text
Gaut source -> Gaut compiler -> AArch64 program
```

The compiler directly writes either a static AArch64 Linux bootstrap ELF or a
raw QEMU `virt` image. The source does not name either target. A small external
build request selects the output profile, so the same Gaut bytes can be built
for both without changing the language or parser. The compiler does not invoke
C, Rust, LLVM, an assembler, a linker, libc, or a dynamic loader.

Every function is parsed and validated. After calls and local storage resolve,
the compiler retains only `main` and its transitive callees in the final image.
This removes unused profile and module code without adding imports, annotations,
runtime reachability tables, or a second lowering path.

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

Profiles `1` and `2` also supply ordinary `platform` and `verification` source
units from `gaut/adapters/`. `platform.run()`, `platform.ready()`, and the
byte-comparison helpers are therefore Gaut functions, not compiler-reserved
effects. The only lower execution operation is `call_image(address)`, which
calls an image and returns its `main()` word.

Gaut OS also accepts external `store`, `run`, `test`, `fixed-point`,
`workspace-test`, and `workspace-fixed-point` commands. Stored-source metadata
remains in a bounded supervisor catalog while payloads remain in platform
storage and load only when selected. A test request
compiles an ordinary Gaut program and lets Gaut OS judge its
returned `main()` word: zero passes and every nonzero word fails. These commands
are host protocol data, not Gaut keywords. No `assert` syntax or test runtime is
added. A fixed-point request retains one generated compiler, transfers control
to it, and lets that compiler compare the size and every byte of its following
generation before reporting through the same Gaut-native verdict.

The workspace commands carry no source bytes. Gaut OS scans up to 64 named
records from the QEMU `virt` second CFI flash bank into an 8192-byte metadata
catalog, reads one selected payload at a time, reconstructs the same unpadded
multi-unit command `1` request in memory, and uses the existing parser and
compiler path. Profile `3` executes and judges the workspace; profile `2`
performs the same native fixed-point check. Replacement keeps physical order
across shutdown without adding project, file-system, or import syntax.

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
and child use separate 1 MiB runtime arenas, and a third arena can retain one
compiler generation. The compiler survives each child return. After a
sixteen-byte zero terminator, Gaut requests PSCI machine shutdown and QEMU exits
by itself; manual interruption is not part of the run contract.

`os/boot.sh` accepts one complete serial stream. Setting
`GAUT_WORKSPACE_FLASH` attaches or creates one 64 MiB backing file as the second
QEMU CFI flash bank while preserving direct `-kernel` boot:

```sh
GAUT_WORKSPACE_FLASH="$PWD/workspace.flash" ./os/boot.sh requests.stream
```

The file is host transport. Gaut validates slot headers, lengths, exact bytes,
and checksums and decides whether the restored workspace is usable.

`tests/self-host-qemu.sh` sends the compiler request twice in one serial stream.
The first generated compiler is retained in a third machine arena and builds
the second. Gaut compares both images and emits the final verdict; the script
does not use `cmp`, a hash, or artifact extraction to decide success.

`tests/self-host-workspace-qemu.sh` stores the nine compiler, platform, and
verification units once. Two following 24-byte workspace requests rebuild the
compiler from the resident inventory, and Gaut OS judges the following
generations byte-identical without receiving the source bytes again.

`tests/persistent-workspace-qemu.sh` stores, restores, replaces, and corrupts
source slots across separate QEMU boots.
`tests/persistent-self-host-workspace-qemu.sh` stores the compiler sources in
one boot and reaches the Gaut-native fixed point after reboot without
retransmitting them.
`tests/flash-source-catalog.sh` crosses the previous ten-source boundary,
replaces the eleventh record, and checks wrong-index and checksum rejection
across reboot.

`tests/codegen-baseline.sh` records deterministic generated instruction and
evaluation-stack counts for small Gaut programs. `tests/capacity-baseline.sh`
checks the accepted and rejected sides of the current request, module, function,
local, fixed-memory, and output-image bounds. These are implementation and
platform capacities rather than new Gaut syntax; see `docs/PERFORMANCE.md`.
`tests/direct-expression-lowering.sh` builds the current compiler source with
the retained compiler, verifies reduced stack traffic, and lets Gaut OS judge
nested arithmetic, memory, and call results from the generated code.
`tests/hex-literals.sh` verifies decimal/hexadecimal byte equivalence, the full
64-bit range, malformed and overflow rejection, and a following decimal
recovery build.
`tests/local-slot-reuse.sh` verifies that non-overlapping functions reuse fixed
local storage, caller values survive reachable callees, a 513-slot active call
chain and call cycles are rejected, and the resident compiler recovers.
`tests/unreachable-functions.sh` verifies byte-identical clean/dead images,
forward and backward cross-module calls, moved structured control flow, the
256-function reachability boundary, and full validation of unreachable source.
`tests/build-rejection.sh` verifies that the host build adapter accepts only the
documented page-framed artifact and never saves Gaut's rejection diagnostic as
an output file.

## Current files

- `gaut/compiler/`: readable self-hosted compiler source units
- `gaut/adapters/`: profile-selected Gaut platform modules
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
- `os/boot.sh`: serial-stream launcher with optional persistent CFI flash
- `os/build.sh`: QEMU and Gaut OS artifact builder
- `os/VERIFIED`: current fixed-point and machine verification

Completed bootstrap implementations, transitional compilers, and phase plans
remain available through Git history but are not part of the current tree.

## Development rule

A compiler change is complete only when the retained Gaut OS compiler builds
the readable compiler through two following byte-identical generations in
QEMU and Gaut itself judges their sizes and bytes equal. This fixed-point path
contains no Linux guest. A freestanding change additionally requires a clean
networkless QEMU boot.
Source implementation, fixed point, boot proof, commit, and remote push remain
separate gates.

Adding another platform means adding an external profile and its backend
adapter. It must not add target syntax, reserve a platform name, or create a
second parser.

The launcher still supplies a deliberately bounded session, but the resident
compiler can now wait for delayed serial input through `WFI` without consuming
a host CPU core. A rejected Gaut source emits the canonical compiler diagnostic
and returns to `ready` without rebooting. Named sources, replacement order, and
the compiler workspace survive clean shutdown through the profile-supplied CFI
adapter. The emitter now retains one pending expression value in an existing
register and uses the previous evaluation stack only when that bounded contract
is insufficient. The compiler now assigns fixed local ranges from its acyclic
call graph, so non-overlapping functions reuse storage without runtime work. The
bounded catalog now keeps source payloads out of resident RAM and supports the
full 64-unit request inventory. The final image now removes unreachable
functions from profile-selected modules after full validation. The next gate
measures whole-workspace rebuild work before retaining any reusable module
artifact; see `docs/ROADMAP.md` for the exact contract.

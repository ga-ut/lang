# Core language -> Core OS

This is the authoritative project boundary. M0 is frozen as a recoverable seed;
it will not grow into the user-facing language.

## Gate L0 — frozen trust root (complete)

- `hex0 -> M0 -> M0` reaches a byte-identical fixed point.
- M0 is a native AArch64 Linux executable with no dynamic dependencies.
- M0 and its tiny source remain only for audit and disaster recovery.

## Gate L1 — Core-0 semantics (complete)

- The source language contains no AArch64 instruction names or Linux syscall
  numbers.
- It has functions, local values, structured conditions and loops, integer and
  pointer operations, and explicit effects.
- No garbage collector, exceptions, implicit allocation, implicit I/O, or
  ambient authority.
- The bootstrap host is Linux/AArch64, but Linux is not part of the language
  semantics.

The exact v0 contract is in `LANGUAGE.md`.

## Gate L2 — native compiler (complete)

- A one-time, reviewable constructor makes generation 0.
- Generation 0 compiles the compiler source to generation 1.
- Generation 1 compiles the same source to generation 2.
- Generations 1 and 2 are byte-identical.
- The retained active build uses only the Core compiler; the constructor is
  audit material, not an active dependency.

Verified fixed-point SHA-256:
`be6c623f15602c051fe664289659c1cdd83265506e05f7b5e34d694280fbb293`.

## Gate L3 — readable Raw Core and freestanding target (next)

The first L3 sub-gate is complete: `raw0/compiler.core` is a native Core-0
bridge that validates Raw Core 0.4 and emits deterministic Core-0 source.
Readable arithmetic, functions, branches, loops, fixed memory, rejection
tests, and whitespace-insensitive parsing passed in the offline AArch64 VM.
This bridge result is not yet readable-language self-hosting.

- The self-hosted compiler can emit both bootstrap-host ELF and a freestanding
  AArch64 image.
- Target-specific effects live behind explicit target modules.
- A normal program cannot accidentally issue a Linux syscall or access a
  device register.
- Maintained source uses names and parenthesized expressions from
  `RAW-SPEC.md`; numbered slots and exposed evaluation-stack bookkeeping stay
  in recovery Core-0 only.
- There is one runtime `word` type and exactly one lowering path for each
  canonical primitive.
- The bridge compiler is rewritten in readable Raw Core and two successive
  native generations are byte-identical.

## Gate O1 — first Core OS

- Boots on QEMU `virt` as AArch64 without Linux, libc, a dynamic loader, or a
  foreign runtime.
- All executable kernel logic is Core source. The compiler may synthesize the
  image header, reset entry, and exception-vector layout.
- Brings up PL011 serial output, a page allocator, synchronous exception
  reporting, the ARM generic timer, and one cooperative task.
- Repeated clean boots produce the same serial transcript through the final
  acceptance marker.

The exact OS boundary is in `OS.md`.

## Deliberately deferred

- graphical compositor and GPU driver
- browser engine
- networking and package manager
- mobile hardware ports
- AI runtime

Those begin only after L2 and O1 are reproducible. This prevents the browser or
AI design from silently dictating an unstable language or kernel ABI.

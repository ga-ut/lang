# Gaut roadmap

## Current verified boundary

- readable Gaut compiler directly emits AArch64 Linux ELF and QEMU raw images
- one runtime `word` type and one lowering per canonical primitive
- hosted compiler rebuilds itself to a byte-identical fixed point
- freestanding compiler boots without Linux in a networkless AArch64 machine
- one RAM-resident Gaut source is compiled and executed per boot
- output platform is selected by an external build profile, never source syntax

## Next: resident compiler

The current non-returning child transfer and shared arena prevent a second
compile/run cycle. The next gate is deliberately limited to fixing that exact
boundary.

Required implementation:

1. extract one callable compiler engine with explicit source, output, state,
   capacity, and result memory;
2. keep hosted file I/O and freestanding supervision in separate small entry
   adapters around that same engine;
3. assign compiler and child non-overlapping runtime arenas;
4. add one returnable child-image entry and continuation contract;
5. compile and run two children in one boot; and
6. prove that compiler state remains intact after each child returns.

Acceptance transcript:

```text
gaut-os: child 1
gaut-os: child 2
gaut-os: ready
```

## Then

1. serial command channel for repeated source upload;
2. minimal source buffer editing and named in-memory files;
3. explicit persistent workspace;
4. exception reporting and EL0 process isolation;
5. physical pages, MMU mappings, timer, and scheduler;
6. rendering and input;
7. networking and browser runtime;
8. CPU-oriented numerical and AI runtime.

Each gate adds only behavior required by its acceptance program. Syntax sugar,
new types, optimizers, package management, graphics, networking, browser work,
and AI work do not enter the resident-compiler gate.

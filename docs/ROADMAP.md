# Gaut roadmap

## Current verified boundary

- readable Gaut compiler directly emits AArch64 Linux ELF and QEMU raw images
- one runtime `word` type and one lowering per canonical primitive
- hosted compiler rebuilds itself to a byte-identical fixed point
- freestanding compiler boots without Linux in a networkless AArch64 machine
- output platform is selected by an external build profile, never source syntax
- profile `2` keeps one resident compiler alive across profile `3` child calls
- compiler and child own separate 1 MiB runtime arenas
- two large-fixed-memory children compile, run, return, and leave the compiler
  ready in one boot
- completion requests PSCI machine shutdown; the emulator must not survive the
  verified transcript

## Next: serial development loop

The resident compiler currently consumes a fixed sequence placed in RAM before
boot. The next gate changes only that input boundary.

Required implementation:

1. define a framed serial command for source upload and execution;
2. validate length and command before changing the resident source buffer;
3. compile, run, and return using the existing profile `3` child contract;
4. report success or the existing Gaut diagnostic over serial;
5. accept a second command without rebooting; and
6. recover to `gaut-os: ready` after every successful child return.

Acceptance transcript:

```text
gaut-os: ready
gaut-os: received 1
gaut-os: child 1
gaut-os: ready
gaut-os: received 2
gaut-os: child 2
gaut-os: ready
```

## Then

1. exception vectors and checked fault reporting back to the resident loop;
2. minimal named in-memory source buffers;
3. explicit persistent workspace;
4. EL0 process isolation and MMU-backed physical pages;
5. timer and scheduler;
6. rendering and input;
7. networking and browser runtime;
8. CPU-oriented numerical and AI runtime.

Each gate adds only behavior required by its acceptance program. An editor,
file system, optimizer, package manager, graphics, networking, browser work,
and AI work do not enter the serial-loop gate.

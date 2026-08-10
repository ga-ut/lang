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
- the resident compiler receives exact unpadded requests through PL011 serial
- one bounded session accepts consecutive child requests without rebooting
- PL011/GIC wake events and `WFI` let the resident compiler wait for delayed
  input without consuming a host CPU core
- completion requests PSCI machine shutdown; the emulator must not survive the
  verified transcript

## Next: recoverable supervisor errors

The resident compiler can now wait for serial input without polling. The next
gate changes only failure recovery.

Required implementation:

1. validate request length and profile before compilation;
2. report malformed input and compiler failure without shutting down;
3. install exception vectors that report a child fault and restore the
   supervisor continuation;
4. accept a valid request after each rejected request or child fault; and
5. retain the explicit zero-record shutdown path.

Acceptance transcript:

```text
gaut-os: ready
gaut-os: rejected
gaut-os: ready
gaut-os: received 1
gaut-os: child 1
gaut-os: ready
```

The already verified low-power wait must remain active at both ready markers.

## Then

1. minimal named in-memory source buffers;
2. explicit persistent workspace;
3. EL0 process isolation and MMU-backed physical pages;
4. timer and scheduler;
5. rendering and input;
6. networking and browser runtime;
7. CPU-oriented numerical and AI runtime.

Each gate adds only behavior required by its acceptance program. An editor,
file system, optimizer, package manager, graphics, networking, browser work,
and AI work do not enter the recoverable-supervisor gate.

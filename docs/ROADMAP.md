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
- a complete invalid Gaut source emits the canonical diagnostic and restarts
  the resident compiler without rebooting the machine
- a valid request following that rejection compiles, runs, and returns to ready
- completion requests PSCI machine shutdown; the emulator must not survive the
  verified transcript
- one request may contain named source units; filesystem paths become dotted
  module names only in the host adapter
- qualified cross-module calls link without `module` or `import` syntax
- Module v0 is byte-identical across three self-hosted compiler generations
  and runs as a returnable Gaut OS child
- the maintained compiler is split into compiler, emitter, lexer, parser,
  request, storage, and symbol units without changing Gaut syntax
- Gaut OS rebuilds those units through a byte-identical fixed point in QEMU
  without a Linux guest
- external command IDs store and run one named source without adding keywords
- eight fixed source slots support deterministic replacement and survive a
  compile rejection within the same boot

## Next: persistent source storage

The resident named table currently disappears at machine shutdown. The next
gate preserves explicitly stored sources across boots without changing Gaut
syntax or adding hidden allocation.

Required implementation:

1. define one explicit block-storage adapter outside Gaut syntax;
2. retain exact name, length, source bytes, and replacement behavior;
3. recover a complete stored table after a clean reboot;
4. reject torn or malformed stored data without losing the last valid table;
5. keep compilation and execution separate from persistence; and
6. retain the networkless, low-power, and clean-shutdown contracts.

## Later

1. malformed-frame recovery;
2. EL0 process isolation, exception recovery, and MMU-backed physical pages;
3. timer and scheduler;
4. rendering and input;
5. networking and browser runtime;
6. CPU-oriented numerical and AI runtime.

Each gate adds only behavior required by its acceptance program. An editor,
file system, optimizer, package manager, graphics, networking, browser work,
and AI work do not enter the persistent-storage gate.

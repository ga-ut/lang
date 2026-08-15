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

## Next: split the compiler across Module v0

Split `gaut/compiler.gaut` along its existing lexer, symbol, parser, emitter,
and request boundaries. The split must not add `module`, `import`, manifests,
aliases, or automatic dependency search.

Required acceptance:

1. the split source rebuilds to the same compiler behavior;
2. Module v0 and existing language regressions remain green;
3. two following self-host generations are byte-identical; and
4. the freestanding compiler retains the current QEMU transcript and shutdown.

## Then: named in-memory source storage

The resident compiler currently receives source and immediately runs it. The
next gate separates source storage from execution without adding persistence.

Required implementation:

1. add external command IDs for storing and running source; these are protocol
   fields, not Gaut syntax or reserved names;
2. keep a fixed-capacity table of source name, exact byte length, and source
   bytes in supervisor-owned memory;
3. replace an existing name deterministically without an allocator;
4. compile and run only when a named source is requested;
5. retain the stored source after a compile rejection; and
6. retain the low-power ready wait and explicit zero-record shutdown path.

Acceptance transcript:

```text
gaut-os: ready
gaut-os: stored work
gaut-os: ready
gaut-os: child 1
gaut-os: ready
```

The same source name must also be replaceable and runnable again in the same
boot. Names and commands remain host protocol data; they do not enter the Gaut
grammar.

## Later

1. explicit persistent workspace;
2. malformed-frame recovery;
3. EL0 process isolation, exception recovery, and MMU-backed physical pages;
4. timer and scheduler;
5. rendering and input;
6. networking and browser runtime;
7. CPU-oriented numerical and AI runtime.

Each gate adds only behavior required by its acceptance program. An editor,
file system, optimizer, package manager, graphics, networking, browser work,
and AI work do not enter the in-memory-workspace gate.

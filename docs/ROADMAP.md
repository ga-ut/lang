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
- profiles `1` and `2` supply `platform` as ordinary Gaut source instead of
  compiler-reserved `platform.run` or `platform.ready` effects
- the QEMU adapter implements its ready message with Gaut memory operations
  and wraps the single lower `call_image(address)` control transfer
- a profile `3` program can define its own `platform.run()` and resolve it as a
  normal module call
- `call_image` carries the child's returned `main()` word back to the resident
  Gaut caller
- external command `4` uses the ordinary source compiler and child execution
  path without adding `test` or `assert` syntax
- Gaut OS judges the returned `main()` word itself: zero passes and every
  nonzero word fails
- one boot distinguishes pass, fail, and pass again with deterministic result
  records while the host only transports and observes the stream

## Next: Gaut-native fixed-point verifier

Gaut now compiles, runs, and judges ordinary Gaut test programs, but the host
still compares compiler generations with `cmp` and reports their hash. The next
gate moves the equality decision into Gaut without adding syntax or a hash
dependency.

Required implementation:

1. build two following compiler generations inside the resident machine;
2. retain both generated image sizes and byte regions;
3. compare the sizes and every byte with ordinary Gaut memory operations;
4. return zero only when the generations are identical;
5. report the result through the existing Gaut-native test protocol; and
6. retain the networkless, low-power, rejection-recovery, and clean-shutdown
   contracts.

## Later

1. persistent named-source storage across shutdown;
2. malformed-frame recovery;
3. EL0 process isolation, exception recovery, and MMU-backed physical pages;
4. timer and scheduler;
5. rendering and input;
6. networking and browser runtime;
7. CPU-oriented numerical and AI runtime.

Each gate adds only behavior required by its acceptance program. An editor,
file system, optimizer, package manager, graphics, networking, browser work,
and AI work do not enter the native fixed-point gate.

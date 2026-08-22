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
- ten fixed source slots support deterministic replacement and survive a
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
- command `5` retains one generated profile `2` compiler in a separate machine
  arena and transfers control to it without adding language syntax
- that compiler builds the following generation in the ordinary output buffer
  and compares both sizes and every byte with ordinary Gaut memory operations
- equal images pass, while a changed byte or changed size fails through the
  existing Gaut-native test result protocol
- `tests/self-host-qemu.sh` supplies the source records and observes the final
  verdict without using `cmp`, a hash, or host artifact extraction to decide it
- command `6` reconstructs one ordinary multi-unit command `1` request from all
  occupied resident source slots without receiving source bytes
- profile `3` workspace execution distinguishes an initial failing helper from
  its same-name replacement and reports both through Gaut-native verdicts
- the nine compiler, platform, and verification units can be stored once and
  rebuilt through two following 24-byte workspace fixed-point requests
- the retained inventory survives the generated compiler handoff, both
  following generations are byte-identical, and QEMU shuts down cleanly
- deterministic code-generation fixtures record used image bytes, emitted
  instruction words, and evaluation-stack traffic without host timing noise
- capacity fixtures exercise the accepted and rejected sides of request,
  module, function, local, fixed-memory, and output-image limits, then recover
  to a following passing Gaut-native test in the same boot
- profile `2` maps each 32768-byte source slot to one 256 KiB sector in the
  second QEMU CFI flash bank and commits the validated slot header last
- a workspace stored in one boot is restored and executed from a 24-byte
  command `6` request after shutdown without retransmitting source bytes
- same-name replacement survives another reboot, while an invalid slot header
  is ignored and reaches the ordinary compiler rejection path
- the nine-unit compiler workspace stored before shutdown reaches a Gaut-native
  two-generation fixed point after reboot
- the emitter retains one pending constant, local load, or computed result in
  `x0`, lets binary consumers use `x1`, and falls back to the existing
  evaluation stack when a second value must survive
- nested noncommutative arithmetic, ordered memory operations, and user-call
  fallback preserve their Gaut-native results without new syntax or types
- the deterministic binary fixture fell from 298 to 208 instructions and from
  89 to 43 total evaluation-stack movements
- the directly lowered compiler reaches a 73728-byte Gaut-native fixed point
  across two following freestanding generations
- decimal and lowercase `0x` hexadecimal literals produce the same unsigned
  `word` and byte-identical generated programs without a new type or operator
- empty, uppercase, malformed, and greater-than-64-bit hexadecimal forms are
  rejected, after which an ordinary decimal program still compiles and passes
- the decimal-only retained compiler built a bridge lexer, the bridge accepted
  the selectively converted system source, and both notations generated the
  same 77824-byte compiler image
- maintained hardware addresses, masks, flags, ELF fields, and AArch64
  instruction words use hexadecimal notation while quantities remain decimal

## Performance and capacity discipline

The current fixed limits are compiler-table and platform-memory capacities, not
Gaut grammar. Performance work begins from the reproducible records in
`docs/PERFORMANCE.md`. Direct expression lowering removed the first measured
stack traffic. Local lifetime reuse, register allocation, and measured inlining
follow only when an acceptance program justifies them. None requires new Gaut
keywords or types.

The persistent workspace and direct-expression gates changed the baseline
deliberately. Later work must now reduce a measured capacity or cost without
weakening storage, fixed-point, ordered-effect, and recovery contracts.

## Next: local-slot lifetime reuse

The current compiler assigns local slots monotonically across all functions in
one program. Call-cycle rejection makes lifetimes statically bounded, but the
backend does not yet reuse slots whose functions cannot be active together.
The next gate should improve that measured capacity without exposing frames,
slots, or allocation policy in Gaut source.

Required implementation:

1. add a failing multi-function program that exceeds the current global
   512-local total while keeping every individual function within 256 locals;
2. derive reusable slot ranges from the already rejected acyclic call graph;
3. preserve caller locals across every reachable callee and reuse slots only
   between functions that cannot be active together;
4. keep the existing 256-local per-function and fixed-memory boundaries unless
   a separate acceptance program justifies changing them;
5. rebuild through the Gaut-native fixed point and retain direct-expression,
   persistent workspace, boot, rejection, low-power, and shutdown regressions.

## Later

1. malformed-frame recovery;
2. EL0 process isolation, exception recovery, and MMU-backed physical pages;
3. timer and scheduler;
4. rendering and input;
5. networking and browser runtime;
6. CPU-oriented numerical and AI runtime.

Each gate adds only behavior required by its acceptance program. Local-slot
lifetime reuse must not add source types, keywords, aliases, implicit source
allocation, a second parser, or a platform-specific Gaut spelling.

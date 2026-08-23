# Gaut performance and capacity baseline

Gaut keeps performance policy out of source syntax. Optimizer work must not add
types, keywords, aliases, hidden allocation, or a second lowering for a
canonical primitive. A backend change is accepted only when an ordinary Gaut
program demonstrates the need and the retained compiler still reaches the
Gaut-native fixed point.

## Deterministic code-generation baseline

`tests/codegen-baseline.sh` builds four small programs with the retained Gaut OS
compiler. It records the padded image size, the last used image byte, emitted
AArch64 instruction words, and evaluation-stack cursor movements. Counting
instructions and stack movements gives a repeatable first baseline without
using host wall-clock time, which varies with QEMU and host load.

The current Linux profile includes the ordinary `platform` and `verification`
modules in every image. The rows therefore measure complete build artifacts,
not isolated primitive latency.

| program | image bytes | used bytes | instructions | pushes | pops |
| --- | ---: | ---: | ---: | ---: | ---: |
| constant | 4096 | 188 | 17 | 1 | 1 |
| binary | 4096 | 228 | 27 | 2 | 2 |
| call | 4096 | 236 | 29 | 2 | 2 |
| memory | 4096 | 268 | 37 | 4 | 4 |

The persistent workspace gate added two ordinary hosted `platform` stubs to
every Linux-profile image. This deliberately moved each fixture by 88 used
bytes, 22 instructions, two pushes, and two pops. The storage driver itself is
present only in the QEMU adapter. The later unreachable-function gate removes
these and every other function that the selected `main` cannot reach.

Direct expression lowering changed the complete-image baseline as follows:

| program | used bytes removed | instructions removed | pushes removed | pops removed |
| --- | ---: | ---: | ---: | ---: |
| constant | 312 | 78 | 20 | 20 |
| binary | 360 | 90 | 23 | 23 |
| call | 312 | 78 | 20 | 20 |
| memory | 312 | 78 | 20 | 20 |

The emitter defers at most one constant, local load, or computed result. A
consumer can take that value directly from `x0`, moving it to `x1` for the top
binary operand. If another value must be retained, the older value uses the
existing evaluation stack. User calls flush the pending value before the
existing calling convention, and ordered memory and platform effects keep
their source order. No Gaut source syntax or value semantics changed.

These values are characterization results, not performance targets. A backend
optimization should normally reduce one or more values. Any intentional change
updates `tests/fixtures/codegen-baseline/EXPECTED.tsv` only after the generated
programs run correctly and the compiler passes fixed-point verification. Local
slot lifetime reuse is now complete. Later code-generation candidates are
register allocation and measured call-site inlining.

The lowercase hexadecimal-literal lexer gate did not change any of the four
generated-program rows above. It increased the retained freestanding compiler
from 73728 to 77824 padded bytes; the hosted compiler remained within its 69632
byte padding boundary. This is compiler parsing cost, not generated-program
runtime cost. Hardware addresses, masks, flags, ELF fields, and AArch64 words
now use hexadecimal notation while quantities and capacity limits remain
decimal.

The local-slot lifetime gate also left all four generated-program rows
unchanged. Functions first emit relative local slots. After calls resolve, the
compiler topologically assigns one fixed range per activation depth, using the
largest function at that depth. Functions at the same depth cannot reach one
another and share that range; every caller and reachable callee remain disjoint.
This first allocator is deliberately conservative: unrelated functions at
different depths do not yet share a range. A more exact reachability allocator
requires a separate failing program before adding that complexity.
The final AArch64 instructions contain ordinary fixed offsets, so execution adds
no allocator, release, branch, or call overhead. The retained freestanding
compiler remains 77824 padded bytes, while the hosted compiler moved from 69632
to 73728 padded bytes.

The flash-backed source-catalog gate also leaves every generated-program row
unchanged. It preserves the existing one-argument hosted platform stubs, so
ordinary Linux images gain no catalog instructions or stack traffic. Inside the
resident compiler, 327680 bytes previously reserved for ten simultaneous source
payloads became an 8192-byte catalog for up to 64 names. The existing output
region is reused as one 32768-byte record scratch area, so supervisor fixed
memory falls by 319488 bytes without dynamic allocation. Catalog boot scanning
and record checksumming are bounded platform-storage work; child execution gains
no lookup, allocation, or dispatch overhead. The retained freestanding compiler
grew from 77824 to 81920 padded bytes, while the hosted compiler remains within
73728 bytes.

Unreachable-function removal changed every small Linux fixture by the same
platform overhead: 724 used bytes, 181 instructions, 18 pushes, and 21 pops
were removed. All source is still parsed and resolved before removal. The
resolved call graph marks `main` and its transitive callees with a bounded queue;
reachable bodies are copied forward once in source order. Internal relative
branches move with their function, while entry and user-call branches are
patched to the compacted indices. Generated code adds no reachability table,
runtime branch, allocator, or dispatch. The compiler implementation itself adds
one 4096-byte padding page: the retained freestanding image moves from 81920 to
86016 bytes and the hosted compiler from 73728 to 77824 bytes.

`tests/unreachable-functions.sh` proves byte-identical Linux images with an
unused function inserted before both forward and backward cross-module calls.
The moved backward callee includes `while` and `if` control flow, both directions
execute under profile 3, and a 256-function reachable chain exercises the queue
boundary. Unresolved calls and call cycles inside unreachable functions still
reject before a following passing request.

## Capacity baseline

`tests/capacity-baseline.sh` verifies both sides of the current fixed bounds:

- a 131072-byte request is accepted and the next byte is rejected;
- 64 named source units are accepted and 65 are rejected by request framing;
- 256 functions are accepted and 257 are rejected by the Gaut compiler;
- a two-function active call chain can consume 256 locals in each function,
  while one more active slot is rejected;
- three mutually non-overlapping functions can declare 600 total locals and
  reuse a 200-slot range;
- 917504 fixed-memory bytes are accepted and 917505 are rejected; and
- 5039 retained nested-expression statements fit below the 262144-byte rounded
  image boundary, while the 5040th is rejected.

The last boundary remains an emission-workspace limit: the current one-pass
compiler lowers every validated function before post-validation compaction.
Unreachable functions reduce the retained artifact but still consume temporary
output space while the request is checked. Removing that transient cost requires
a separate staged or retained-module acceptance program rather than weakening
whole-source validation.

Every guest-side rejection is followed by a valid Gaut test in the same boot.
This proves that a capacity failure does not create a second recovery path or
prevent clean shutdown.

These numbers describe the current compiler tables, request framing, output
buffer, and 1 MiB runtime arena. They are not Gaut grammar or value semantics.
Increasing them requires a platform memory-layout change and the same boundary,
isolation, fixed-point, and boot tests; it does not require new source syntax.

Host-side C output is not a conformance dependency. A later comparative suite
may record C compiler results for engineering decisions, but Gaut's retained
build and pass/fail path must continue to work without C, LLVM, an assembler,
or a linker.

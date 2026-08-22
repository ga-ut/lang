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
| constant | 4096 | 912 | 198 | 19 | 22 |
| binary | 4096 | 952 | 208 | 20 | 23 |
| call | 4096 | 960 | 210 | 20 | 23 |
| memory | 4096 | 992 | 218 | 22 | 25 |

The persistent workspace gate added two ordinary hosted `platform` stubs to
every Linux-profile image. This deliberately moved each fixture by 88 used
bytes, 22 instructions, two pushes, and two pops. The storage driver itself is
present only in the QEMU adapter. A later whole-program dead-function pass may
remove unused adapter functions, but the retained dependency-free compiler does
not yet perform that optimization.

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
programs run correctly and the compiler passes fixed-point verification. Later
candidates are local-slot lifetime reuse, register allocation, and measured
call-site inlining.

The lowercase hexadecimal-literal lexer gate did not change any of the four
generated-program rows above. It increased the retained freestanding compiler
from 73728 to 77824 padded bytes; the hosted compiler remained within its 69632
byte padding boundary. This is compiler parsing cost, not generated-program
runtime cost. Hardware addresses, masks, flags, ELF fields, and AArch64 words
now use hexadecimal notation while quantities and capacity limits remain
decimal.

## Capacity baseline

`tests/capacity-baseline.sh` verifies both sides of the current fixed bounds:

- a 131072-byte request is accepted and the next byte is rejected;
- 64 named source units are accepted and 65 are rejected by request framing;
- 256 functions are accepted and 257 are rejected by the Gaut compiler;
- two functions can consume 256 locals each, while a 513th program slot is
  rejected;
- 917504 fixed-memory bytes are accepted and 917505 are rejected; and
- 5039 retained nested-expression statements fit below the 262144-byte rounded
  image boundary, while the 5040th is rejected.

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

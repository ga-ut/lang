# Raw Core specification - draft 0.1

Raw Core is the one semantic layer shared by the compiler, future kernel, and
later frontends. It is deliberately small, unsafe, and direct.

## 1. One behavior, one primitive

Each observable raw behavior has exactly one primitive ID, one spelling, one
semantic definition, and one lowering function per target.

- No aliases or overloads.
- No hidden macro expansion or compiler-known library helpers.
- No alternative fast path outside the same primitive lowering.
- Derived behavior is ordinary Raw Core source.
- Each target has one adapter implementation for each supported target effect.

## 2. Not built yet

- signed integers
- separate boolean, byte, pointer, slice, array, structure, enum, or string
  types
- ownership, garbage collection, exceptions, or hidden bounds checks
- infix precedence, implicit conversions, generics, macros, optimizers, or
  multiple calling conventions

A feature is added only after a compiler or OS program demonstrates that the
existing primitives cannot express a required behavior clearly and correctly.

## 3. Readability is a semantic requirement

The maintained compiler and OS source must expose names and tree structure. A
reader must not have to simulate the data stack or decode numbered slots to
understand an assignment or call.

Postfix Core-0 remains the frozen recovery representation. It is not the
maintained source language for the OS.

Raw Core uses one canonical parenthesized form. Parentheses are structure, not
optional sugar: there is no alternative infix notation, precedence, shorthand,
or implicit call syntax.

```text
(fn sum (limit)
  (let total 0x0)
  (let i 0x1)
  (while (lt i limit)
    (set total (add total i))
    (set i (add i 0x1)))
  (return total))
```

This source has the same minimal machine model as Core-0, but names replace
numbered slots and every operation's arguments are visible at its use site.

## 4. Lexical format

Source is ASCII. Other input bytes are rejected. Space, tab, LF, and CR
separate tokens. A `#` begins a comment through LF or end of file.

- A literal is `0x` followed by one through sixteen hexadecimal digits.
- A name starts with an ASCII letter or underscore and continues with ASCII
  letters, digits, underscore, or dot.
- Parentheses are the only structural punctuation.

Wider or malformed literals are rejected rather than truncated. There are no
decimal literals, strings, escapes, implicit terminators, or alternative
spellings.

## 5. The only runtime type

```text
word = unsigned 64-bit value
```

Every parameter, local, return value, address, comparison result, and loaded
value is one `word`.

- Arithmetic wraps modulo 2^64.
- Comparisons return exactly zero or one.
- Zero is false; every nonzero word is true.
- Addresses are words interpreted as byte addresses by memory primitives.
- Address validity and alignment are programmer obligations.
- Allocation and conversion never happen implicitly.

Raw Core is intentionally unsafe. A new type is proposed only with a concrete
compiler or kernel failure that it prevents.

## 6. Canonical grammar

The grammar below is the entire maintained source surface for draft 0.1.

```text
program   := function+
function  := (fn name (name*) statement* (return expression))

statement := (let name expression)
           | (set name expression)
           | (drop expression)
           | (store8 expression expression)
           | (store32 expression expression)
           | (store64 expression expression)
           | (if expression (then statement*) (else statement*))
           | (while expression statement*)
           | target-statement

expression := literal
            | name
            | (call name expression*)
            | (primitive expression*)
            | target-expression
```

`let` introduces one named local initialized by its expression. `set` changes
an existing local. Parameters and locals share one namespace; shadowing and
redeclaration are errors. Reading before `let` is impossible because names are
resolved in source order.

Every function returns exactly one word through its final `return`. There is no
implicit result, early return, void function, optional else, or implicit call.
These constraints can be relaxed only when kernel source demonstrates a need.

Function arguments and primitive arguments evaluate left to right exactly
once. Functions must be defined before calls, calls must match parameter count,
and recursion is not part of draft 0.1.

## 7. Canonical value primitives

Each name below has one arity, one meaning, and one primitive ID.
`PRIMITIVES.tsv` is the machine-readable authoritative inventory. Compiler
dispatch, lowering coverage, and conformance coverage must be checked against
that file; a backend must not maintain a second handwritten inventory.

```text
add a b      wrapping addition
sub a b      wrapping subtraction
mul a b      low 64 bits of multiplication
bitand a b   bitwise AND
bitor a b    bitwise OR
bitxor a b   bitwise XOR
shl a n      shift left by (n bitand 63)
shr a n      unsigned shift right by (n bitand 63)
eq a b       one when equal, otherwise zero
lt a b       one when a is unsigned-less-than b, otherwise zero
```

There is no primitive division, remainder, not-equal, greater-than,
less-or-equal, or greater-or-equal. Such behavior begins as an ordinary Raw
Core function composed from the canonical primitives. The compiler never
recognizes that helper as special.

There are no source-visible `dup`, `swap`, `over`, local-slot, register, or
stack operations. Evaluation storage is an implementation detail, not work the
human author must simulate.

## 8. Canonical memory primitives

```text
load8 address
load32 address
load64 address
store8 value address
store32 value address
store64 value address
```

The three widths are different observable behaviors, not aliases. They are
already justified by source parsing, 32-bit instruction/MMIO emission, and
64-bit machine values.

Every Raw Core memory access is ordered and observable. A compiler must not
remove, duplicate, merge, or move it across another memory or target effect.
The same primitive therefore serves RAM and device access through one lowering
path. Reorderable ordinary memory is not introduced yet.

Stores keep the low 8, 32, or 64 bits. Loads zero-extend 8- and 32-bit values.
Misaligned or invalid addresses violate the Raw Core program contract and may
produce a target fault; no hidden check is inserted.

## 9. Control and function behavior

`if` evaluates its condition once and executes exactly one explicit branch.
`while` evaluates its condition before every iteration. There are no labels,
numeric branches, fallthrough cases, or second control-flow representation.

Each function invocation receives fresh storage for its parameters and locals.
The backend has one calling convention. Stack underflow cannot be a source
concept because the maintained language has expression arity; arity mismatch
is rejected before emission.

`main` has no parameters and returns one word as the bootstrap-host status. A
freestanding target supplies a separate, explicit entry adapter.

## 10. Platform boundary

Platform effects are outside the canonical value primitive set and are legal
only for the selected platform. Trap numbers and device addresses are backend
data, never implicit source behavior.

The freestanding platform adds only an effect demanded by the next boot test.
It does not predefine UART, timer, task, file, network, rendering, or AI APIs.

## 11. Determinism and failure

Identical source bytes, compiler version, and platform ID must produce
identical artifact bytes.

- Output contains no timestamps, machine paths, locale, or randomness.
- Unknown forms, wrong arity, malformed names, duplicate bindings, bad literal
  width, and unmatched parentheses are compile errors.
- The whole source is validated before an artifact is written.
- Failure emits zero artifact bytes and returns a nonzero status.
- Diagnostics use ASCII on a separate error channel.

## 12. Single lowering path

The pipeline is fixed:

```text
source form -> primitive ID -> validated operands -> platform lowering
```

There is no second source-to-instruction route. Compiler, kernel, and user
programs use the same primitive inventory and lowering. Container headers,
entry setup, and image metadata remain separate platform construction.

Each platform owns exactly one lowering function for each supported primitive.
An unsupported primitive is a compile error. Optimizations are absent in draft
0.1.

## 13. Conformance gate

A compiler conforms only after all of these pass:

1. one semantic vector per primitive, including boundary values;
2. parser and arity rejection vectors with zero artifact bytes;
3. an inventory proving one spelling and one lowering per primitive ID;
4. repeated-build artifact equality;
5. self-host generations N and N+1 byte equality;
6. the same primitive vectors on every new platform;
7. human review showing maintained source uses names rather than numbered slots
   or exposed evaluation-stack bookkeeping.

## 14. Migration from verified Core-0

Core-0 remains the recovery compiler. Draft 0.1 is not yet claimed implemented.
The next compiler is written in Core-0 but accepts the readable grammar above.

Migration order:

1. parse balanced forms, names, and hexadecimal literals;
2. resolve functions, parameters, and locals and validate arity;
3. map every canonical primitive to one stable ID;
4. lower each ID through one AArch64 implementation;
5. self-host the readable compiler to a new byte-identical fixed point;
6. add freestanding image construction without changing primitive meaning;
7. write the first OS in readable Raw Core.

No larger type system or syntactic sugar is required for the first OS. New
surface area requires a concrete failing program and a spec revision.

# Gaut specification - draft 0.4

Gaut is the one semantic layer shared by the compiler, future kernel, and
later frontends. It is deliberately small, unsafe, and direct.

## 1. One behavior, one primitive

Each observable raw behavior has exactly one primitive ID, one spelling, one
semantic definition, and one lowering function per target.

- No aliases or overloads.
- No hidden macro expansion or compiler-known library helpers.
- No alternative fast path outside the same primitive lowering.
- Derived behavior is ordinary Gaut source.
- Each build profile has one adapter implementation for each supported
  platform effect.

## 2. Not built yet

- signed integers
- source-level type declarations
- separate boolean, byte, pointer, slice, array, structure, enum, or string
  value types
- ownership, garbage collection, exceptions, or hidden bounds checks
- infix precedence, implicit conversions, generics, macros, optimizers, or
  multiple calling conventions

A feature is added only after a compiler or OS program demonstrates that the
existing primitives cannot express a required behavior clearly and correctly.

## 3. Readability is a semantic requirement

The maintained compiler and OS source must expose names and tree structure. A
reader must not have to simulate the data stack or decode numbered slots to
understand an assignment or call.

Gaut uses explicitly terminated statements, braced blocks, and call-shaped
expressions. Whitespace, including line breaks, never changes program meaning.
There is no infix expression parser, operator precedence, indentation
semantics, shorthand, or implicit call syntax.

```text
fn sum(limit) {
  let total = 0;
  let i = 1;

  while lt(i, limit) {
    total = add(total, i);
    i = add(i, 1);
  }

  return total;
}
```

Names replace numbered storage and every operation's arguments are visible at
its use site.

## 4. Lexical format

Source is ASCII. Other input bytes are rejected. Space, tab, LF, and CR are
equivalent whitespace and separate tokens. A `#` begins a comment through LF
or end of file.

- A literal is one or more decimal digits from `0` through `9`.
- A name starts with an ASCII letter or underscore and continues with ASCII
  letters, digits, underscore, or dot.
- Punctuation is limited to `(`, `)`, `{`, `}`, `,`, `;`, and `=`.

Whitespace is ignored outside tokens; indentation and line breaks have no
meaning. A value above the maximum 64-bit unsigned word or any malformed
literal is rejected rather than truncated. There are no hexadecimal literals,
strings, escapes, or alternative numeric spellings.

## 5. The only runtime type

```text
word = unsigned 64-bit value
```

Every parameter, local, return value, address, comparison result, and loaded
value is one `word`. Source contains no type annotations because there is no
second value type to select.

- Arithmetic wraps modulo 2^64.
- Comparisons return exactly zero or one.
- Zero is false; every nonzero word is true.
- Addresses are words interpreted as byte addresses by memory primitives.
- Address validity and alignment are programmer obligations.
- Allocation and conversion never happen implicitly.

The compiler still distinguishes name categories at compile time. A value
name, fixed-memory name, function name, primitive name, and target-effect name
cannot be redeclared or used in a position that requires another category.
This is name and arity validation, not a source-visible type system.

Gaut is intentionally unsafe. A new type is proposed only with a concrete
compiler or kernel failure that it prevents.

## 6. Canonical grammar and small-parser boundary

The grammar below is the entire maintained source surface for draft 0.4.

```text
program    := function+
function   := "fn" name "(" parameters? ")" function-body
parameters := name ("," name)*
function-body := "{" statement* "return" expression ";" "}"
block      := "{" statement* "}"

statement  := "let" name "=" expression ";"
            | name "=" expression ";"
            | "memory" name literal ";"
            | "drop" expression ";"
            | call ";"
            | "if" expression block ("else" block)?
            | "while" expression block

expression := literal
            | name
            | call
call       := name "(" arguments? ")"
arguments  := expression ("," expression)*
```

The lexer has ten token kinds: name, decimal literal, left/right parenthesis,
left/right brace, comma, equals, semicolon, and end-of-file. Keywords are
ordinary names interpreted by statement position. Expression parsing needs one
token of lookahead and three cases: literal, name, or call. There is no Pratt
parser, precedence table, newline state, or backtracking.

`let` introduces one initialized local. Plain assignment changes an existing
local. Parameters and locals share one namespace; shadowing, redeclaration, and
assignment before declaration are errors.

Every function ends with exactly one `return` and returns one word. `return`
is not a general statement and therefore cannot occur inside `if`, `while`, or
another nested block. Early return, void functions, and recursion are outside
draft 0.4. Functions must be defined before calls and calls must match
parameter count.

Arguments evaluate left to right exactly once. A call whose inventory entry
has zero outputs is legal only as a `call ";"` statement and cannot appear
inside an expression. A call with one output must be consumed by an expression,
assignment, return, condition, or explicit `drop`; it is an error to use it as
a bare statement.

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
less-or-equal, or greater-or-equal. Such behavior begins as an ordinary Gaut
function composed from the canonical primitives. The compiler never
recognizes that helper as special.

There are no source-visible `dup`, `swap`, `over`, local-slot, register, or
stack operations. Evaluation storage is an implementation detail, not work the
human author must simulate.

## 8. Canonical memory primitives

`memory name literal;` declares one fixed byte region. Its size is a positive
decimal compile-time constant. The compiler aligns the region's base to eight
bytes and decides its physical placement; no allocator is called at runtime.

- The name evaluates to the region's base byte address but cannot be assigned.
- The declaration is visible from its source position to the end of the
  function, including later nested blocks; there is no shadowing.
- Each function invocation has a fresh logical region. Its bytes are
  uninitialized, and reading a byte not written during that invocation
  violates the program contract.
- The address becomes invalid when the invocation returns. Retaining or using
  it afterward violates the program contract.
- A declaration inside a repeated or conditional block still denotes the one
  statically allocated region for that function invocation. Entering the block
  does not allocate or initialize it.
- A zero size, a region larger than the target limit, or total fixed storage
  beyond the target limit is a compile error.

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

Every Gaut memory access is ordered and observable. A compiler must not
remove, duplicate, merge, or move it across another memory or target effect.
The same primitive therefore serves RAM and device access through one lowering
path. Reorderable ordinary memory is not introduced yet.

Stores keep the low 8, 32, or 64 bits. Loads zero-extend 8- and 32-bit values.
Misaligned or invalid addresses violate the Gaut program contract and may
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

Platform selection is not Gaut syntax. `target`, platform names, ABI names,
container names, and board names are never keywords or canonical source names.
Identical source bytes may be submitted with different external build profiles
without changing the source or parser.

The compiler consumes one binary build request:

```text
offset  size  meaning
0       8     little-endian build profile ID
8       8     little-endian source byte length
16      N     exact Gaut source bytes
```

Profile `1` selects the current AArch64 Linux static ELF adapter. Profile `2`
selects the current AArch64 QEMU virt boot-image adapter. Profile `3` selects
the returnable Gaut OS child-image adapter. Profile `3` is an execution request
for the resident compiler, not a standalone boot image. These IDs belong to
the compiler request protocol, not the language inventory. The source length
is at most 131055 bytes and one request contains no trailing bytes.

A build profile selects the architecture backend, ABI, container, entry
adapter, and available platform-effect lowering. Adding a new platform changes
that external profile table and its backend adapter; it must not change the
Gaut grammar or reserve a source name.

Profile selection does not grant an implicit UART operation. Gaut board source
performs device access through the same ordered `load32` and `store32`
primitive implementations used for ordinary explicit addresses. Board
addresses belong in board-specific source and must not leak into portable
programs.

Target effects retain one spelling and arity across targets. Their adapters
may differ only where the selected machine boundary requires it:

- `host.read(descriptor, address, count)` reads platform input bytes and
  returns the number copied. On bootstrap Linux it is the direct descriptor
  read. In the QEMU `virt` compiler image, the machine has one input channel:
  a complete build request at physical address `0x47000000`. The descriptor is
  accepted for source compatibility but is not interpreted, and the request
  length must not exceed `count`.
- `host.write(descriptor, address, count)` writes platform output bytes and
  returns the number written. On bootstrap Linux it is the direct descriptor
  write. Under profile `2`, the machine has one polled PL011 serial output
  channel, so every descriptor maps to that channel.
- `host.exit(status)` terminates the bootstrap process or enters the
  freestanding target's wait loop. It does not return.
- `platform.run(address)` transfers control to the entry word at `address`
  from the profile `2` resident supervisor. It preserves the supervisor
  continuation and runtime registers, and the profile `3` child returns to
  that continuation. The bootstrap-host adapter consumes the address and
  returns so the same compiler source remains self-hostable. Profile `3`
  source cannot invoke `platform.run`; nested child execution is not part of
  this contract.

The complete effect IDs and arities are in `EFFECTS.tsv`. These are ordered,
observable effects, not value primitives. The current freestanding adapter
permits one RAM input packet and one generated-image transfer per boot; it is
not a file system or an interactive command protocol.

## 11. Determinism and failure

Identical source bytes, compiler version, and build profile ID must produce
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
0.4.

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

## 14. Current implementation boundary

`gaut/compiler.gaut` implements this grammar and directly emits a static
AArch64 Linux ELF or an explicit QEMU `virt` raw image selected by the external
build profile. It has rebuilt itself through byte-identical native generations.
The current freestanding adapter compiles and transfers control to one child
per boot.

The next implementation boundary is a returnable child ABI with non-overlapping
compiler and child memory. It does not require a larger type system or syntax
sugar. New language surface still requires a concrete failing program and a
spec revision.

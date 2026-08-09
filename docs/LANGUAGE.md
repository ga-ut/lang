# Core-0 language contract

`Core` is a provisional project name. Renaming it later does not change this
contract.

## Philosophy: the visible machine

Core makes cost and authority visible in source. A plain function cannot
allocate, perform I/O, enter privileged mode, or touch device memory unless the
required object is passed to it explicitly. The compiler may optimize work but
may not invent an observable effect.

Core is not an AArch64 assembler with renamed instructions. Its source meaning
is independent of registers, instruction encodings, ELF, and QEMU. A backend
chooses those details.

## Implemented Core-0 bootstrap surface

The self-hosted bootstrap surface is intentionally small and unsafe:

```text
value       wrapping u64, with 0/1 comparison results
binding     128 explicit local slots
functions   fn, call, endfn
control     if/else/endif, while/do/endwhile
integer     + - * / & | ^ << >> and unsigned comparisons
memory      load8/32/64, store8/32/64, one explicit arena
effects     host.read, host.write, host.exit bootstrap adapter
```

Core-0 uses postfix bootstrap syntax because a small program can compile it.
It remains the recovery representation, not the maintained OS source.

## Readable Raw Core layer

The compiler and OS that people maintain use the named, parenthesized grammar
in `RAW-SPEC.md`. Names and visible expression trees are required semantic
structure, not optional syntactic sugar.

Raw Core has only one runtime type:

```text
word = unsigned 64-bit value
```

Parameters, locals, returns, comparison results, addresses, and loaded values
are all `word`. Memory width belongs to the operation. A new type requires a
concrete failing compiler or kernel example; none is reserved in advance.

## Memory model

- Words have 64-bit width and arithmetic wraps modulo 2^64.
- Raw addresses are words and arithmetic is measured in bytes.
- Address validity and alignment are programmer obligations.
- Raw memory effects are always ordered and observable, including device
  access. Reorderable memory is not introduced yet.
- Allocation is never implicit. Early programs receive an explicit linear
  arena; the OS later supplies page and region allocators.
- There is no garbage collector or implicit lifetime mechanism.

## Effect boundary

Bootstrap-host I/O and kernel device I/O are different modules:

```text
host.read(input, bytes)       # Linux bootstrap adapter only
host.write(output, bytes)
uart.write(device, bytes)     # Core OS target
timer.arm(device, deadline)
```

There is no general `syscall(number, ...)` in user Core. Raw trap and system
register operations are compiler intrinsics restricted to target modules.

## Calling and binary rules

- The language ABI is defined by values and effects, not AArch64 registers.
- The first compiler uses a simple stack IR and deterministic code emission.
- Source order, map iteration, timestamps, host paths, and random values may
  not affect output bytes.
- Diagnostics go to a separate output and never modify a successful artifact.
- Invalid source must fail closed; it must not emit a runnable partial image.

## Language completion test

Core is considered self-hosted only when all are true:

1. The compiler source is Core source, not generated assembly.
2. A retained native compiler builds that source without Python, C, Rust,
   LLVM, an assembler, or a linker.
3. The produced compiler builds the same source again.
4. The last two native compiler files are byte-identical.
5. A clean offline AArch64 VM reproduces the fixed point and runs language
   tests for functions, branches, loops, memory, and rejected invalid input.

The one-time constructor remains reviewable evidence but is not an active
dependency after this gate passes.

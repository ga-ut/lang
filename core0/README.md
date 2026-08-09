# Core-0 executable semantics

This directory is the transition from the frozen M0 seed to the real language.
`reference_compiler.py` directly encodes ELF and AArch64 for audit. It does not
invoke C, Rust, LLVM, an assembler, or a linker, but Python still makes it a
constructor rather than the finished compiler.

Core-0's temporary bootstrap spelling is postfix:

```text
fn main
  $0 v0!
  while v0@ $a < do
    v0@ $1 + v0!
  endwhile
  v0@
endfn
```

- hexadecimal literals begin with `$`
- `v0@` loads local slot 0 and `v0!` stores it
- functions must be defined before they are called
- `if/else/endif` and `while/do/endwhile` are structured
- `mem` returns the explicit linear arena
- memory and host I/O are explicit operations

`compiler.core` now builds itself to a byte-identical fixed point. The retained
active compiler is `../dist/core0.elf`; the Python constructor is outside the
active build path. See `../dist/CORE0_VERIFIED` for the isolated-VM record.

Core-0 is deliberately an unsafe bootstrap language. The next language layer
adds readable named values, fixed-memory declarations, and explicit target
effects while preserving this compiler as the small recovery base. Raw Core
0.4 has one runtime `word` value and therefore no source-level type syntax.

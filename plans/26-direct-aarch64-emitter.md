# Direct Raw Core AArch64 emitter

## Goal

Make the maintained readable Raw Core compiler emit a complete static AArch64
Linux ELF directly. Remove Core-0 text and `dist/core0.elf` from the active
compile path while retaining them as recovery artifacts.

## Gates

1. Keep the Raw Core 0.4 grammar and primitive inventory unchanged.
2. Replace Core-0 text emission with one direct AArch64 lowering for every
   existing value, memory, control, call, and bootstrap-host operation.
3. Use the previously verified two-stage compiler only once to construct the
   first direct compiler.
4. In an offline AArch64 VM, have the direct compiler build its own source for
   two further generations and require byte-identical ELF output.
5. Compile and run the retained language examples, and verify invalid programs
   fail without partial output.
6. Retain only source, the final compiler, checksums, and concise verification;
   remove VM runtime, generated compiler generations, examples, and caches.

## Direct output contract

- Output is one page-aligned static `ET_EXEC` ELF for AArch64 Linux.
- The file contains its ELF header, one executable load segment, and directly
  encoded AArch64 instructions. It has no dynamic loader, library, assembler,
  linker, C, Rust, LLVM, or Core-0 runtime dependency.
- The generated program owns one explicit 1 MiB arena. Named values use fixed
  compile-time slots; declared memories use fixed offsets in that arena.
- Structured source control is lowered to patched relative AArch64 branches.
- Bootstrap host effects remain Linux system calls and are not promoted into
  the canonical language primitive inventory.

## Boundaries

- This completes the hosted direct-emission compiler gate, not the
  freestanding OS target or device boot gate.
- `core0/compiler.core` and `dist/core0.elf` remain recovery seeds and audit
  evidence, not the normal build path.
- No remote push without explicit authorization.

## Status

- Gate 1: complete
- Gate 2: complete
- Gate 3: complete
- Gate 4: complete
- Gate 5: complete
- Gate 6: complete

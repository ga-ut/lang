# Readable Raw Core implementation

## Goal

Replace maintained postfix Core-0 source with the readable Raw Core 0.4
grammar while retaining Core-0 as the dependency-free recovery compiler.

## Gates

1. Freeze the 0.4 grammar, single-`word` value model, fixed-memory lifetime,
   call arities, and bootstrap-host effect boundary.
2. Implement a Core-0 bridge that validates Raw Core and emits deterministic
   Core-0 source without writing partial output on failure.
3. Verify readable examples through `raw0 -> Core-0 -> AArch64 ELF` in the
   offline VM.
4. Rewrite the bridge compiler in readable Raw Core.
5. Build the readable compiler through two native generations and require
   byte-identical results.
6. Retain source, the final compiler, checksums, and concise verification;
   remove intermediate sources, binaries, VM workspaces, and caches.

## Boundaries

- No C, Rust, LLVM, assembler, linker, libc, or dynamic loader in the active
  build path.
- Python remains audit-only and cannot establish a completion claim.
- Source implementation, bridge verification, readable self-hosting,
  freestanding emission, OS boot, commit, and push are separate gates.
- No remote push without explicit authorization.

## Status

- Gate 1: complete
- Gate 2: complete
- Gate 3: complete
- Gate 4: complete
- Gate 5: complete
- Gate 6: complete

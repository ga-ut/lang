# Gaut repository guidelines

## Project boundary

This repository builds a dependency-free AArch64 language and then an OS in
that language. The authoritative documents are:

- `docs/GAUT-SPEC.md`: readable maintained language
- `docs/PRIMITIVES.tsv`: single canonical primitive inventory
- `docs/LANGUAGE.md`: language philosophy and bootstrap boundary
- `docs/OS.md`: first OS acceptance contract
- `docs/ROADMAP.md`: completion gates

M0 and postfix Core-0 are frozen recovery layers. Maintained compiler and OS
source must use the readable named Gaut grammar once its compiler exists.

## Dependency boundary

- The active compiler path must not require Rust, C, LLVM, an assembler, a
  linker, libc, or a dynamic loader.
- Python files are audit constructors only. Never make them the normal build
  path or describe their output alone as self-hosting evidence.
- A language change is complete only after the retained native compiler builds
  the compiler source twice to byte-identical generations in the offline
  AArch64 VM.
- Keep platform effects separate from canonical language primitives.

## Language discipline

- One observable raw behavior has one primitive ID, one canonical spelling,
  and one lowering implementation per platform.
- Do not add aliases, overloads, implicit conversions, hidden allocation,
  macros, optimizer paths, or new types without a concrete failing compiler or
  kernel program.
- Human readability is mandatory for maintained source: use names and visible
  expression structure; do not expose numbered slots, registers, instruction
  words, or evaluation-stack bookkeeping.
- Update `docs/GAUT-SPEC.md` and `docs/PRIMITIVES.tsv` before implementing a new
  primitive.

## Verification

- Verify `dist/SHA256SUMS` after copying or changing retained artifacts.
- Keep source, the final compiler, concise verification records, and checksum
  manifests. Do not commit VM runtime images, work disks, compiler generations,
  generated examples, caches, traces, or crash dumps.
- Report source implementation, fixed-point verification, freestanding build,
  emulator boot, commit, and push as separate gates.

## Planning and commits

- Record destructive migrations and multi-gate work in `plans/` before acting.
- Preserve unrelated user changes and inspect Git status before committing.
- Use focused imperative commit messages. Do not push, tag remote state, or
  rewrite history without explicit user authorization.

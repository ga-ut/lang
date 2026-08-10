# Gaut repository guidelines

## Project boundary

This repository builds the dependency-free AArch64 Gaut language and Gaut OS.
The authoritative documents are:

- `docs/GAUT-SPEC.md`: readable maintained language
- `docs/PRIMITIVES.tsv`: single canonical primitive inventory
- `docs/EFFECTS.tsv`: single canonical platform-effect inventory
- `docs/OS.md`: first OS acceptance contract
- `docs/ROADMAP.md`: current boundary and next gates

Maintained compiler and OS source use only readable named Gaut. Earlier
bootstrap layers live in Git history, not in the current tree.

## Dependency boundary

- The active compiler path must not require Rust, C, LLVM, an assembler, a
  linker, libc, or a dynamic loader.
- A language change is complete only after the retained native compiler builds
  the compiler source twice to byte-identical generations in the offline
  AArch64 VM.
- Keep platform effects separate from canonical language primitives.
- Select architecture, ABI, container, board, and effect adapter through the
  external build request. Adding a platform must not change Gaut grammar or
  reserve a source name.

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
- Verify at least one identical source through every supported build profile.
- For resident execution, verify two profile `3` children return in one boot
  and that each can touch the end of its fixed-memory arena without damaging
  compiler state.
- Verify that the resident compiler receives requests through serial rather
  than a preloaded RAM packet and emits the exact documented transcript.
- For a serial-wait change, delay input after the first ready marker and verify
  both negligible QEMU CPU use and successful wake-up before accepting it.
- For resident compiler recovery, send an invalid Gaut source followed by a
  valid source in one boot and verify diagnostic, ready, execution, and clean
  shutdown order exactly.
- A QEMU verification is complete only when the guest requests shutdown, the
  emulator exits with status zero, and no Gaut QEMU process remains.
- Keep source, the final compiler, concise verification records, and checksum
  manifests. Do not commit VM runtime images, work disks, compiler generations,
  generated examples, caches, traces, or crash dumps.
- Report source implementation, fixed-point verification, freestanding build,
  emulator boot, commit, and push as separate gates.

## Planning and commits

- Record destructive migrations in a focused commit before acting.
- Preserve unrelated user changes and inspect Git status before committing.
- Use focused imperative commit messages. Do not push, tag remote state, or
  rewrite history without explicit user authorization.

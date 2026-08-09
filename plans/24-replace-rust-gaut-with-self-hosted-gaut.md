# Replace old Rust/C Gaut with self-hosted Gaut

Status: completed locally after user confirmation

## Why this is a replacement

The current repository is one coupled implementation: Rust crates parse and
interpret Gaut, generate C, link a C runtime, and support the existing `.gaut`
compiler and standard library. Removing only `*.rs` and Cargo files would leave
the remaining compiler, runtime, examples, plans, and documentation unusable
and would mix two incompatible language directions.

The replacement therefore removes the old implementation as one unit and
imports the verified dependency-free Core-0 bootstrap plus the readable Gaut
specification.

## Recovery boundary

- Current pre-replacement HEAD: `f7d1e97`.
- `main` is four commits ahead of `origin/main`; those commits already remain
  in Git history.
- Before deletion, create a local recovery tag at the current HEAD.
- Do not rewrite or squash existing history.
- Do not push the replacement until the local tree, checksums, and commit have
  been reviewed separately.

## Remove

- Cargo workspace files and all Rust crates
- the C generator runtime
- old `.gaut` compiler, standard library, and examples
- scripts coupled to Cargo, C generation, or clang
- old language specification and implementation plans
- generated `target/` contents from the local worktree
- obsolete repository instructions tied to Rust/C Gaut

The existing `LICENSE` is retained unless a later provenance review requires a
change.

## Import

```text
README.md                  project state and bootstrap instructions
AGENTS.md                  Gaut repository rules
docs/LANGUAGE.md           language philosophy and current boundary
docs/GAUT-SPEC.md           readable minimal Gaut draft
docs/PRIMITIVES.tsv        single authoritative primitive inventory
docs/OS.md                 first OS acceptance contract
docs/ROADMAP.md            Core-to-OS gates
bootstrap/                 auditable hex0 constructor and verifier
m0/                        frozen M0 recovery source and constructor
core0/compiler.core        self-hosted recovery compiler source
core0/reference_compiler.py audit-only direct encoder
core0/examples/            source-only conformance examples
dist/core0.elf             retained 20 KiB self-hosted ARM64 Linux compiler
dist/m0.elf                retained frozen M0 compiler
dist/*                     sources, checksums, and verification records
lab/                       Apple Virtualization launcher source and setup only
```

Large or generated resources are excluded:

- Rust `target/`
- VM kernel, initramfs, launcher binary, and workspace image
- intermediate compiler generations
- generated example ELF files
- caches and logs

## Verification before commit

1. Confirm no tracked Rust, Cargo, C-runtime, or old `.gaut` implementation
   files remain.
2. Verify every retained distribution checksum.
3. Rebuild the audit Core-0 seed in a disposable directory and compare it with
   `dist/core0.elf` without invoking C, Rust, LLVM, an assembler, or a linker.
4. Confirm the imported Core-0 verification record still matches the retained
   compiler and compiler source.
5. Confirm the readable spec and primitive inventory have unique primitive IDs
   and canonical names.
6. Report deletion, local commit, push, and remote state as separate gates.

## Intended commit sequence

1. `Document Gaut repository replacement`
2. `Replace Rust Gaut with self-hosted Gaut`

The second commit is made only after user confirmation of this plan.

## Result

- Recovery tag: `archive/rust-gaut-before-self-hosted-gaut` at `f7d1e97`.
- Existing Rust, Cargo, C runtime, and old `.gaut` implementation files: zero.
- Retained distribution checksums: all passed.
- Audit constructor output: byte-identical to `dist/core0.elf`.
- Primitive inventory: 24 unique IDs and 24 unique canonical names.
- VM runtime and generated workspace: absent from the repository.
- Remote push: deliberately not performed as part of the replacement commit.

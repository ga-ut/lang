# Name the self-hosted language Gaut

## Goal

Retire the temporary bootstrap codename and make Gaut the single maintained name
for the language, compiler, source files, examples, specification, and direct
AArch64 artifact.

## Canonical names

- Language: Gaut 0.4
- Maintained source extension: `.gaut`
- Compiler source: `gaut/compiler.gaut`
- Native compiler: `dist/gaut.elf`
- Language specification: `docs/GAUT-SPEC.md`
- Hosted compile path: `program.gaut -> gaut.elf -> AArch64 Linux ELF`

Core-0 and M0 keep their existing names because they are distinct frozen
recovery languages, not Gaut components.

## Gates

1. Move active source, examples, specification, verification records, and the
   retained compiler to their Gaut paths.
2. Replace the temporary naming throughout maintained and historical project
   documentation without changing language semantics or primitive IDs.
3. In the isolated offline AArch64 VM, use `gaut.elf` to compile
   `compiler.gaut` for two direct generations and require byte equality.
4. Compile and run all renamed valid examples and rerun rejection tests.
5. Verify the retained manifest, remove VM runtime and intermediate artifacts,
   and commit the migration locally.

## Boundaries

- This is a naming migration, not a grammar, ABI, primitive, or lowering
  change. The compiler binary is expected to remain byte-identical because
  comments and filenames are not program semantics.
- Gaut is self-hosted for the AArch64 Linux target. Freestanding image output,
  OS boot, and later optimization work remain separate gates.
- No remote push without explicit authorization.

## Status

- Gate 1: complete
- Gate 2: complete
- Gate 3: complete
- Gate 4: complete
- Gate 5: complete

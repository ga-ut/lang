# Prune Gaut current tree

## Goal

Move completed bootstrap stages, phase-labelled artifacts, duplicate documents,
and finished plans out of the current checkout while preserving all of them in
Git history.

## Keep

- readable Gaut compiler source and regression programs
- retained hosted and freestanding compiler artifacts
- canonical language, primitive, effect, OS, and roadmap documents
- current compile-and-run example, launcher, checksums, and verification
- disposable ARM64 lab source until Gaut OS can persist a rebuilt compiler

## Remove or rename

- remove the M0 and Core-0 bootstrap trees, constructors, artifacts, and records
- remove completed plans and transition verification records
- remove the superseded direct-boot image, source, and F0 record
- remove duplicate component READMEs and the obsolete Core-0 language document
- rename phase-labelled F1 launcher, image, effect inventory, and verification
  to current Gaut names
- rewrite surviving documents to describe only the current system and future
  work

## Verification

1. retained artifact checksums pass
2. executable launcher passes shell syntax validation
3. no current file references removed paths or phase names
4. the worktree contains only the intended current tree
5. no history rewrite or remote push occurs

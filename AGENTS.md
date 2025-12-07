# Repository Guidelines

This guide explains how to work on the Gaut language workspace and contribute safe, reviewable changes.

## Project Structure & Module Organization
- Workspace root: `Cargo.toml` with crates `frontend` (parser/typechecker), `runtime` (arena + std stubs), `interp` (evaluator), `cgen` (C emitter), `cli` (binary wrapper).
- Language assets: `examples/*.gaut` for fixtures, `std/` for standard modules, `docs/lang-spec.md` for the current spec.
- Utility scripts: `scripts/run_examples.sh` for interp + cgen smoke runs.

## Build, Test, and Development Commands
- Format and lint: `cargo fmt` then `cargo clippy --all-targets --all-features -- -D warnings`.
- Full test sweep: `cargo test` (workspace); scoped checks like `cargo test -p interp` or `cargo test -p cgen`.
- Example smoke run: `./scripts/run_examples.sh` to exercise key .gaut programs and C generation.
- CLI usage during dev: `cargo run -p cli -- examples/hello.gaut`; release binary: `cargo build -p cli --release`.
- Std path override when running binaries: `GAUT_STD_DIR=/path/to/std gaut file.gaut`.

## Coding Style & Naming Conventions
- Rust style: `rustfmt` defaults (4-space indent), `#![forbid(unsafe_code)]` enforced across crates; prefer explicit types over inference when readability helps.
- Naming: `snake_case` for functions/vars/modules, `UpperCamelCase` for types, `SCREAMING_SNAKE_CASE` for consts. Keep `.gaut` modules one-per-file (`import foo` maps to `foo.gaut`).
- Error handling: favor descriptive enums/results; avoid `unwrap` in non-test code unless guarded.

## Testing Guidelines
- Add Rust tests near the code they cover (e.g., parser/typechecker in `crates/frontend`, interpreter flows in `crates/interp`).
- When adding `.gaut` samples, place them under `examples/` and hook them into interpreter tests or `scripts/run_examples.sh`.
- For C generation changes, extend tests in `crates/cgen` and confirm output via `cargo test -p cgen`.
- Capture regressions around move semantics and block lifetimes; prefer `--nocapture` locally for debugging.

## Commit & Pull Request Guidelines
- Commits: short imperative summaries (e.g., `Add move checker coverage`); group related changes by crate or feature.
- PRs: describe intent + approach, list touched crates, include `cargo fmt`, `cargo clippy`, and `cargo test` results (or scoped commands) and note any new examples/std additions. Link related issues/plan items; attach small before/after snippets or CLI outputs when behavior changes.

## Security & Configuration Tips
- Keep contributions free of `unsafe`; preserve arena/borrow invariants noted in `docs/lang-spec.md`.
- Network/runtime pieces in `std/net.gaut` and `runtime/src/net.rs` are stubs—flag any behavior changes and avoid claiming full IO support.
- Do not commit build artifacts under `target/`; prefer reproducible commands over local patches.

# Contributing

## Scope

`neco-parser` is a crates.io-oriented monorepo for parser-only crates (KDL / JSON / future syntax-to-AST converters). The repository was extracted from `barineco/neco-crates` so that parser crates can release independently of the broader monorepo. Small, focused changes are preferred over broad speculative rewrites.

## Independent release policy

- Each member crate is published to crates.io independently.
- Version bumps are per-crate; do not couple version numbers across members unless an API change forces it.
- Public API of an existing crate must remain backward-compatible within a major version. Breaking changes require a new major version of that crate alone (other members unaffected).

## Zero-dependency direction

- Member crates aim for zero external dependencies (only `std` / `core` where possible).
- `no_std`-friendly designs are preferred when the parser nature allows.
- Adding an external dependency requires explicit justification in the PR description.

## Development checks

Before opening a pull request, run from the workspace root:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Pull requests

- Keep changes narrowly scoped and technically justified.
- Update the crate-level README when public behavior changes.
- Avoid introducing silent fallbacks at public API boundaries.
- Prefer adding tests for bug fixes and new public behavior.

## Workspace notes

- Crates in this repository must remain publishable independently.
- Path dependencies must include a version fallback when they point to another workspace crate (so each crate is publishable on its own).
- Public-facing metadata in each `Cargo.toml` should remain suitable for crates.io.
- `repository` field of each `Cargo.toml` points to `https://github.com/barineco/neco-parser`.

# Contributing

## Scope

`neco-parser` is a crates.io-oriented monorepo for JSON and KDL parser crates.
Changes should preserve the format's source meaning and keep one parsing path for each grammar.

## Independent release policy

- Each member crate is published to crates.io independently.
- Version bumps are per-crate; version numbers stay independent across members unless an API change forces coupling.
- Public APIs follow Semantic Versioning. A breaking change increments the crate's version independently.

## Zero-dependency direction

- Member crates aim for zero external dependencies and may use components distributed with Rust:
  - `core`
  - `alloc`
  - `std`
- `no_std`-friendly designs are preferred when the nature of the parser permits.
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
- The `repository` field in each `Cargo.toml` must point to the monorepo origin.
- The origin is `https://github.com/barineco/neco-parser`.

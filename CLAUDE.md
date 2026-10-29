# Repository conventions

## Workspace layout

- `crates/qix-core` (`qix_core`) contains all game rules and state. It must
  remain independent of egui and other user-interface code.
- `crates/qix` (`qix`) is the eframe desktop application. It translates input,
  advances the core, and renders game state.
- Put gameplay tuning values in named constants under `qix_core::tuning`; do
  not scatter magic numbers through the implementation.

The core advances on a fixed timestep. Time is accumulated by the application,
not read by the core. Supply randomness through an injected RNG so simulations
and tests remain deterministic.

## Code organization

- Keep files under 500 lines and functions under 50 lines. Split code into
  focused modules or helpers before exceeding those limits.
- Keep tests out of implementation files. Tests for `<module>.rs` belong in a
  sibling `<module>_test.rs`, declared by the parent module:

  ```rust
  mod board;

  #[cfg(test)]
  mod board_test;
  ```

- Never define an inline `#[cfg(test)] mod tests { ... }` block.

## Required checks

Run these commands before committing:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace --no-tests=warn
cargo test --workspace --doc
```

Drop `--no-tests=warn` once the workspace contains tests.

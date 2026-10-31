# Qix

A Rust and egui clone of the classic arcade game Qix. Claim the playfield by
drawing lines through open space while avoiding the Qix and border-travelling
Sparx. Slow drawing earns more points but leaves the player exposed longer.

## Controls

| Key | Action |
| --- | --- |
| Arrow keys | Move |
| Z | Draw quickly |
| X | Draw slowly |
| Enter | Start or confirm |
| Escape | Pause |
| F1 | Toggle the debug overlay (claimed percentage, lives, tick count) |

## Build and run

Install a current stable Rust toolchain, then build the workspace:

```sh
cargo build --workspace
```

Run the desktop game with:

```sh
cargo run -p qix
```

Repository development and test conventions are documented in
[`CLAUDE.md`](CLAUDE.md).

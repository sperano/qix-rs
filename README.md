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
| Enter | Start a game, the next level, or leave the game-over screen; after a game over with a high score, go to the initials entry and confirm each letter there |
| Arrow keys (initials entry) | Up/Down change the letter, Left/Right move between letters |
| Escape | Pause or resume |
| F1 | Toggle the debug overlay (claimed percentage, lives, tick count) |
| F2 | On the title screen, open the settings (claim threshold, starting lives, key bindings, window scale) |

Left idle for ten seconds, the title screen plays a short demo; any game
control stops it, and Enter also starts a game.

## Build and run

Install a current stable Rust toolchain, then build the workspace:

```sh
cargo build --workspace
```

Run the desktop game with:

```sh
cargo run -p qix
```

The game is silent by default. Build it with the `audio` feature for sound
effects (drawing, claims, deaths, a completed level and extra lives),
played through the default audio output; on Linux it needs the ALSA
development files (`libasound2-dev` on Debian and Ubuntu):

```sh
cargo run -p qix --features audio
```

Without an audio device the game prints a warning and plays silently.

Repository development and test conventions are documented in
[`CLAUDE.md`](CLAUDE.md).

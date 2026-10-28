# infinite-wealth-no-flashbang

Removes the full-screen white flashes from Like a Dragon: Infinite Wealth cutscenes,
including the three flashes per Sujimon in every gacha pull.

- Short white flashes (10 frames or less) are removed.
- Longer white fades in scene transitions fade to black instead.
- Only the cutscene scripts change.

## Usage

Close the game, then:

```
cargo run --release              # patch (finds the game in your Steam libraries)
cargo run --release -- scan      # show what would change, change nothing
cargo run --release -- restore   # put the original cutscenes back
```

Works on Windows and Linux / Steam Deck. If the game isn't found, pass
`--game-dir <folder>` (the folder that contains `runtime`).

The original cutscene scripts (~80 MB) are saved to
`runtime/media/no_flashbang_backup`, and `restore` rebuilds the original files
byte for byte. A game update or Steam's "Verify integrity of game files" also
restores the originals; run the patcher again afterwards.

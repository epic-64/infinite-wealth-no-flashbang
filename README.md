# infinite-wealth-no-flashbang

Removes the full-screen white flashes from Like a Dragon: Infinite Wealth cutscenes,
including the three flashes per Sujimon in every gacha pull.

- Short white flashes (10 frames or less) are removed.
- Longer white fades in scene transitions fade to black instead.
- Only the cutscene scripts change.

## Install

Download the latest release for your system from
[Releases](https://github.com/epic-64/infinite-wealth-no-flashbang/releases), extract it
and close the game. The patcher finds the game in your Steam libraries by itself.

**Windows**: double-click `no-flashbang.exe`.

**Linux / Steam Deck**: in a terminal, in the extracted folder:

```bash
./no-flashbang
```

To undo, run `restore.cmd` (Windows) or `./no-flashbang restore` (Linux).
`no-flashbang scan` shows what would change without changing anything.
If the game isn't found, pass its folder (the one that contains `runtime`) with
`--game-dir <folder>`.

The original cutscene scripts (~80 MB) are saved to
`runtime/media/no_flashbang_backup`, and restoring rebuilds the original files byte
for byte. A game update or Steam's "Verify integrity of game files" also restores the
originals; run the patcher again afterwards.

## Development

With [Rust](https://rustup.rs) installed:

```
cargo run --release -- scan      # or patch, restore
```

### Releasing

Commit everything, then:

```bash
scripts/release.sh --bump patch     # or minor, major, or an explicit version like 1.2.3
```

It raises the version in `VERSION`, `Cargo.toml` and `Cargo.lock`, commits and pushes
that, then tags the commit `v<version>` and pushes the tag. GitHub Actions
(`.github/workflows/release.yml`) builds a static Windows `.exe` and a static Linux
binary, packs them with `scripts/distribute.sh` and publishes
`no-flashbang-<version>-windows.zip` and `no-flashbang-<version>-linux.tar.gz` as a
GitHub release. Without `--bump` the script only tags the current, already pushed
version. No binaries are committed to the repository.

## License

[MIT](LICENSE). This is an unofficial mod, not affiliated with RGG Studio or SEGA.

# infinite-wealth-no-flashbang

Removes the full-screen white flashes from Like a Dragon: Infinite Wealth cutscenes,
including the three flashes per Sujimon in every gacha pull.

- Short white flashes (10 frames or less) are removed
- Longer white fades in scene transitions fade to black instead

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

After a game update or Steam's "Verify integrity of game files", run the patcher again.

Want to build it yourself or help out? See [CONTRIBUTING.md](https://github.com/epic-64/infinite-wealth-no-flashbang/blob/main/CONTRIBUTING.md).

## License

[MIT](LICENSE). This is an unofficial mod, not affiliated with RGG Studio or SEGA.

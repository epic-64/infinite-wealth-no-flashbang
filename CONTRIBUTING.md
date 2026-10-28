# Contributing

## Development

With [Rust](https://rustup.rs) installed:

```
cargo run --release -- scan      # or patch, restore
```

## Releasing

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

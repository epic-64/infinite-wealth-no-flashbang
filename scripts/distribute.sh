#!/usr/bin/env bash
# Packs built binaries into player downloads in dist/:
#   no-flashbang-<VERSION>-windows.zip     no-flashbang.exe, restore.cmd, README, LICENSE
#   no-flashbang-<VERSION>-linux.tar.gz    no-flashbang, README, LICENSE
# Nothing is built here; GitHub Actions builds the binaries and passes them in.
#
# Usage: scripts/distribute.sh [--windows FILE.exe] [--linux FILE]
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

windows=""
linux=""
while [ $# -gt 0 ]; do
    case "$1" in
        --windows) windows="${2:?--windows needs a file}"; shift 2 ;;
        --linux) linux="${2:?--linux needs a file}"; shift 2 ;;
        -h|--help) sed -n '2,7p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "Unknown option: $1" >&2; exit 2 ;;
    esac
done
if [ -z "$windows" ] && [ -z "$linux" ]; then
    echo "Pass --windows and/or --linux" >&2
    exit 2
fi

version="$(tr -d '[:space:]' < VERSION)"
cargo_version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
if [ "$version" != "$cargo_version" ]; then
    echo "VERSION ($version) and Cargo.toml ($cargo_version) disagree" >&2
    exit 1
fi

name="no-flashbang-$version"
stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT
mkdir -p dist

# Folder contents shared by both downloads.
stage_common() {
    local dir="$stage/$1/$name"
    mkdir -p "$dir"
    cp README.md LICENSE "$dir/"
    echo "$dir"
}

if [ -n "$windows" ]; then
    dir="$(stage_common windows)"
    cp "$windows" "$dir/no-flashbang.exe"
    printf '@echo off\r\n"%%~dp0no-flashbang.exe" restore\r\npause\r\n' > "$dir/restore.cmd"
    rm -f "dist/$name-windows.zip"
    (cd "$stage/windows" && zip -qr "$root/dist/$name-windows.zip" "$name")
    echo "Wrote dist/$name-windows.zip"
fi

if [ -n "$linux" ]; then
    dir="$(stage_common linux)"
    cp "$linux" "$dir/no-flashbang"
    chmod +x "$dir/no-flashbang"
    tar -czf "dist/$name-linux.tar.gz" -C "$stage/linux" "$name"
    echo "Wrote dist/$name-linux.tar.gz"
fi

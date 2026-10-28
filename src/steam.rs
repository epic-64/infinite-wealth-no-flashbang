//! Finds the game in the user's Steam libraries (Windows, Linux, Steam Deck).

use std::path::{Path, PathBuf};

const GAME_FOLDER: &str = "LikeADragon8";

/// Steam's own install folders on this machine.
fn steam_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if cfg!(windows) {
        // Read the registry through `reg`, so no extra dependencies are needed.
        for (key, value) in [
            (r"HKCU\Software\Valve\Steam", "SteamPath"),
            (r"HKLM\SOFTWARE\WOW6432Node\Valve\Steam", "InstallPath"),
            (r"HKLM\SOFTWARE\Valve\Steam", "InstallPath"),
        ] {
            let Ok(out) = std::process::Command::new("reg").args(["query", key, "/v", value]).output() else {
                continue;
            };
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                if let Some((_, path)) = line.split_once("REG_SZ") {
                    roots.push(PathBuf::from(path.trim()));
                }
            }
        }
        roots.push(PathBuf::from(r"C:\Program Files (x86)\Steam"));
    } else if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        for rel in [
            ".local/share/Steam",
            ".steam/steam",
            ".steam/root",
            ".var/app/com.valvesoftware.Steam/.local/share/Steam",
        ] {
            roots.push(home.join(rel));
        }
    }
    roots
}

/// Library folders listed in a `libraryfolders.vdf`.
fn vdf_libraries(vdf: &Path) -> Vec<PathBuf> {
    let Ok(text) = std::fs::read_to_string(vdf) else { return Vec::new() };
    text.lines()
        .filter_map(|line| {
            let mut parts = line.trim().split('"').filter(|s| !s.trim().is_empty());
            if parts.next()? != "path" {
                return None;
            }
            Some(PathBuf::from(parts.next()?.replace(r"\\", r"\")))
        })
        .collect()
}

pub fn is_game_dir(dir: &Path) -> bool {
    dir.join("runtime").join("media").join("data").join("hact_elvis").is_dir()
}

/// The game's install folder, if it is in any Steam library.
pub fn find_game_dir() -> Option<PathBuf> {
    let mut libraries = Vec::new();
    for root in steam_roots() {
        let vdf = root.join("steamapps").join("libraryfolders.vdf");
        libraries.push(root);
        libraries.extend(vdf_libraries(&vdf));
    }
    libraries
        .into_iter()
        .map(|lib| lib.join("steamapps").join("common").join(GAME_FOLDER))
        .find(|dir| is_game_dir(dir))
}

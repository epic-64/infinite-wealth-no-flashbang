//! Removes white screen flashes from Like a Dragon: Infinite Wealth cutscenes.
//!
//! Usage: no-flashbang [patch|restore|scan] [--game-dir DIR]
//! The game folder is found automatically in the Steam libraries if not given.

mod cmn;
mod par;
mod steam;

use std::fs;
use std::path::{Path, PathBuf};

const CUTSCENE_DIRS: [&str; 2] = ["hact_elvis", "auth"];
const BACKUP_DIR: &str = "no_flashbang_backup";

/// Backups hold only the original script files (cmn*.bin) of each archive,
/// packed into a small PAR. Textures, models and animations are never modified.
const BACKUP_EXT: &str = "scripts.par";

struct Patched {
    bytes: Vec<u8>,
    /// Original script files, packed for the backup.
    backup: Vec<u8>,
    stats: cmn::Stats,
}

/// cmn.bin plus its per-language copies (cmn_vo.bin, cmn_zh.bin, ...).
fn is_script(f: &par::File) -> bool {
    let name = par::name_str(&f.name);
    name.starts_with("cmn") && name.ends_with(".bin")
}

/// Pack the original script files of a cmn.par into a standalone PAR.
fn pack_backup(inner: &par::Par) -> Vec<u8> {
    let files: Vec<par::File> = inner.files.iter().filter(|f| is_script(f)).cloned().collect();
    let mut root = inner.folders[0].clone();
    root.sub_count = 0;
    root.sub_first = 0;
    root.file_count = files.len() as u32;
    root.file_first = 0;
    par::Par { header: inner.header, big_endian: inner.big_endian, folders: vec![root], files }.write()
}

/// Patch one cutscene archive. Returns None if it has nothing bright to fix.
fn patch_archive(bytes: &[u8]) -> Result<Option<Patched>, String> {
    let mut outer = par::Par::parse(bytes)?;
    let Some(ci) = outer.find("cmn.par") else { return Ok(None) };
    let mut inner = par::Par::parse(&outer.data(ci)?)?;
    let backup = pack_backup(&inner);
    let mut total = cmn::Stats::default();
    let mut changed = false;
    for i in 0..inner.files.len() {
        if !is_script(&inner.files[i]) {
            continue;
        }
        let mut d = inner.data(i)?;
        let s = cmn::patch(&mut d)?;
        if s.removed + s.darkened > 0 {
            if par::name_str(&inner.files[i].name) == "cmn.bin" {
                total = s;
            }
            inner.set_data(i, d);
            changed = true;
        }
    }
    if !changed {
        return Ok(None);
    }
    outer.set_data(ci, inner.write());
    let out = outer.write();

    // Sanity check: the rebuilt archive must parse, contain no bright fades,
    // and keep every non-script file byte-for-byte.
    let check = par::Par::parse(&out)?;
    let ci2 = check.find("cmn.par").ok_or("rebuilt archive lost cmn.par")?;
    let inner2 = par::Par::parse(&check.data(ci2)?)?;
    for (i, f) in inner2.files.iter().enumerate() {
        if is_script(f) && cmn::find_fades(&inner2.data(i)?)?.iter().any(|f| f.is_bright()) {
            return Err("verification failed".into());
        }
    }
    let same = |a: &[par::File], b: &[par::File]| a.iter().zip(b).all(|(x, y)| x.raw == y.raw);
    if !same(&outer.files, &check.files) || !same(&inner.files, &inner2.files) {
        return Err("verification failed (data mismatch)".into());
    }
    Ok(Some(Patched { bytes: out, backup, stats: total }))
}

/// Put the original script files back into a patched archive.
fn restore_archive(bytes: &[u8], backup: &[u8]) -> Result<Vec<u8>, String> {
    let mut outer = par::Par::parse(bytes)?;
    let ci = outer.find("cmn.par").ok_or("archive has no cmn.par")?;
    let mut inner = par::Par::parse(&outer.data(ci)?)?;
    for orig in par::Par::parse(backup)?.files {
        let name = par::name_str(&orig.name);
        let i = inner.find(&name).ok_or(format!("archive has no {name}"))?;
        inner.files[i] = orig;
    }
    outer.set_data(ci, inner.write());
    Ok(outer.write())
}

fn archives(data: &Path) -> Vec<(String, PathBuf)> {
    let mut v = Vec::new();
    for dir in CUTSCENE_DIRS {
        let Ok(rd) = fs::read_dir(data.join(dir)) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("par")) {
                v.push((dir.to_string(), p));
            }
        }
    }
    v.sort();
    v
}

fn run(mode: &str, game: &Path) -> Result<(), String> {
    let media = game.join("runtime").join("media");
    let data = media.join("data");
    if !steam::is_game_dir(game) {
        return Err(format!("game data not found at {}", data.display()));
    }
    let backup = media.join(BACKUP_DIR);

    match mode {
        "scan" | "patch" => {
            let (mut files, mut removed, mut darkened, mut errors) = (0, 0, 0, 0);
            for (dir, path) in archives(&data) {
                let name = path.file_name().unwrap().to_string_lossy().into_owned();
                let bytes = fs::read(&path).map_err(|e| e.to_string())?;
                match patch_archive(&bytes) {
                    Ok(None) => {}
                    Ok(Some(p)) => {
                        files += 1;
                        removed += p.stats.removed;
                        darkened += p.stats.darkened;
                        println!("{dir}/{name}: {} flash(es) removed, {} white fade(s) made black", p.stats.removed, p.stats.darkened);
                        if mode == "patch" {
                            let bdir = backup.join(&dir);
                            fs::create_dir_all(&bdir).map_err(|e| e.to_string())?;
                            fs::write(bdir.join(format!("{name}.{BACKUP_EXT}")), &p.backup)
                                .map_err(|e| e.to_string())?;
                            let tmp = path.with_extension("par.tmp");
                            fs::write(&tmp, &p.bytes).map_err(|e| e.to_string())?;
                            fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
                        }
                    }
                    Err(e) => {
                        errors += 1;
                        eprintln!("skipped {dir}/{name}: {e}");
                    }
                }
            }
            let verb = if mode == "patch" { "Patched" } else { "Would patch" };
            println!("\n{verb} {files} cutscenes: {removed} flashes removed, {darkened} white fades made black, {errors} skipped.");
            if mode == "patch" && files > 0 {
                println!("Originals backed up to {}", backup.display());
            }
        }
        "restore" => {
            let mut n = 0;
            for dir in CUTSCENE_DIRS {
                let Ok(rd) = fs::read_dir(backup.join(dir)) else { continue };
                for e in rd.flatten() {
                    let bname = e.file_name().to_string_lossy().into_owned();
                    let Some(name) = bname.strip_suffix(&format!(".{BACKUP_EXT}")) else { continue };
                    let target = data.join(dir).join(name);
                    let read = |p: &Path| fs::read(p).map_err(|e| format!("{}: {e}", p.display()));
                    let restored = restore_archive(&read(&target)?, &read(&e.path())?)?;
                    fs::write(&target, restored).map_err(|e| e.to_string())?;
                    n += 1;
                }
            }
            fs::remove_dir_all(&backup).ok();
            println!("Restored {n} original cutscene files.");
        }
        _ => return Err("usage: no-flashbang [patch|restore|scan] [game folder]".into()),
    }
    Ok(())
}

const USAGE: &str = "usage: no-flashbang [patch|restore|scan] [--game-dir DIR]

  patch    remove the white flashes (default)
  restore  put the original cutscenes back
  scan     show what would be patched, change nothing";

fn main() {
    let mut mode = None;
    let mut game = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--game-dir" => game = args.next().map(PathBuf::from),
            "-h" | "--help" => {
                println!("{USAGE}");
                return;
            }
            // A bare folder also works, e.g. dragging the game folder onto the exe.
            _ if mode.is_none() && ["patch", "restore", "scan"].contains(&arg.as_str()) => mode = Some(arg),
            _ if game.is_none() => game = Some(PathBuf::from(arg)),
            _ => {
                eprintln!("{USAGE}");
                std::process::exit(2);
            }
        }
    }
    let Some(game) = game.or_else(steam::find_game_dir) else {
        eprintln!("Could not find Like a Dragon: Infinite Wealth in any Steam library.");
        eprintln!("Pass its folder with --game-dir.");
        std::process::exit(1);
    };
    println!("Game folder: {}", game.display());
    if let Err(e) = run(mode.as_deref().unwrap_or("patch"), &game) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

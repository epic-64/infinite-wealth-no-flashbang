//! Reader/writer for RGG "PARC" archives and SLLZ decompression.

use std::io::Read;

#[derive(Clone)]
pub struct Folder {
    pub name: [u8; 64],
    pub sub_count: u32,
    pub sub_first: u32,
    pub file_count: u32,
    pub file_first: u32,
    pub attr: u32,
    pub rest: [u8; 12],
}

#[derive(Clone)]
pub struct File {
    pub name: [u8; 64],
    pub flag: u32,
    pub usize_: u32,
    pub attr: u32,
    pub timestamp: u64,
    /// Raw stored bytes (compressed if `flag & 0x8000_0000`).
    pub raw: Vec<u8>,
}

pub struct Par {
    pub header: [u8; 16],
    pub big_endian: bool,
    pub folders: Vec<Folder>,
    pub files: Vec<File>,
}

fn u32_at(d: &[u8], off: usize, be: bool) -> u32 {
    let b: [u8; 4] = d[off..off + 4].try_into().unwrap();
    if be { u32::from_be_bytes(b) } else { u32::from_le_bytes(b) }
}

fn u64_at(d: &[u8], off: usize, be: bool) -> u64 {
    let b: [u8; 8] = d[off..off + 8].try_into().unwrap();
    if be { u64::from_be_bytes(b) } else { u64::from_le_bytes(b) }
}

pub fn name_str(n: &[u8; 64]) -> String {
    let end = n.iter().position(|&c| c == 0).unwrap_or(64);
    String::from_utf8_lossy(&n[..end]).into_owned()
}

impl Par {
    pub fn parse(d: &[u8]) -> Result<Par, String> {
        if d.len() < 0x20 || &d[..4] != b"PARC" {
            return Err("not a PARC archive".into());
        }
        let be = d[5] == 1;
        let folder_count = u32_at(d, 0x10, be) as usize;
        let folder_table = u32_at(d, 0x14, be) as usize;
        let file_count = u32_at(d, 0x18, be) as usize;
        let file_table = u32_at(d, 0x1C, be) as usize;
        let name_at = |i: usize| -> [u8; 64] { d[0x20 + i * 64..0x20 + (i + 1) * 64].try_into().unwrap() };

        let mut folders = Vec::with_capacity(folder_count);
        for i in 0..folder_count {
            let o = folder_table + i * 32;
            folders.push(Folder {
                name: name_at(i),
                sub_count: u32_at(d, o, be),
                sub_first: u32_at(d, o + 4, be),
                file_count: u32_at(d, o + 8, be),
                file_first: u32_at(d, o + 12, be),
                attr: u32_at(d, o + 16, be),
                rest: d[o + 20..o + 32].try_into().unwrap(),
            });
        }
        let mut files = Vec::with_capacity(file_count);
        for i in 0..file_count {
            let o = file_table + i * 32;
            let flag = u32_at(d, o, be);
            let usize_ = u32_at(d, o + 4, be);
            let csize = u32_at(d, o + 8, be) as usize;
            let off = u32_at(d, o + 12, be) as u64 | ((u32_at(d, o + 20, be) as u64) << 32);
            let off = off as usize;
            if off + csize > d.len() {
                return Err(format!("file {} out of bounds", i));
            }
            files.push(File {
                name: name_at(folder_count + i),
                flag,
                usize_,
                attr: u32_at(d, o + 16, be),
                timestamp: u64_at(d, o + 24, be),
                raw: d[off..off + csize].to_vec(),
            });
        }
        Ok(Par { header: d[..16].try_into().unwrap(), big_endian: be, folders, files })
    }

    pub fn find(&self, name: &str) -> Option<usize> {
        self.files.iter().position(|f| name_str(&f.name) == name)
    }

    pub fn data(&self, i: usize) -> Result<Vec<u8>, String> {
        let f = &self.files[i];
        if f.raw.len() >= 4 && &f.raw[..4] == b"SLLZ" {
            sllz_decompress(&f.raw)
        } else {
            Ok(f.raw.clone())
        }
    }

    /// Replace a file's contents, stored uncompressed.
    pub fn set_data(&mut self, i: usize, data: Vec<u8>) {
        let f = &mut self.files[i];
        f.flag &= !0x8000_0000;
        f.usize_ = data.len() as u32;
        f.raw = data;
    }

    pub fn write(&self) -> Vec<u8> {
        const ALIGN: usize = 0x800;
        let be = self.big_endian;
        let put32 = |v: &mut Vec<u8>, x: u32| v.extend_from_slice(&if be { x.to_be_bytes() } else { x.to_le_bytes() });
        let put64 = |v: &mut Vec<u8>, x: u64| v.extend_from_slice(&if be { x.to_be_bytes() } else { x.to_le_bytes() });
        let nf = self.folders.len();
        let nfi = self.files.len();
        let folder_table = 0x20 + (nf + nfi) * 64;
        let file_table = folder_table + nf * 32;
        let table_end = file_table + nfi * 32;
        let align = |x: usize| x.div_ceil(ALIGN) * ALIGN;

        // Same layout as the game's own archives: files are packed back to back,
        // but a file that would reach a 0x800 boundary starts on the next one.
        let mut offsets = Vec::with_capacity(nfi);
        let mut pos = align(table_end);
        for f in &self.files {
            if pos % ALIGN + f.raw.len() >= ALIGN {
                pos = align(pos);
            }
            offsets.push(pos);
            pos += f.raw.len();
        }
        let pos = align(pos);

        let mut out = Vec::with_capacity(pos);
        out.extend_from_slice(&self.header);
        put32(&mut out, nf as u32);
        put32(&mut out, folder_table as u32);
        put32(&mut out, nfi as u32);
        put32(&mut out, file_table as u32);
        for fo in &self.folders {
            out.extend_from_slice(&fo.name);
        }
        for f in &self.files {
            out.extend_from_slice(&f.name);
        }
        for fo in &self.folders {
            put32(&mut out, fo.sub_count);
            put32(&mut out, fo.sub_first);
            put32(&mut out, fo.file_count);
            put32(&mut out, fo.file_first);
            put32(&mut out, fo.attr);
            out.extend_from_slice(&fo.rest);
        }
        for (f, &off) in self.files.iter().zip(&offsets) {
            put32(&mut out, f.flag);
            put32(&mut out, f.usize_);
            put32(&mut out, f.raw.len() as u32);
            put32(&mut out, off as u32);
            put32(&mut out, f.attr);
            put32(&mut out, (off as u64 >> 32) as u32);
            put64(&mut out, f.timestamp);
        }
        for (f, &off) in self.files.iter().zip(&offsets) {
            out.resize(off, 0);
            out.extend_from_slice(&f.raw);
        }
        out.resize(pos, 0);
        out
    }
}

pub fn sllz_decompress(d: &[u8]) -> Result<Vec<u8>, String> {
    if d.len() < 16 {
        return Err("SLLZ too short".into());
    }
    let be = d[4] == 1;
    let version = d[5];
    let hsize = if be { u16::from_be_bytes([d[6], d[7]]) } else { u16::from_le_bytes([d[6], d[7]]) } as usize;
    let usz = u32_at(d, 8, be) as usize;
    let csz = (u32_at(d, 12, be) as usize).min(d.len());
    let mut out = Vec::with_capacity(usz);
    let mut p = hsize;
    match version {
        1 => {
            let mut flag = 0u8;
            let mut bits = 0;
            while out.len() < usz {
                if bits == 0 {
                    flag = *d.get(p).ok_or("SLLZ v1 truncated")?;
                    p += 1;
                    bits = 8;
                }
                if flag & 0x80 != 0 {
                    let v = u16::from_le_bytes([d[p], d[p + 1]]) as usize;
                    p += 2;
                    let dist = (v >> 4) + 1;
                    let cnt = (v & 0xF) + 3;
                    if dist > out.len() {
                        return Err("SLLZ v1 bad distance".into());
                    }
                    for _ in 0..cnt {
                        out.push(out[out.len() - dist]);
                    }
                } else {
                    out.push(*d.get(p).ok_or("SLLZ v1 truncated")?);
                    p += 1;
                }
                flag <<= 1;
                bits -= 1;
            }
        }
        2 => {
            while p + 5 <= csz {
                let chunk = ((d[p] as usize) << 16) | ((d[p + 1] as usize) << 8) | d[p + 2] as usize;
                if chunk < 5 || p + chunk > d.len() {
                    return Err("SLLZ v2 bad chunk".into());
                }
                let mut z = flate2::read::ZlibDecoder::new(&d[p + 5..p + chunk]);
                z.read_to_end(&mut out).map_err(|e| format!("zlib: {e}"))?;
                p += chunk;
            }
        }
        v => return Err(format!("unknown SLLZ version {v}")),
    }
    if out.len() != usz {
        return Err(format!("SLLZ size mismatch {} != {}", out.len(), usz));
    }
    Ok(out)
}

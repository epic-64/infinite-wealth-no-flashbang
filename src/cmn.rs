//! HAct / auth `cmn.bin` node tree (Dragon Engine, version 18) and the fade patch.

const CATEGORY_ELEMENT: u32 = 12;
const ELEMENT_UI_FADE: u32 = 49;

/// Fades this short (in frames) are flashes: they get removed entirely.
const FLASH_MAX_FRAMES: f32 = 10.0;
/// Start frame for removed flashes, far past the end of any cutscene.
const NEVER: f32 = 1_000_000.0;

fn u32_at(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(d[o..o + 4].try_into().unwrap())
}
fn f32_at(d: &[u8], o: usize) -> f32 {
    f32::from_le_bytes(d[o..o + 4].try_into().unwrap())
}

pub struct Fade {
    /// Offset of the element data (starts with the element kind).
    pub data: usize,
    pub start: f32,
    pub end: f32,
    pub rgba: [u32; 4],
}

impl Fade {
    pub fn is_bright(&self) -> bool {
        let [r, g, b, a] = self.rgba;
        let luma = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
        a > 0 && luma >= 64.0
    }
}

/// Walk the node tree and return all UI fade elements.
pub fn find_fades(d: &[u8]) -> Result<Vec<Fade>, String> {
    if d.len() < 0x2C || u32_at(d, 0) != 18 {
        return Err("unsupported cmn version".into());
    }
    let mut fades = Vec::new();
    let mut stack = vec![u32_at(d, 0x28) as usize];
    // Nodes are stored depth-first: header (0x50 bytes), data, then children.
    // Iterate with an explicit counter of nodes still to read.
    let mut pos = stack.pop().unwrap();
    let mut remaining = 1usize;
    while remaining > 0 {
        if pos + 0x50 > d.len() {
            return Err("node out of bounds".into());
        }
        let category = u32_at(d, pos + 16);
        let size = u32_at(d, pos + 20) as usize * 4;
        let children = u32_at(d, pos + 24) as usize;
        let data = pos + 0x50;
        if data + size > d.len() {
            return Err("node data out of bounds".into());
        }
        if category == CATEGORY_ELEMENT && size >= 0xC0 && u32_at(d, data) == ELEMENT_UI_FADE {
            fades.push(Fade {
                data,
                start: f32_at(d, data + 4),
                end: f32_at(d, data + 8),
                rgba: [0, 4, 8, 12].map(|i| u32_at(d, data + 0xB0 + i)),
            });
        }
        remaining = remaining - 1 + children;
        pos = data + size;
    }
    Ok(fades)
}

#[derive(Default, Clone, Copy)]
pub struct Stats {
    pub removed: usize,
    pub darkened: usize,
}

/// Remove short bright flashes and turn longer bright fades black.
pub fn patch(d: &mut [u8]) -> Result<Stats, String> {
    let mut stats = Stats::default();
    for f in find_fades(d)? {
        if !f.is_bright() {
            continue;
        }
        // Black in every case, so even if a moved flash somehow plays it isn't white.
        for i in [0, 4, 8] {
            d[f.data + 0xB0 + i..f.data + 0xB4 + i].copy_from_slice(&0u32.to_le_bytes());
        }
        if f.end - f.start <= FLASH_MAX_FRAMES {
            d[f.data + 4..f.data + 8].copy_from_slice(&NEVER.to_le_bytes());
            d[f.data + 8..f.data + 12].copy_from_slice(&NEVER.to_le_bytes());
            stats.removed += 1;
        } else {
            stats.darkened += 1;
        }
    }
    Ok(stats)
}

//! Save container skeleton (versioned, checksummed, atomic). Layout, all integers little-endian:
//!   "PLTF" | schema u32 | payload_len u32 | payload | fnv1a64(everything before it) u64
//!   payload v1: seed u64, tick u64, requested_speed u64, n_streams u32, n * 4 * u64 (rng states), label_len u32, label (utf-8)
//! Golden bytes produced by the independent Python encoder (tools/ref/platform_ref.py) are checked in core/tests/save.rs.
//! Migrations are pure functions vN -> vN+1 (none exist yet: schema 1 is the first). Importing/export UI comes later.
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const MAGIC: &[u8; 4] = b"PLTF";
pub const CURRENT_SCHEMA: u32 = 1;
const MAX_STREAMS: u32 = 64;
const MAX_LABEL: u32 = 4096;
const HEADER_LEN: usize = 12;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveData {
    pub seed: u64,
    pub tick: u64,
    pub requested_speed: u64,
    pub rng_states: Vec<[u64; 4]>,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveError {
    BadMagic,
    TooNew(u32),
    UnknownSchema(u32),
    Truncated,
    BadLength,
    BadChecksum,
    Malformed(&'static str),
    Io(String),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn fnv1a64(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in data {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

pub fn encode(d: &SaveData) -> Vec<u8> {
    let mut p: Vec<u8> = Vec::new();
    p.extend_from_slice(&d.seed.to_le_bytes());
    p.extend_from_slice(&d.tick.to_le_bytes());
    p.extend_from_slice(&d.requested_speed.to_le_bytes());
    p.extend_from_slice(&(d.rng_states.len() as u32).to_le_bytes());
    for st in &d.rng_states {
        for w in st {
            p.extend_from_slice(&w.to_le_bytes());
        }
    }
    let lb = d.label.as_bytes();
    p.extend_from_slice(&(lb.len() as u32).to_le_bytes());
    p.extend_from_slice(lb);
    let mut out: Vec<u8> = Vec::with_capacity(HEADER_LEN + p.len() + 8);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&CURRENT_SCHEMA.to_le_bytes());
    out.extend_from_slice(&(p.len() as u32).to_le_bytes());
    out.extend_from_slice(&p);
    let sum = fnv1a64(&out);
    out.extend_from_slice(&sum.to_le_bytes());
    out
}

struct Reader<'a> {
    b: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], SaveError> {
        if self.b.len() - self.pos < n {
            return Err(SaveError::Truncated);
        }
        let s = &self.b[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }
    fn u32(&mut self) -> Result<u32, SaveError> {
        let s = self.take(4)?;
        Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }
    fn u64(&mut self) -> Result<u64, SaveError> {
        let s = self.take(8)?;
        let mut a = [0u8; 8];
        a.copy_from_slice(s);
        Ok(u64::from_le_bytes(a))
    }
}

/// Migration hook: turns a payload of an older schema into the current one. Only schema 1 exists.
fn migrate_payload(schema: u32, payload: &[u8]) -> Result<Vec<u8>, SaveError> {
    match schema {
        1 => Ok(payload.to_vec()),
        _ => Err(SaveError::UnknownSchema(schema)),
    }
}

pub fn decode(bytes: &[u8]) -> Result<SaveData, SaveError> {
    if bytes.len() < HEADER_LEN + 8 {
        return Err(SaveError::Truncated);
    }
    if bytes[0..4] != MAGIC[..] {
        return Err(SaveError::BadMagic);
    }
    let mut r = Reader { b: bytes, pos: 4 };
    let schema = r.u32()?;
    if schema > CURRENT_SCHEMA {
        return Err(SaveError::TooNew(schema));
    }
    if schema == 0 {
        return Err(SaveError::UnknownSchema(0));
    }
    let plen = r.u32()? as usize;
    if bytes.len() != HEADER_LEN + plen + 8 {
        return Err(SaveError::BadLength);
    }
    let stored = {
        let mut a = [0u8; 8];
        a.copy_from_slice(&bytes[HEADER_LEN + plen..]);
        u64::from_le_bytes(a)
    };
    if fnv1a64(&bytes[..HEADER_LEN + plen]) != stored {
        return Err(SaveError::BadChecksum);
    }
    let payload = migrate_payload(schema, &bytes[HEADER_LEN..HEADER_LEN + plen])?;
    let mut p = Reader { b: &payload, pos: 0 };
    let seed = p.u64()?;
    let tick = p.u64()?;
    let requested_speed = p.u64()?;
    let n = p.u32()?;
    if n > MAX_STREAMS {
        return Err(SaveError::Malformed("too many rng streams"));
    }
    let mut rng_states = Vec::new();
    for _ in 0..n {
        rng_states.push([p.u64()?, p.u64()?, p.u64()?, p.u64()?]);
    }
    let ll = p.u32()?;
    if ll > MAX_LABEL {
        return Err(SaveError::Malformed("label too long"));
    }
    let lb = p.take(ll as usize)?;
    let label = String::from_utf8(lb.to_vec()).map_err(|_| SaveError::Malformed("label is not utf-8"))?;
    if p.pos != payload.len() {
        return Err(SaveError::Malformed("trailing bytes in payload"));
    }
    Ok(SaveData { seed, tick, requested_speed, rng_states, label })
}

fn sibling(path: &Path, ext: &str) -> PathBuf {
    path.with_extension(ext)
}

/// Atomic write: temp file -> flush to disk -> old file kept as `.bak` -> rename temp into place.
/// (A directory fsync is not done; acceptable for now, recorded as a limitation.)
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = sibling(path, "tmp");
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    if path.exists() {
        fs::rename(path, sibling(path, "bak"))?;
    }
    fs::rename(&tmp, path)
}

pub fn save_slot(path: &Path, d: &SaveData) -> Result<(), SaveError> {
    write_atomic(path, &encode(d)).map_err(|e| SaveError::Io(e.to_string()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadSource {
    Main,
    Temp,
    Backup,
}

/// Tries the main file, then a completed temp file (crash between the two renames), then the backup.
pub fn load_slot(path: &Path) -> Result<(SaveData, LoadSource), SaveError> {
    let candidates = [(path.to_path_buf(), LoadSource::Main), (sibling(path, "tmp"), LoadSource::Temp), (sibling(path, "bak"), LoadSource::Backup)];
    let mut first_err: Option<SaveError> = None;
    for (p, src) in candidates.iter() {
        match fs::read(p) {
            Ok(bytes) => match decode(&bytes) {
                Ok(d) => return Ok((d, *src)),
                Err(e) => {
                    if first_err.is_none() {
                        first_err = Some(e);
                    }
                }
            },
            Err(e) => {
                if first_err.is_none() && *src == LoadSource::Main {
                    first_err = Some(SaveError::Io(e.to_string()));
                }
            }
        }
    }
    Err(first_err.unwrap_or(SaveError::Truncated))
}

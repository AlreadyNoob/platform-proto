//! Save container (versioned, checksummed, atomic). Layout, all integers little-endian:
//!   "PLTF" | schema u32 | payload_len u32 | payload | fnv1a64(everything before it) u64
//!   payload v1: seed u64, tick u64, requested_speed u64, n_streams u32, n * 4 * u64 (rng states), label_len u32, label (utf-8)
//!   payload v2: the v1 fields, then a world block: u8 present (0/1); if 1: base_p32, cap_p32, count, free, chains (5 x u64),
//!     n_types u32 + per type (ascending id) [id u16, cost u64, count_per_tick u64, spawn_p32 u64, spawn_type u16, blocked_tag u8, blocked_n u64, free_first u8],
//!     n_cells u32 + per cell (ascending y then x) [y i32, x i32, kind u16]
//! Golden bytes from the independent Python encoders (tools/ref/platform_ref.py, tools/ref/save2_ref.py) are checked in core/tests/.
//! Migration: a schema 1 file is read as schema 2 with no world (the pure function `migrate_payload`). New files are written as schema 2.
use crate::collapse::{BlockedPolicy, TypeConfig};
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const MAGIC: &[u8; 4] = b"PLTF";
pub const CURRENT_SCHEMA: u32 = 2;
const MAX_STREAMS: u32 = 64;
const MAX_LABEL: u32 = 4096;
const MAX_TYPES: u32 = 1024;
const CELL_BYTES: usize = 10;
const HEADER_LEN: usize = 12;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveData {
    pub seed: u64,
    pub tick: u64,
    pub requested_speed: u64,
    pub rng_states: Vec<[u64; 4]>,
    pub label: String,
}

/// Everything about the world that is not in the header: the collapse model, the economy, the types and every particle (y, x, kind).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldState {
    pub base_p32: u64,
    pub cap_p32: u64,
    pub count: u64,
    pub free: u64,
    pub chains: u64,
    /// ascending by id
    pub types: Vec<TypeConfig>,
    /// ascending by (y, x)
    pub cells: Vec<(i32, i32, u16)>,
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

fn encode_world(p: &mut Vec<u8>, w: Option<&WorldState>) {
    let ws = match w {
        None => {
            p.push(0);
            return;
        }
        Some(ws) => ws,
    };
    p.push(1);
    for v in [ws.base_p32, ws.cap_p32, ws.count, ws.free, ws.chains] {
        p.extend_from_slice(&v.to_le_bytes());
    }
    p.extend_from_slice(&(ws.types.len() as u32).to_le_bytes());
    for t in ws.types.iter() {
        p.extend_from_slice(&t.id.to_le_bytes());
        p.extend_from_slice(&t.cost.to_le_bytes());
        p.extend_from_slice(&t.count_per_tick.to_le_bytes());
        p.extend_from_slice(&t.spawn_p32.to_le_bytes());
        p.extend_from_slice(&t.spawn_type.to_le_bytes());
        let (tag, n) = match t.blocked {
            BlockedPolicy::Skip => (0u8, 0u64),
            BlockedPolicy::ConvertToCount(n) => (1u8, n),
        };
        p.push(tag);
        p.extend_from_slice(&n.to_le_bytes());
        p.push(if t.free_first { 1 } else { 0 });
    }
    p.extend_from_slice(&(ws.cells.len() as u32).to_le_bytes());
    for (y, x, k) in ws.cells.iter() {
        p.extend_from_slice(&y.to_le_bytes());
        p.extend_from_slice(&x.to_le_bytes());
        p.extend_from_slice(&k.to_le_bytes());
    }
}

/// Encodes the header and, optionally, the whole world (always as the current schema).
pub fn encode_game(d: &SaveData, w: Option<&WorldState>) -> Vec<u8> {
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
    encode_world(&mut p, w);
    let mut out: Vec<u8> = Vec::with_capacity(HEADER_LEN + p.len() + 8);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&CURRENT_SCHEMA.to_le_bytes());
    out.extend_from_slice(&(p.len() as u32).to_le_bytes());
    out.extend_from_slice(&p);
    let sum = fnv1a64(&out);
    out.extend_from_slice(&sum.to_le_bytes());
    out
}

/// Header only (no world).
pub fn encode(d: &SaveData) -> Vec<u8> {
    encode_game(d, None)
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
    fn u8(&mut self) -> Result<u8, SaveError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, SaveError> {
        let s = self.take(2)?;
        Ok(u16::from_le_bytes([s[0], s[1]]))
    }
    fn i32(&mut self) -> Result<i32, SaveError> {
        let s = self.take(4)?;
        Ok(i32::from_le_bytes([s[0], s[1], s[2], s[3]]))
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

/// Migration: turns a payload of an older schema into the current one (a pure function).
/// 1 -> 2: schema 1 had no world, so the world block "not present" (one zero byte) is appended.
fn migrate_payload(schema: u32, payload: &[u8]) -> Result<Vec<u8>, SaveError> {
    match schema {
        1 => {
            let mut v = payload.to_vec();
            v.push(0);
            Ok(v)
        }
        2 => Ok(payload.to_vec()),
        _ => Err(SaveError::UnknownSchema(schema)),
    }
}

fn read_world(p: &mut Reader) -> Result<WorldState, SaveError> {
    let base_p32 = p.u64()?;
    let cap_p32 = p.u64()?;
    let count = p.u64()?;
    let free = p.u64()?;
    let chains = p.u64()?;
    let nt = p.u32()?;
    if nt > MAX_TYPES {
        return Err(SaveError::Malformed("too many types"));
    }
    let mut types: Vec<TypeConfig> = Vec::new();
    for _ in 0..nt {
        let id = p.u16()?;
        if let Some(prev) = types.last() {
            if id <= prev.id {
                return Err(SaveError::Malformed("types not sorted or not unique"));
            }
        }
        let cost = p.u64()?;
        let count_per_tick = p.u64()?;
        let spawn_p32 = p.u64()?;
        let spawn_type = p.u16()?;
        let tag = p.u8()?;
        let bn = p.u64()?;
        let ff = p.u8()?;
        let blocked = match tag {
            0 => {
                if bn != 0 {
                    return Err(SaveError::Malformed("blocked policy has a stray value"));
                }
                BlockedPolicy::Skip
            }
            1 => BlockedPolicy::ConvertToCount(bn),
            _ => return Err(SaveError::Malformed("unknown blocked policy")),
        };
        if ff > 1 {
            return Err(SaveError::Malformed("bad free_first flag"));
        }
        types.push(TypeConfig { id, cost, count_per_tick, spawn_p32, spawn_type, blocked, free_first: ff == 1 });
    }
    let nc = p.u32()? as usize;
    let remaining = p.b.len() - p.pos;
    if nc.checked_mul(CELL_BYTES).map_or(true, |n| n > remaining) {
        return Err(SaveError::Truncated);
    }
    let mut cells: Vec<(i32, i32, u16)> = Vec::with_capacity(nc);
    for _ in 0..nc {
        let y = p.i32()?;
        let x = p.i32()?;
        let k = p.u16()?;
        if let Some((py, px, _)) = cells.last() {
            if (y, x) <= (*py, *px) {
                return Err(SaveError::Malformed("cells not sorted or not unique"));
            }
        }
        cells.push((y, x, k));
    }
    Ok(WorldState { base_p32, cap_p32, count, free, chains, types, cells })
}

/// Decodes any supported schema (older ones are migrated). Returns the header and, if the file has one, the world.
pub fn decode_game(bytes: &[u8]) -> Result<(SaveData, Option<WorldState>), SaveError> {
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
    let world = match p.u8()? {
        0 => None,
        1 => Some(read_world(&mut p)?),
        _ => return Err(SaveError::Malformed("bad world flag")),
    };
    if p.pos != payload.len() {
        return Err(SaveError::Malformed("trailing bytes in payload"));
    }
    Ok((SaveData { seed, tick, requested_speed, rng_states, label }, world))
}

/// Header only: the world (if any) is ignored.
pub fn decode(bytes: &[u8]) -> Result<SaveData, SaveError> {
    decode_game(bytes).map(|(d, _)| d)
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

pub fn save_slot_game(path: &Path, d: &SaveData, w: Option<&WorldState>) -> Result<(), SaveError> {
    write_atomic(path, &encode_game(d, w)).map_err(|e| SaveError::Io(e.to_string()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadSource {
    Main,
    Temp,
    Backup,
}

/// Tries the main file, then a completed temp file (crash between the two renames), then the backup. Header only.
pub fn load_slot(path: &Path) -> Result<(SaveData, LoadSource), SaveError> {
    load_slot_game(path).map(|(d, _, src)| (d, src))
}

/// Same, returning the world too when the file has one.
pub fn load_slot_game(path: &Path) -> Result<(SaveData, Option<WorldState>, LoadSource), SaveError> {
    let candidates = [(path.to_path_buf(), LoadSource::Main), (sibling(path, "tmp"), LoadSource::Temp), (sibling(path, "bak"), LoadSource::Backup)];
    let mut first_err: Option<SaveError> = None;
    for (p, src) in candidates.iter() {
        match fs::read(p) {
            Ok(bytes) => match decode_game(&bytes) {
                Ok((d, w)) => return Ok((d, w, *src)),
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

//! What a ROM file is: its console, its identity from the content, its title and its icon.
//!
//! Identity (docs/feasibility.md, "Game identity"), written `<console>:<hex>`:
//! - GB / GBC / GBA: SHA-1 of the whole file.
//! - NDS: SHA-1 of the first *used size* bytes (header 0x80; DSi-enhanced titles 0x210), so a
//!   trimmed dump and the untrimmed one are the same game.
//! - 3DS: program ID of the NCCH (header 0x118) and its remaster version (extended header
//!   0x0E), `3ds:<program id, 16 hex><version, 4 hex>`. No hash: the title ID already names the
//!   game, region and all.
//! - 3DS homebrew (`.3dsx`): no title ID, so the SHA-1 of the whole file, as for GBA.
//!
//! Extensions: `gb`, `gbc`, `gba`, `nds`, `3ds` / `cci` (NCSD card image), `cxi` (a bare NCCH),
//! `cia` (installable archive: the NCCH of content 0), `3dsx` (homebrew, its SMDH appended when
//! it has one). A file whose header does not look like its console is not a game
//! ([`NotAGame`]) and is skipped; so is an encrypted 3DS game ([`NotAGame::Encrypted`], E10:
//! EmuDome plays decrypted games only, `crate::n3ds::rom`).
//!
//! Titles: GB 0x134, GBA 0xA0, NDS banner (English, else the first non-empty language; the
//! header's 0x00 when there is no banner), 3DS SMDH short title (same language rule).
//! Icons: NDS banner 32×32 (4 bpp, 8×8 tiles, BGR555 palette, colour 0 transparent), 3DS SMDH
//! large icon 48×48 (RGB565, 8×8 tiles in Morton order).

use std::fmt;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use crate::settings::ConsoleId;
use crate::GameKey;

/// The file extensions the library looks at (lowercase).
pub const EXTENSIONS: &[&str] = &[
    "gb", "gbc", "gba", "nds", "3ds", "cci", "cxi", "cia", "3dsx",
];

/// What a game is, from its content. Cached by path + size + modification time.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    /// `<console>:<hex>`: the game's one key.
    pub id: GameKey,
    pub console: ConsoleId,
    /// From the header / banner / SMDH; empty when the ROM has none.
    pub title: String,
    pub has_icon: bool,
    /// Game / product code from the header (GBA 0xAC, NDS 0x0C, 3DS product code), if any.
    pub code: Option<String>,
    /// ROM version (GBA 0xBC, NDS 0x1E, 3DS remaster or title version).
    pub version: Option<u16>,
}

/// Why a file was not taken as a game (logged, never shown).
#[derive(Debug)]
pub enum NotAGame {
    /// The extension belongs to no console.
    Extension,
    /// The header does not match the console the extension says.
    Header(&'static str),
    /// An encrypted 3DS game (E10): EmuDome plays decrypted games only, and never adds or opens
    /// an encrypted one (`crate::n3ds::rom`).
    Encrypted,
    Io(io::Error),
}

impl fmt::Display for NotAGame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotAGame::Extension => f.write_str("unknown extension"),
            NotAGame::Header(why) => write!(f, "bad header: {why}"),
            NotAGame::Encrypted => f.write_str("encrypted 3DS content"),
            NotAGame::Io(e) => write!(f, "{e}"),
        }
    }
}

impl From<io::Error> for NotAGame {
    fn from(e: io::Error) -> Self {
        NotAGame::Io(e)
    }
}

/// A decoded icon, RGBA 8 bit, row by row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Icon {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Icon {
    /// The icon as a PNG file.
    pub fn to_png(&self) -> Result<Vec<u8>, png::EncodingError> {
        crate::frame::rgba_png(self.width, self.height, &self.rgba)
    }
}

/// The container a file is, from its extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Format {
    Gb,
    Gbc,
    Gba,
    Nds,
    /// `.3ds` / `.cci`: NCSD card image.
    Ncsd,
    /// `.cxi`: a bare NCCH.
    Ncch,
    Cia,
    /// `.3dsx`: homebrew.
    Homebrew3dsx,
}

fn format_of(path: &Path) -> Option<Format> {
    format_of_extension(&path.extension()?.to_string_lossy())
}

fn format_of_extension(ext: &str) -> Option<Format> {
    match ext.to_lowercase().as_str() {
        "gb" => Some(Format::Gb),
        "gbc" => Some(Format::Gbc),
        "gba" => Some(Format::Gba),
        "nds" => Some(Format::Nds),
        "3ds" | "cci" => Some(Format::Ncsd),
        "cxi" => Some(Format::Ncch),
        "cia" => Some(Format::Cia),
        "3dsx" => Some(Format::Homebrew3dsx),
        _ => None,
    }
}

/// True when the extension is one the library looks at.
pub fn is_rom(path: &Path) -> bool {
    format_of(path).is_some()
}

/// The console a file extension belongs to (one of [`EXTENSIONS`], any case), before its
/// content is read: a `.gb` whose header asks for a Game Boy Color is still `Gb` here.
pub fn console_of_extension(ext: &str) -> Option<ConsoleId> {
    Some(match format_of_extension(ext)? {
        Format::Gb => ConsoleId::Gb,
        Format::Gbc => ConsoleId::Gbc,
        Format::Gba => ConsoleId::Gba,
        Format::Nds => ConsoleId::Nds,
        Format::Ncsd | Format::Ncch | Format::Cia | Format::Homebrew3dsx => ConsoleId::N3ds,
    })
}

/// Where the icon lives in the file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IconAt {
    /// NDS banner at this offset.
    Banner(u64),
    /// 3DS SMDH at this offset.
    Smdh(u64),
}

/// What the header says, before any hashing.
struct Header {
    console: ConsoleId,
    title: String,
    code: Option<String>,
    version: Option<u16>,
    icon: Option<IconAt>,
    /// How the id is made.
    key: Key,
}

enum Key {
    /// SHA-1 of the first `n` bytes.
    Hash(u64),
    /// Already known (3DS).
    Known(String),
}

/// The name shown for a game. GB / GBC / GBA headers hold 11–16 upper-case characters and a
/// ROM hack keeps its base game's («POKEMON FIRE» for a FireRed hack): the file name says
/// more, so it wins there. NDS banners and 3DS SMDH carry the real name: they win, with the
/// file name as the fallback. The user's own override comes with the game details (L04).
pub fn display_title(console: ConsoleId, header_title: &str, path: &Path) -> String {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().trim().to_owned())
        .unwrap_or_default();
    let by_file = matches!(console, ConsoleId::Gb | ConsoleId::Gbc | ConsoleId::Gba);
    if (by_file && !stem.is_empty()) || header_title.trim().is_empty() {
        stem
    } else {
        header_title.to_owned()
    }
}

/// Identifies the game in `path`: reads the header and hashes what the identity needs.
pub fn identify(path: &Path) -> Result<Identity, NotAGame> {
    let mut file = File::open(path)?;
    let header = inspect(path, &mut file)?;
    let hex = match header.key {
        Key::Hash(n) => sha1_prefix(&mut file, n)?,
        Key::Known(hex) => hex,
    };
    let title = display_title(header.console, &header.title, path);
    let id = GameKey::new(header.console, &hex).map_err(|e| {
        log::warn!("{}: {e}", path.display());
        NotAGame::Header("identity not made of hex digits")
    })?;
    Ok(Identity {
        id,
        console: header.console,
        title,
        has_icon: header.icon.is_some(),
        code: header.code,
        version: header.version,
    })
}

/// The icon inside the ROM (NDS banner, 3DS SMDH), without hashing anything.
pub fn read_icon(path: &Path) -> Result<Option<Icon>, NotAGame> {
    let mut file = File::open(path)?;
    let header = inspect(path, &mut file)?;
    Ok(match header.icon {
        Some(IconAt::Banner(at)) => Some(nds_icon(&read_at(&mut file, at + 0x20, 0x220)?)),
        Some(IconAt::Smdh(at)) => Some(smdh_icon(&read_at(&mut file, at + 0x24C0, 0x1200)?)),
        None => None,
    })
}

fn inspect(path: &Path, file: &mut File) -> Result<Header, NotAGame> {
    let format = format_of(path).ok_or(NotAGame::Extension)?;
    let len = file.metadata()?.len();
    match format {
        Format::Gb | Format::Gbc => gb(file, len, format),
        Format::Gba => gba(file, len),
        Format::Nds => nds(file, len),
        Format::Ncsd => ncsd(file, len),
        Format::Ncch => ncch(file, 0, len)?.ok_or(NotAGame::Header("no NCCH magic")),
        Format::Cia => cia(file, len),
        Format::Homebrew3dsx => homebrew_3dsx(file, len),
    }
}

// ---- GB / GBC ----

fn gb(file: &mut File, len: u64, format: Format) -> Result<Header, NotAGame> {
    if len < 0x150 {
        return Err(NotAGame::Header("GB file shorter than its header"));
    }
    let h = read_at(file, 0x134, 0x1A)?;
    // Header checksum (0x14D) over 0x134..=0x14C.
    let sum = h[..0x19]
        .iter()
        .fold(0u8, |x, b| x.wrapping_sub(*b).wrapping_sub(1));
    if sum != h[0x19] {
        return Err(NotAGame::Header("GB header checksum"));
    }
    let cgb_flag = h[0x0F];
    // CGB-only cartridges are GBC whatever the extension; dual ones follow it.
    let console = if cgb_flag == 0xC0 || format == Format::Gbc {
        ConsoleId::Gbc
    } else {
        ConsoleId::Gb
    };
    // Newer cartridges use 0x143 for the CGB flag: the title is 15 bytes then.
    let title_len = if cgb_flag & 0x80 != 0 { 15 } else { 16 };
    Ok(Header {
        console,
        title: ascii(&h[..title_len]),
        code: None,
        version: Some(u16::from(h[0x18])),
        icon: None,
        key: Key::Hash(len),
    })
}

// ---- GBA ----

fn gba(file: &mut File, len: u64) -> Result<Header, NotAGame> {
    if len < 0xC0 {
        return Err(NotAGame::Header("GBA file shorter than its header"));
    }
    let h = read_at(file, 0xA0, 0x20)?;
    // Fixed value at 0xB2.
    if h[0x12] != 0x96 {
        return Err(NotAGame::Header("GBA fixed value 0x96"));
    }
    Ok(Header {
        console: ConsoleId::Gba,
        title: ascii(&h[..0x0C]),
        code: code(&h[0x0C..0x10]),
        version: Some(u16::from(h[0x1C])),
        icon: None,
        key: Key::Hash(len),
    })
}

// ---- NDS ----

fn nds(file: &mut File, len: u64) -> Result<Header, NotAGame> {
    if len < 0x200 {
        return Err(NotAGame::Header("NDS file shorter than its header"));
    }
    let h = read_at(file, 0, 0x200)?;
    if crc16(&h[..0x15E]) != le16(&h, 0x15E) {
        return Err(NotAGame::Header("NDS header CRC"));
    }
    let mut used = u64::from(le32(&h, 0x80));
    // DSi-enhanced / DSi-only: the used size including the DSi areas.
    if h[0x12] & 0x02 != 0 && le32(&h, 0x210) != 0 {
        used = u64::from(le32(&h, 0x210));
    }
    if used == 0 || used > len {
        used = len;
    }
    let banner = u64::from(le32(&h, 0x68));
    let mut title = String::new();
    let mut icon = None;
    if banner != 0 && banner + 0x840 <= len {
        let b = read_at(file, banner, 0x840)?;
        if le16(&b, 0) != 0 {
            icon = Some(IconAt::Banner(banner));
            title = pick_title(6, |lang| {
                let text = utf16(&b[0x240 + lang * 0x100..0x340 + lang * 0x100]);
                text.lines().next().unwrap_or_default().trim().to_owned()
            });
        }
    }
    if title.is_empty() {
        title = ascii(&h[..0x0C]);
    }
    Ok(Header {
        console: ConsoleId::Nds,
        title,
        code: code(&h[0x0C..0x10]),
        version: Some(u16::from(h[0x1E])),
        icon,
        key: Key::Hash(used),
    })
}

/// CRC-16 as the NDS header uses it (MODBUS: reflected 0x8005, init 0xFFFF).
fn crc16(bytes: &[u8]) -> u16 {
    bytes.iter().fold(0xFFFF, |mut crc: u16, b| {
        crc ^= u16::from(*b);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xA001
            } else {
                crc >> 1
            };
        }
        crc
    })
}

fn nds_icon(b: &[u8]) -> Icon {
    // b: bitmap (0x200) then palette (0x20), from banner + 0x20.
    let palette: Vec<[u8; 4]> = (0..16)
        .map(|i| {
            if i == 0 {
                return [0, 0, 0, 0];
            }
            let c = le16(b, 0x200 + i * 2);
            [five(c), five(c >> 5), five(c >> 10), 0xFF]
        })
        .collect();
    let mut rgba = vec![0u8; 32 * 32 * 4];
    for tile in 0..16 {
        let (tx, ty) = (tile % 4 * 8, tile / 4 * 8);
        for i in 0..64 {
            let byte = b[tile * 32 + i / 2];
            let index = if i % 2 == 0 { byte & 0x0F } else { byte >> 4 };
            let (x, y) = (tx + i % 8, ty + i / 8);
            let at = (y * 32 + x) * 4;
            rgba[at..at + 4].copy_from_slice(&palette[usize::from(index)]);
        }
    }
    Icon {
        width: 32,
        height: 32,
        rgba,
    }
}

// ---- 3DS ----

use crate::n3ds::rom::{ncch_encrypted, ncsd_game_partition, MEDIA_UNIT};

fn ncsd(file: &mut File, len: u64) -> Result<Header, NotAGame> {
    if len < 0x200 {
        return Err(NotAGame::Header("NCSD file shorter than its header"));
    }
    let h = read_at(file, 0, 0x200)?;
    // Partition table at 0x120: (offset, length) in media units; partition 0 is the game.
    let base = ncsd_game_partition(&h).ok_or(NotAGame::Header("no NCSD magic"))?;
    ncch(file, base, len)?.ok_or(NotAGame::Header("NCSD partition 0 is not an NCCH"))
}

/// The NCCH at `base`, or `None` when there is none there. An encrypted one is not a game
/// ([`NotAGame::Encrypted`]): EmuDome never adds or opens it.
fn ncch(file: &mut File, base: u64, len: u64) -> Result<Option<Header>, NotAGame> {
    if base + 0x200 > len {
        return Ok(None);
    }
    let h = read_at(file, base, 0x200)?;
    match ncch_encrypted(&h) {
        None => return Ok(None),
        Some(true) => return Err(NotAGame::Encrypted),
        Some(false) => {}
    }
    let program_id = le64(&h, 0x118);
    let unit = MEDIA_UNIT << h[0x18E].min(16);
    let product = ascii(&h[0x150..0x160]);
    let mut version = None;
    let mut smdh = None;
    // Extended header right after the NCCH header; SystemInfoFlags at 0x08, remaster version at
    // 0x0E.
    if le32(&h, 0x180) != 0 && base + 0x210 <= len {
        version = Some(le16(&read_at(file, base + 0x200, 0x10)?, 0x0E));
    }
    let exefs = base + u64::from(le32(&h, 0x1A0)) * unit;
    if le32(&h, 0x1A4) != 0 && exefs + 0x200 <= len {
        let x = read_at(file, exefs, 0x200)?;
        // 8 sections: name (8), offset (4), size (4); data after the 0x200 header.
        smdh = (0..8)
            .map(|i| &x[i * 16..i * 16 + 16])
            .find(|s| &s[..8] == b"icon\0\0\0\0")
            .map(|s| exefs + 0x200 + u64::from(le32(s, 8)));
    }
    let title = smdh_title(file, smdh, len)?;
    let icon = title.as_ref().and(smdh).map(IconAt::Smdh);
    Ok(Some(Header {
        console: ConsoleId::N3ds,
        title: title.unwrap_or_default(),
        code: (!product.is_empty()).then_some(product),
        version,
        icon,
        key: Key::Known(format!(
            "{program_id:016x}{:04x}",
            version.unwrap_or_default()
        )),
    }))
}

fn cia(file: &mut File, len: u64) -> Result<Header, NotAGame> {
    if len < 0x20 {
        return Err(NotAGame::Header("CIA file shorter than its header"));
    }
    let h = read_at(file, 0, 0x20)?;
    let header_size = u64::from(le32(&h, 0x00));
    if header_size != 0x2020 {
        return Err(NotAGame::Header("CIA header size"));
    }
    // Sections, each aligned to 64 bytes: header, certificates, ticket, TMD, content, meta.
    let align = |x: u64| (x + 63) & !63;
    let cert = align(header_size);
    let ticket = cert + align(u64::from(le32(&h, 0x08)));
    let tmd = ticket + align(u64::from(le32(&h, 0x0C)));
    let content = tmd + align(u64::from(le32(&h, 0x10)));
    let meta_size = u64::from(le32(&h, 0x14));
    let meta = content + align(le64(&h, 0x18));

    // The SMDH of the meta section (after 0x400 bytes of dependencies and core version).
    let meta_smdh = (meta_size >= 0x400 + 0x36C0).then_some(meta + 0x400);

    if let Some(mut header) = ncch(file, content, len)? {
        if header.icon.is_none() {
            if let Some(title) = smdh_title(file, meta_smdh, len)? {
                header.title = title;
                header.icon = meta_smdh.map(IconAt::Smdh);
            }
        }
        return Ok(header);
    }

    // No NCCH to read at content 0: the content is encrypted (with its title key, which
    // EmuDome never has nor reads).
    if tmd + 4 > len {
        return Err(NotAGame::Header("CIA without TMD"));
    }
    Err(NotAGame::Encrypted)
}

/// A `.3dsx` (3dbrew «3DSX Format»): magic `3DSX`, the header's size at 0x04; a header of 0x2C
/// bytes or more is the extended one, with the SMDH's offset at 0x20 and its size at 0x24.
fn homebrew_3dsx(file: &mut File, len: u64) -> Result<Header, NotAGame> {
    if len < 0x2C {
        return Err(NotAGame::Header("3DSX file shorter than its header"));
    }
    let h = read_at(file, 0, 0x2C)?;
    if &h[..4] != b"3DSX" {
        return Err(NotAGame::Header("no 3DSX magic"));
    }
    let smdh =
        (le16(&h, 0x04) >= 0x2C && le32(&h, 0x24) >= 0x36C0).then(|| u64::from(le32(&h, 0x20)));
    let title = smdh_title(file, smdh, len)?;
    let icon = title.as_ref().and(smdh).map(IconAt::Smdh);
    Ok(Header {
        console: ConsoleId::N3ds,
        title: title.unwrap_or_default(),
        code: None,
        version: None,
        icon,
        key: Key::Hash(len),
    })
}

/// The short title of the SMDH at `at`, `None` when there is no valid SMDH there.
fn smdh_title(file: &mut File, at: Option<u64>, len: u64) -> Result<Option<String>, NotAGame> {
    let Some(at) = at.filter(|at| at + 0x36C0 <= len) else {
        return Ok(None);
    };
    let s = read_at(file, at, 0x2008)?;
    if &s[..4] != b"SMDH" {
        return Ok(None);
    }
    // 16 titles of 0x200 bytes from 0x08: short (0x80), long (0x100), publisher (0x80).
    Ok(Some(pick_title(16, |lang| {
        utf16(&s[0x08 + lang * 0x200..0x88 + lang * 0x200])
            .trim()
            .to_owned()
    })))
}

fn smdh_icon(b: &[u8]) -> Icon {
    // 48×48 RGB565, 8×8 tiles left to right, top to bottom; Morton order inside a tile.
    let mut rgba = vec![0u8; 48 * 48 * 4];
    for tile in 0..36 {
        let (tx, ty) = (tile % 6 * 8, tile / 6 * 8);
        for i in 0..64 {
            let x = (i & 1) | ((i >> 1) & 2) | ((i >> 2) & 4);
            let y = ((i >> 1) & 1) | ((i >> 2) & 2) | ((i >> 3) & 4);
            let c = le16(b, (tile * 64 + i) * 2);
            let r = (c >> 11) & 0x1F;
            let g = (c >> 5) & 0x3F;
            let bl = c & 0x1F;
            let at = ((ty + y) * 48 + tx + x) * 4;
            rgba[at..at + 4].copy_from_slice(&[
                expand5(r),
                ((g << 2) | (g >> 4)) as u8,
                expand5(bl),
                0xFF,
            ]);
        }
    }
    Icon {
        width: 48,
        height: 48,
        rgba,
    }
}

// ---- Helpers ----

/// English (language 1) when it is there, else the first non-empty language.
fn pick_title(languages: usize, title: impl Fn(usize) -> String) -> String {
    std::iter::once(1)
        .chain(0..languages)
        .map(title)
        .find(|t| !t.is_empty())
        .unwrap_or_default()
}

/// SHA-1 of the first `n` bytes of the file (all of it if shorter), as lowercase hex.
fn sha1_prefix(file: &mut File, n: u64) -> io::Result<String> {
    file.seek(SeekFrom::Start(0))?;
    let mut hasher = Sha1::new();
    let mut reader = file.take(n);
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let read = reader.read(&mut buf)?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    Ok(hex(&hasher.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn read_at(file: &mut File, at: u64, len: usize) -> io::Result<Vec<u8>> {
    file.seek(SeekFrom::Start(at))?;
    let mut buf = vec![0u8; len];
    file.read_exact(&mut buf)?;
    Ok(buf)
}

/// Printable ASCII up to the first NUL, trimmed.
fn ascii(bytes: &[u8]) -> String {
    bytes
        .iter()
        .take_while(|b| **b != 0)
        .filter(|b| b.is_ascii_graphic() || **b == b' ')
        .map(|b| char::from(*b))
        .collect::<String>()
        .trim()
        .to_owned()
}

/// A 4-character game code, when it is one (homebrew often has none).
fn code(bytes: &[u8]) -> Option<String> {
    bytes
        .iter()
        .all(u8::is_ascii_alphanumeric)
        .then(|| ascii(bytes))
}

/// UTF-16LE up to the first NUL.
fn utf16(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|u| *u != 0)
        .collect();
    String::from_utf16_lossy(&units)
}

fn le16(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn le32(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn le64(b: &[u8], at: usize) -> u64 {
    u64::from(le32(b, at)) | (u64::from(le32(b, at + 4)) << 32)
}

/// A 5-bit channel (low bits of `c`) to 8 bits.
fn five(c: u16) -> u8 {
    expand5(c & 0x1F)
}

fn expand5(v: u16) -> u8 {
    ((v << 3) | (v >> 2)) as u8
}

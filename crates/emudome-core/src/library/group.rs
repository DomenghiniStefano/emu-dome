//! The regional versions of one game, grouped (Trello #102). Identity stays the content: each dump
//! keeps its own saves, settings and play time (docs/decisions.md, «different regions are
//! different games»); the library shows one tile per family while `library.versions` is on, and
//! the game's details list the versions.
//!
//! # Which games are one family
//!
//! Same console and the same base game code, the code the cartridge's maker gave every regional
//! version of a game ([`family_key`]):
//! - GBA and NDS: the 4-letter header code without its last letter, the region (`BPEE` / `BPEJ`
//!   are one game, `BPRE` another);
//! - 3DS: the product code without its region letter (`CTR-P-AJRE` / `CTR-P-AJRP`), its platform
//!   kept (`CTR` and `KTR`, New 3DS only, are never one game);
//! - never grouped: a game without a code, a code that is not a maker's (homebrew writes none,
//!   `####`, digits, or the tools' defaults: PassMe's `PASS` on DS, makerom's `CTAP` on 3DS), a
//!   region letter [`super::region`] does not know, Game Boy / Game Boy Color (their header has a
//!   title, a licensee and a Japan / overseas flag, which EmuDome does not read: a title alone
//!   also matches hacks and other games, so nothing is grouped rather than something wrongly),
//!   and a game the user hid (it shows on its own when hidden games are shown).
//!
//! Two dumps with the same code and region (revisions, or a good and a bad dump) are one family
//! too.
//!
//! # Which version is shown (owner, 2026-11-22: no region or language rule)
//!
//! 1. The version the user pinned («Mostra questa in libreria», [`Pins`]): it always wins (two
//!    pins in one family, from a file written elsewhere: the one the next rules put first);
//! 2. else the version launched last (so the tile's «Gioca» resumes the game being played);
//! 3. else, in a family never launched, the version added to the library first;
//! 4. then the content id, so two equal ones never swap.
//!
//! The other versions follow in the same order. Pure: the caller passes the games and the pins;
//! the library's store keeps the pins (`Library::pin_version`).

use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use ts_rs::TS;

use super::details::region;
use super::store::GameRecord;
use crate::settings::ConsoleId;
use crate::GameKey;

/// What the grouping reads of a game.
#[derive(Clone, Copy, Debug)]
pub struct Member<'a> {
    pub id: &'a GameKey,
    pub console: ConsoleId,
    pub code: Option<&'a str>,
    /// When it was last launched, Unix ms.
    pub last_played: Option<u64>,
    /// When it joined the library, Unix ms.
    pub added_at: u64,
    /// The user hid it: never grouped.
    pub hidden: bool,
}

impl<'a> From<&'a GameRecord> for Member<'a> {
    fn from(g: &'a GameRecord) -> Self {
        Self {
            id: &g.id,
            console: g.console,
            code: g.code.as_deref(),
            last_played: g.last_played,
            added_at: g.added_at,
            hidden: g.hidden,
        }
    }
}

/// The versions the user pinned («Mostra questa in libreria»), by content id.
pub type Pins = BTreeSet<GameKey>;

/// One game in several versions (two or more). TypeScript: `GameFamily` (generated).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "GameFamily")]
pub struct Family {
    /// [`family_key`]: `<console>:<base code>` (`gba:BPE`, `3ds:CTR-AJR`).
    pub id: String,
    /// The version the library shows, and its tile starts.
    pub shown: GameKey,
    /// Every version, the shown one first, then by the same order.
    pub versions: Vec<GameKey>,
    /// `shown` is the user's pin.
    pub pinned: bool,
}

/// The DS tools' default code (PassMe), written into homebrew: never a game's.
const DS_PLACEHOLDERS: &[&str] = &["PASS"];
/// makerom's default product code, written into homebrew `.cia` / `.3ds`: never a game's.
const N3DS_PLACEHOLDERS: &[&str] = &["CTAP"];

/// A maker's 4-character game code: uppercase letters and digits, a letter first, a region
/// letter last that [`region`] knows, and not one of the tools' `placeholders`.
fn maker_code(code: &str, placeholders: &[&str]) -> bool {
    let b = code.as_bytes();
    b.len() == 4
        && b[0].is_ascii_uppercase()
        && b.iter()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        && region(Some(code)).is_some()
        && !placeholders.contains(&code)
}

/// The family a game belongs to, from its console and header code; `None`: never grouped.
pub fn family_key(console: ConsoleId, code: Option<&str>) -> Option<String> {
    let code = code?.trim();
    match console {
        ConsoleId::Gba => maker_code(code, &[]).then(|| format!("gba:{}", &code[..3])),
        ConsoleId::Nds => maker_code(code, DS_PLACEHOLDERS).then(|| format!("nds:{}", &code[..3])),
        ConsoleId::N3ds => {
            let mut parts = code.split('-');
            let (platform, _kind, game) = (parts.next()?, parts.next()?, parts.next()?);
            let ok = parts.next().is_none()
                && matches!(platform, "CTR" | "KTR")
                && maker_code(game, N3DS_PLACEHOLDERS);
            ok.then(|| format!("3ds:{platform}-{}", &game[..3]))
        }
        ConsoleId::Gb | ConsoleId::Gbc => None,
    }
}

/// The families among `games`: only those with two or more versions, in the order of their ids.
pub fn families<'a>(games: impl IntoIterator<Item = Member<'a>>, pins: &Pins) -> Vec<Family> {
    let mut by_key: BTreeMap<String, Vec<Member<'a>>> = BTreeMap::new();
    for g in games.into_iter().filter(|g| !g.hidden) {
        if let Some(key) = family_key(g.console, g.code) {
            by_key.entry(key).or_default().push(g);
        }
    }
    by_key
        .into_iter()
        .filter(|(_, members)| members.len() > 1)
        .map(|(id, mut members)| {
            members.sort_by(before);
            let pin = members.iter().position(|m| pins.contains(m.id));
            if let Some(i) = pin {
                let pinned = members.remove(i);
                members.insert(0, pinned);
            }
            Family {
                id,
                shown: members[0].id.clone(),
                versions: members.iter().map(|m| m.id.clone()).collect(),
                pinned: pin.is_some(),
            }
        })
        .collect()
}

/// `a` before `b`: launched more recently; never launched after any launched one, the one
/// added first among those; then the content id.
fn before(a: &Member<'_>, b: &Member<'_>) -> Ordering {
    let key = |m: &Member<'_>| {
        (
            m.last_played.is_none(),
            Reverse(m.last_played.unwrap_or_default()),
            m.added_at,
        )
    };
    key(a).cmp(&key(b)).then_with(|| a.id.cmp(b.id))
}

#[cfg(test)]
mod tests {
    use super::{families, family_key, Member, Pins};
    use crate::settings::ConsoleId;
    use crate::GameKey;

    fn key(console: ConsoleId, n: u8) -> GameKey {
        GameKey::new(console, &format!("{n:02x}")).expect("key")
    }

    /// Games: (number, console, code, last played, added at).
    type Game = (u8, ConsoleId, Option<&'static str>, Option<u64>, u64);

    fn run(games: &[Game], pins: &[GameKey]) -> Vec<(String, GameKey, Vec<GameKey>, bool)> {
        let keys: Vec<GameKey> = games.iter().map(|g| key(g.1, g.0)).collect();
        let members = games.iter().zip(&keys).map(|(g, id)| Member {
            id,
            console: g.1,
            code: g.2,
            last_played: g.3,
            added_at: g.4,
            hidden: false,
        });
        let pins: Pins = pins.iter().cloned().collect();
        families(members, &pins)
            .into_iter()
            .map(|f| (f.id, f.shown, f.versions, f.pinned))
            .collect()
    }

    const GBA: ConsoleId = ConsoleId::Gba;
    const NDS: ConsoleId = ConsoleId::Nds;
    const N3DS: ConsoleId = ConsoleId::N3ds;

    #[test]
    fn the_family_is_the_console_and_the_code_without_its_region() {
        assert_eq!(family_key(GBA, Some("BPEE")).as_deref(), Some("gba:BPE"));
        assert_eq!(family_key(GBA, Some("BPEJ")).as_deref(), Some("gba:BPE"));
        assert_eq!(family_key(NDS, Some("ADAE")).as_deref(), Some("nds:ADA"));
        // A tool's default is refused only on the console whose tools write it.
        assert_eq!(family_key(GBA, Some("CTAP")).as_deref(), Some("gba:CTA"));
        assert_eq!(
            family_key(N3DS, Some("CTR-P-AJRE")).as_deref(),
            Some("3ds:CTR-AJR")
        );
        assert_eq!(
            family_key(N3DS, Some("CTR-N-AJRP")).as_deref(),
            Some("3ds:CTR-AJR"),
            "a card and a download of one game"
        );
        assert_eq!(
            family_key(N3DS, Some("KTR-P-AJRE")).as_deref(),
            Some("3ds:KTR-AJR"),
            "New 3DS only is another platform"
        );
    }

    #[test]
    fn homebrew_and_unknown_codes_are_never_grouped() {
        for (console, code) in [
            (GBA, None),
            (GBA, Some("")),
            (GBA, Some("0000")),
            (GBA, Some("1ABE")),
            (GBA, Some("BPE")),
            (GBA, Some("BPEEE")),
            (GBA, Some("bpee")),
            (GBA, Some("BPEQ")), // no region has Q
            (NDS, Some("PASS")),
            (NDS, Some("####")),
            (N3DS, Some("CTR-P-CTAP")),
            (N3DS, Some("ABCE")),
            (N3DS, Some("XYZ-P-AJRE")),
            (N3DS, Some("CTR-P-AJRE-1")),
            (ConsoleId::Gb, Some("BPEE")),
            (ConsoleId::Gbc, Some("BPEE")),
        ] {
            assert_eq!(family_key(console, code), None, "{console:?} {code:?}");
        }
    }

    #[test]
    fn the_version_launched_last_is_shown_and_starts() {
        let games = [
            (1, GBA, Some("BPEJ"), Some(300), 1),
            (2, GBA, Some("BPEE"), Some(900), 2),
            (3, GBA, Some("BPEP"), None, 0),
            (4, GBA, Some("BPRE"), Some(999), 5), // another game, alone
        ];
        let k = |n| key(GBA, n);
        assert_eq!(
            run(&games, &[]),
            [("gba:BPE".into(), k(2), vec![k(2), k(1), k(3)], false)]
        );
    }

    #[test]
    fn a_family_never_launched_shows_the_version_added_first() {
        let games = [
            (1, NDS, Some("ADAJ"), None, 30),
            (2, NDS, Some("ADAE"), None, 10),
            (3, NDS, Some("ADAP"), None, 20),
        ];
        let k = |n| key(NDS, n);
        assert_eq!(run(&games, &[])[0].2, [k(2), k(3), k(1)]);
        // Added at the same time (one scan): the content id decides, always the same way.
        let twins = [
            (9, NDS, Some("ADAE"), None, 10),
            (8, NDS, Some("ADAP"), None, 10),
        ];
        assert_eq!(run(&twins, &[])[0].2, [key(NDS, 8), key(NDS, 9)]);
    }

    #[test]
    fn two_dumps_of_one_region_are_one_family() {
        let games = [
            (1, GBA, Some("AXVE"), Some(5), 0),
            (2, GBA, Some("AXVE"), Some(7), 0),
        ];
        let k = |n| key(GBA, n);
        assert_eq!(
            run(&games, &[]),
            [("gba:AXV".into(), k(2), vec![k(2), k(1)], false)]
        );
    }

    #[test]
    fn a_3ds_family_by_its_product_code() {
        let games = [
            (1, N3DS, Some("CTR-P-AJRJ"), None, 1),
            (2, N3DS, Some("CTR-P-AJRP"), None, 2),
            (3, N3DS, Some("KTR-P-AJRP"), None, 0),
        ];
        let k = |n| key(N3DS, n);
        assert_eq!(
            run(&games, &[]),
            [("3ds:CTR-AJR".into(), k(1), vec![k(1), k(2)], false)]
        );
    }

    #[test]
    fn the_same_code_on_two_consoles_is_two_games() {
        let games = [
            (1, GBA, Some("ABCE"), None, 0),
            (2, NDS, Some("ABCP"), None, 0),
        ];
        assert!(run(&games, &[]).is_empty());
    }

    #[test]
    fn a_pinned_version_always_wins() {
        let games = [
            (1, GBA, Some("BPEJ"), None, 9),
            (2, GBA, Some("BPEE"), Some(900), 2),
            (3, GBA, Some("BPEP"), Some(100), 1),
        ];
        let k = |n| key(GBA, n);
        assert_eq!(
            run(&games, &[k(1)]),
            [("gba:BPE".into(), k(1), vec![k(1), k(2), k(3)], true)]
        );
        // A pin of another family, or of a game that is not in one, changes nothing here.
        assert_eq!(run(&games, &[key(GBA, 9)])[0].1, k(2));
        // Two pins in one family (a file written elsewhere): the one launched last wins.
        assert_eq!(run(&games, &[k(1), k(3)])[0].1, k(3));
    }

    #[test]
    fn a_hidden_game_is_never_grouped() {
        let a = key(GBA, 1);
        let b = key(GBA, 2);
        let c = key(GBA, 3);
        let m = |id, code, hidden| Member {
            id,
            console: GBA,
            code: Some(code),
            last_played: None,
            added_at: 0,
            hidden,
        };
        let out = families(
            [
                m(&a, "BPEE", false),
                m(&b, "BPEP", true),
                m(&c, "BPEJ", false),
            ],
            &Pins::new(),
        );
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].versions, [a.clone(), c.clone()]);
        let alone = families([m(&a, "BPEE", false), m(&b, "BPEP", true)], &Pins::new());
        assert!(alone.is_empty());
    }

    #[test]
    fn ten_thousand_games_group_in_one_pass() {
        // The scale scenario's size (library-scale: 10 005 games): one call, once per change.
        let keys: Vec<GameKey> = (0..10_000u32)
            .map(|n| GameKey::new(GBA, &format!("{n:08x}")).expect("key"))
            .collect();
        // Two regions of 5 000 games: `AAAE` and `AAAP`, `AABE` and `AABP`…
        let codes: Vec<String> = (0..10_000u32)
            .map(|n| {
                let base = n / 2;
                let letter = |i: u32| char::from(b'A' + u8::try_from(i % 26).unwrap_or(0));
                let r = if n % 2 == 0 { 'E' } else { 'P' };
                format!(
                    "{}{}{}{r}",
                    letter(base / 676),
                    letter(base / 26),
                    letter(base)
                )
            })
            .collect();
        let started = std::time::Instant::now();
        let out = families(
            keys.iter().zip(&codes).map(|(id, code)| Member {
                id,
                console: GBA,
                code: Some(code),
                last_played: None,
                added_at: 0,
                hidden: false,
            }),
            &Pins::new(),
        );
        assert_eq!(out.len(), 5_000);
        assert!(out.iter().all(|f| f.versions.len() == 2));
        assert!(started.elapsed().as_millis() < 2_000);
    }
}

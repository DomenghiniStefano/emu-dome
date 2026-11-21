//! The «in use on <device>» marker: a game being played on one device, told to the others. It
//! never blocks: the interface says the game is open elsewhere and the user plays on, the
//! conflict rule catching what both devices then change.
//!
//! Kept as `appProperties` of the game's Drive folder (`EmuDome/<console>/<hex>/`): the device
//! id and name, when the game started there and the last heartbeat, refreshed every
//! [`HEARTBEAT`] while it runs and removed when it closes. A marker whose heartbeat is older
//! than [`STALE_AFTER`] is a device that stopped without closing (crash, battery, no network)
//! and counts as free.

use std::collections::BTreeMap;

use serde::Serialize;
use ts_rs::TS;

use crate::drive::Properties;

/// How often a device playing the game refreshes its marker, ms.
pub const HEARTBEAT: u64 = 2 * 60 * 1000;
/// A heartbeat older than this is stale, ms (several missed heartbeats).
pub const STALE_AFTER: u64 = 10 * 60 * 1000;

const DEVICE: &str = "inUseDevice";
const NAME: &str = "inUseName";
const SINCE: &str = "inUseSince";
const BEAT: &str = "inUseBeat";

/// A marker as stored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Marker {
    pub device: String,
    pub name: String,
    /// Unix ms.
    pub since: u64,
    /// Unix ms of the last heartbeat.
    pub beat: u64,
}

impl Marker {
    /// The marker in a folder's `appProperties`, if it has a whole one.
    pub fn read(props: &Properties) -> Option<Self> {
        Some(Self {
            device: props.get(DEVICE)?.clone(),
            name: props.get(NAME).cloned().unwrap_or_default(),
            since: props.get(SINCE)?.parse().ok()?,
            beat: props.get(BEAT)?.parse().ok()?,
        })
    }

    /// The `appProperties` change that writes it.
    pub fn set(&self) -> BTreeMap<String, Option<String>> {
        BTreeMap::from([
            (DEVICE.to_owned(), Some(self.device.clone())),
            (NAME.to_owned(), Some(self.name.clone())),
            (SINCE.to_owned(), Some(self.since.to_string())),
            (BEAT.to_owned(), Some(self.beat.to_string())),
        ])
    }

    /// The `appProperties` change that removes it.
    pub fn clear() -> BTreeMap<String, Option<String>> {
        [DEVICE, NAME, SINCE, BEAT]
            .into_iter()
            .map(|k| (k.to_owned(), None))
            .collect()
    }
}

/// Whether a game is in use, for this device. TypeScript: `CloudInUse`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(rename = "CloudInUse")]
pub enum InUse {
    /// Nobody plays it (or the last device stopped answering).
    Free,
    /// This device holds the marker.
    Here,
    /// Open on another device: `name`, since `since` (Unix ms), last heard of at `beat`.
    Elsewhere { name: String, since: u64, beat: u64 },
}

/// What `marker` means for `device` at `now` (Unix ms).
pub fn in_use(marker: Option<&Marker>, device: &str, now: u64) -> InUse {
    match marker {
        None => InUse::Free,
        Some(m) if m.device == device => InUse::Here,
        // A heartbeat ahead of this clock (clocks differ) is fresh, never stale.
        Some(m) if now.saturating_sub(m.beat) > STALE_AFTER => InUse::Free,
        Some(m) => InUse::Elsewhere {
            name: m.name.clone(),
            since: m.since,
            beat: m.beat,
        },
    }
}

/// Whether this device should write its heartbeat now (its marker last written at `beat`).
pub fn heartbeat_due(beat: u64, now: u64) -> bool {
    now.saturating_sub(beat) >= HEARTBEAT
}

#[cfg(test)]
mod tests {
    use super::{heartbeat_due, in_use, InUse, Marker, HEARTBEAT, STALE_AFTER};
    use crate::drive::Properties;

    fn marker(device: &str, beat: u64) -> Marker {
        Marker {
            device: device.into(),
            name: "PC-SALOTTO".into(),
            since: 100,
            beat,
        }
    }

    #[test]
    fn reads_what_it_writes_and_clears_every_key() {
        let m = marker("dev-a", 5_000);
        let props: Properties = m
            .set()
            .into_iter()
            .filter_map(|(k, v)| v.map(|v| (k, v)))
            .collect();
        assert_eq!(Marker::read(&props), Some(m.clone()));
        assert_eq!(
            Marker::clear().keys().collect::<Vec<_>>(),
            m.set().keys().collect::<Vec<_>>()
        );
        assert!(Marker::clear().values().all(Option::is_none));
        let mut broken = props.clone();
        broken.insert("inUseBeat".into(), "soon".into());
        assert_eq!(Marker::read(&broken), None);
        assert_eq!(Marker::read(&Properties::new()), None);
    }

    #[test]
    fn says_who_plays_and_forgets_a_silent_device() {
        let now = 1_000_000_000;
        assert_eq!(in_use(None, "me", now), InUse::Free);
        assert_eq!(in_use(Some(&marker("me", 0)), "me", now), InUse::Here);
        assert_eq!(
            in_use(Some(&marker("other", now - STALE_AFTER)), "me", now),
            InUse::Elsewhere {
                name: "PC-SALOTTO".into(),
                since: 100,
                beat: now - STALE_AFTER
            }
        );
        assert_eq!(
            in_use(Some(&marker("other", now - STALE_AFTER - 1)), "me", now),
            InUse::Free
        );
        // The other device's clock is ahead.
        assert!(matches!(
            in_use(Some(&marker("other", now + 60_000)), "me", now),
            InUse::Elsewhere { .. }
        ));
        assert!(!heartbeat_due(now, now + HEARTBEAT - 1));
        assert!(heartbeat_due(now, now + HEARTBEAT));
        const {
            assert!(
                STALE_AFTER >= 3 * HEARTBEAT,
                "a few missed beats before stale"
            )
        };
    }
}

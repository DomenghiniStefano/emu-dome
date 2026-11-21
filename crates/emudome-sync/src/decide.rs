//! What to do with one file: the conflict rule (docs/decisions.md, «Product»: «conflict =
//! changed on both sides since the last sync → ask»), as a pure function of three checksums.
//!
//! - `local`: the MD5 of the file on this device, `None` if there is none;
//! - `remote`: Drive's `md5Checksum`, `None` if Drive has no such file;
//! - `base`: the MD5 both sides had at the last sync (the sync index), `None` if never synced.
//!
//! A side changed since the last sync when it differs from the base (with no base, any side that
//! exists counts as changed). Then: equal sides are in sync; one changed side wins; two changed
//! sides that differ are a conflict, left as they are for the user to decide. Nothing is ever
//! deleted: a file missing on one side is copied back from the other (a deletion does not
//! travel), so the worst a deletion costs is the file coming back.

/// The decision for one file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    /// Both sides hold the same content (or neither has the file): nothing to send. The index
    /// records the content as synced.
    InSync,
    /// This device's copy goes to Drive.
    Push,
    /// Drive's copy comes to this device.
    Pull,
    /// Both sides changed since the last sync and differ: the user chooses, and the side not
    /// chosen is kept as a copy (`Syncer::resolve`).
    Conflict,
}

/// The decision for a file whose local and remote checksums (and last synced one) are these.
pub fn decide(local: Option<&str>, remote: Option<&str>, base: Option<&str>) -> Decision {
    match (local, remote) {
        (None, None) => Decision::InSync,
        (Some(_), None) => Decision::Push,
        (None, Some(_)) => Decision::Pull,
        (Some(l), Some(r)) if l == r => Decision::InSync,
        (Some(l), Some(r)) => {
            let local_changed = base != Some(l);
            let remote_changed = base != Some(r);
            match (local_changed, remote_changed) {
                (true, false) => Decision::Push,
                (false, true) => Decision::Pull,
                // (false, false) cannot happen with l != r: both equal to one base.
                (true, true) | (false, false) => Decision::Conflict,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{decide, Decision};

    /// Every combination of three sides over four contents (absent, A, B, C).
    #[test]
    fn every_combination_keeps_both_sides_safe() {
        let values = [None, Some("a"), Some("b"), Some("c")];
        for local in values {
            for remote in values {
                for base in values {
                    let d = decide(local, remote, base);
                    let why = format!("local {local:?} remote {remote:?} base {base:?}: {d:?}");
                    match d {
                        // A push replaces Drive's copy: only one nobody changed since the last
                        // sync, or none.
                        Decision::Push => {
                            assert!(local.is_some(), "{why}");
                            assert!(remote.is_none() || remote == base, "{why}");
                        }
                        // A pull replaces this device's copy: only one not changed here.
                        Decision::Pull => {
                            assert!(remote.is_some(), "{why}");
                            assert!(local.is_none() || local == base, "{why}");
                        }
                        Decision::InSync => assert_eq!(local, remote, "{why}"),
                        // Asked only when both changed and differ.
                        Decision::Conflict => {
                            assert!(local.is_some() && remote.is_some(), "{why}");
                            assert_ne!(local, remote, "{why}");
                            assert_ne!(local, base, "{why}");
                            assert_ne!(remote, base, "{why}");
                        }
                    }
                    // Two different contents are never left as they are without asking.
                    if local.is_some() && remote.is_some() && local != remote {
                        assert_ne!(d, Decision::InSync, "{why}");
                    }
                }
            }
        }
    }

    #[test]
    fn the_cases_by_name() {
        let a = Some("a");
        let b = Some("b");
        let c = Some("c");
        // Played here since the last sync.
        assert_eq!(decide(b, a, a), Decision::Push);
        // Played on the other device.
        assert_eq!(decide(a, b, a), Decision::Pull);
        // Played on both: ask.
        assert_eq!(decide(b, c, a), Decision::Conflict);
        // Never synced, two different saves (two devices before the first sync): ask.
        assert_eq!(decide(a, b, None), Decision::Conflict);
        // Never synced, the same save: nothing to do.
        assert_eq!(decide(a, a, None), Decision::InSync);
        // Both ended with the same content.
        assert_eq!(decide(b, b, a), Decision::InSync);
        // New here; new on Drive.
        assert_eq!(decide(a, None, None), Decision::Push);
        assert_eq!(decide(None, a, None), Decision::Pull);
        // Deleted here after a sync: it comes back, never deleted on Drive.
        assert_eq!(decide(None, a, a), Decision::Pull);
        // Deleted on Drive after a sync: uploaded again.
        assert_eq!(decide(a, None, a), Decision::Push);
        // An index entry that matches neither side (a sync interrupted elsewhere): ask.
        assert_eq!(decide(a, b, c), Decision::Conflict);
    }
}

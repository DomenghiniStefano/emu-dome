//! Updates of EmuDome itself (Trello #44): when to look for a new version, what to do with one,
//! and what the interface shows, as a pure state machine. Every time is passed in (ms since the
//! Unix epoch); the shell (`app/src-tauri/src/updates.rs`) carries out the actions it returns
//! (the network, the installer) and reports back.
//!
//! `general.updates` (per device, desktop only) decides:
//! - **auto**: look at start and every [`CHECK_EVERY_MS`], download a new version in the
//!   background, then offer it (X06 «Aggiornamento disponibile»: restart to install);
//! - **ask**: look the same way and offer it; the download starts only on «Riavvia e aggiorna»;
//! - **off**: never look by itself; «Controlla ora» (`about.upd`) still looks once.
//!
//! A build without the update key, and Android (updates from Google Play), cannot update itself:
//! the state is [`UpdateState::Unavailable`] and every request answers
//! [`ErrorCode::UpdateUnavailable`].

use serde::Serialize;
use ts_rs::TS;

use crate::ErrorCode;

/// How often an automatic check runs while EmuDome stays open.
pub const CHECK_EVERY_MS: u64 = 6 * 60 * 60 * 1000;
/// How soon an automatic check runs again after one that failed (offline at start).
pub const RETRY_AFTER_MS: u64 = 30 * 60 * 1000;

/// `general.updates`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UpdatePolicy {
    Auto,
    /// The catalogue's default.
    #[default]
    Ask,
    Off,
}

impl UpdatePolicy {
    /// The policy of an option id of `general.updates`; `None` for an unknown one.
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "auto" => Some(Self::Auto),
            "ask" => Some(Self::Ask),
            "off" => Some(Self::Off),
            _ => None,
        }
    }
}

/// Where the update stands. TypeScript: `UpdateState` (generated).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum UpdateState {
    /// This build cannot update itself ([`ErrorCode::UpdateUnavailable`]).
    Unavailable,
    /// Not looked yet in this run.
    Idle,
    Checking,
    /// The last check found nothing newer.
    UpToDate,
    /// A newer version was found and not downloaded (`ask`, or `off` after «Controlla ora»).
    Available {
        version: String,
    },
    Downloading {
        version: String,
    },
    /// Downloaded and verified: restarting installs it.
    Ready {
        version: String,
    },
    /// The installer is starting; EmuDome closes for it.
    Installing {
        version: String,
    },
    /// The last check, download or install failed.
    Failed {
        error: ErrorCode,
    },
}

/// What the interface shows. TypeScript: `UpdateStatus` (generated).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub state: UpdateState,
    /// When the last check ended, ms since the Unix epoch.
    #[ts(type = "number | null")]
    pub checked_at: Option<u64>,
    /// X06 shows: a version is ready to install (downloaded with `auto`, found with `ask`) and
    /// was not put off with «Più tardi» in this run.
    pub prompt: bool,
}

/// What the shell does next.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateAction {
    /// Ask the release server for a newer version.
    Check,
    /// Download and verify `version`.
    Download { version: String },
    /// Start the installer of the downloaded `version` (EmuDome exits or restarts).
    Install { version: String },
}

/// The state machine. One per run, in memory: nothing about updates is stored.
#[derive(Debug, Clone)]
pub struct UpdateFlow {
    configured: bool,
    policy: UpdatePolicy,
    state: UpdateState,
    /// When the last check started (for the next automatic one).
    last_try: Option<u64>,
    checked_at: Option<u64>,
    /// The version «Più tardi» put off.
    later: Option<String>,
    /// «Riavvia e aggiorna» while the download runs: install once it is done.
    install_wanted: bool,
}

impl UpdateFlow {
    /// `configured`: this build can update itself (the update key is in, not Android).
    pub fn new(configured: bool, policy: UpdatePolicy) -> Self {
        Self {
            configured,
            policy,
            state: if configured {
                UpdateState::Idle
            } else {
                UpdateState::Unavailable
            },
            last_try: None,
            checked_at: None,
            later: None,
            install_wanted: false,
        }
    }

    pub fn status(&self) -> UpdateStatus {
        let offered = match &self.state {
            UpdateState::Ready { version } | UpdateState::Available { version } => Some(version),
            UpdateState::Unavailable
            | UpdateState::Idle
            | UpdateState::Checking
            | UpdateState::UpToDate
            | UpdateState::Downloading { .. }
            | UpdateState::Installing { .. }
            | UpdateState::Failed { .. } => None,
        };
        UpdateStatus {
            state: self.state.clone(),
            checked_at: self.checked_at,
            prompt: offered.is_some_and(|v| self.later.as_ref() != Some(v)),
        }
    }

    /// The version X06 offers now, if any.
    pub fn offered(&self) -> Option<&str> {
        match &self.state {
            UpdateState::Ready { version } | UpdateState::Available { version } => {
                Some(version.as_str())
            }
            UpdateState::Unavailable
            | UpdateState::Idle
            | UpdateState::Checking
            | UpdateState::UpToDate
            | UpdateState::Downloading { .. }
            | UpdateState::Installing { .. }
            | UpdateState::Failed { .. } => None,
        }
    }

    pub fn policy(&self) -> UpdatePolicy {
        self.policy
    }

    /// `general.updates` changed. Nothing is undone: a version already found stays offered.
    pub fn set_policy(&mut self, policy: UpdatePolicy) {
        self.policy = policy;
    }

    fn busy(&self) -> bool {
        matches!(
            self.state,
            UpdateState::Checking
                | UpdateState::Downloading { .. }
                | UpdateState::Installing { .. }
        )
    }

    /// Called now and then: the automatic check, when its time has come.
    pub fn tick(&mut self, now: u64) -> Option<UpdateAction> {
        if !self.configured || self.policy == UpdatePolicy::Off || self.busy() {
            return None;
        }
        let wait = match self.state {
            // A version is waiting: nothing to look for until it is installed.
            UpdateState::Available { .. } | UpdateState::Ready { .. } => return None,
            UpdateState::Failed { .. } => RETRY_AFTER_MS,
            UpdateState::Unavailable
            | UpdateState::Idle
            | UpdateState::Checking
            | UpdateState::UpToDate
            | UpdateState::Downloading { .. }
            | UpdateState::Installing { .. } => CHECK_EVERY_MS,
        };
        if self
            .last_try
            .is_some_and(|last| now.saturating_sub(last) < wait)
        {
            return None;
        }
        self.start_check(now)
    }

    fn start_check(&mut self, now: u64) -> Option<UpdateAction> {
        self.last_try = Some(now);
        self.state = UpdateState::Checking;
        Some(UpdateAction::Check)
    }

    /// «Controlla ora» (`about.upd`): looks whatever the policy, unless something is running.
    pub fn check_now(&mut self, now: u64) -> Result<Option<UpdateAction>, ErrorCode> {
        if !self.configured {
            return Err(ErrorCode::UpdateUnavailable);
        }
        if self.busy() {
            return Ok(None);
        }
        Ok(self.start_check(now))
    }

    /// The check ended: `Ok(Some(version))` a newer version, `Ok(None)` nothing newer.
    pub fn checked(
        &mut self,
        found: Result<Option<String>, ErrorCode>,
        now: u64,
    ) -> Option<UpdateAction> {
        match found {
            Ok(Some(version)) => {
                self.checked_at = Some(now);
                if self.policy == UpdatePolicy::Auto {
                    self.state = UpdateState::Downloading {
                        version: version.clone(),
                    };
                    return Some(UpdateAction::Download { version });
                }
                self.state = UpdateState::Available { version };
            }
            Ok(None) => {
                self.checked_at = Some(now);
                self.state = UpdateState::UpToDate;
            }
            Err(error) => self.state = UpdateState::Failed { error },
        }
        None
    }

    /// The download of `version` ended (verified when `Ok`).
    pub fn downloaded(
        &mut self,
        version: String,
        result: Result<(), ErrorCode>,
    ) -> Option<UpdateAction> {
        let install = std::mem::take(&mut self.install_wanted);
        match result {
            Ok(()) if install => {
                self.state = UpdateState::Installing {
                    version: version.clone(),
                };
                Some(UpdateAction::Install { version })
            }
            Ok(()) => {
                self.state = UpdateState::Ready { version };
                None
            }
            Err(error) => {
                self.state = UpdateState::Failed { error };
                None
            }
        }
    }

    /// «Riavvia e aggiorna»: installs what is downloaded, or downloads first (`ask`).
    pub fn install(&mut self) -> Result<Option<UpdateAction>, ErrorCode> {
        if !self.configured {
            return Err(ErrorCode::UpdateUnavailable);
        }
        match &self.state {
            UpdateState::Ready { version } => {
                let version = version.clone();
                self.state = UpdateState::Installing {
                    version: version.clone(),
                };
                Ok(Some(UpdateAction::Install { version }))
            }
            UpdateState::Available { version } => {
                let version = version.clone();
                self.install_wanted = true;
                self.state = UpdateState::Downloading {
                    version: version.clone(),
                };
                Ok(Some(UpdateAction::Download { version }))
            }
            UpdateState::Downloading { .. } => {
                self.install_wanted = true;
                Ok(None)
            }
            UpdateState::Installing { .. } => Ok(None),
            UpdateState::Unavailable
            | UpdateState::Idle
            | UpdateState::Checking
            | UpdateState::UpToDate
            | UpdateState::Failed { .. } => Err(ErrorCode::UpdateNotReady),
        }
    }

    /// The installer of `version` could not start: the version stays downloaded, offered again.
    pub fn install_failed(&mut self, version: String, error: ErrorCode) {
        log_failure(&error);
        self.state = UpdateState::Ready { version };
        self.later = None;
    }

    /// «Più tardi»: X06 hides for this version until EmuDome starts again.
    pub fn later(&mut self) {
        self.later = self.offered().map(str::to_owned);
    }
}

fn log_failure(error: &ErrorCode) {
    log::warn!("update not installed: {error:?}");
}

#[cfg(test)]
mod tests {
    use super::{
        UpdateAction, UpdateFlow, UpdatePolicy, UpdateState, CHECK_EVERY_MS, RETRY_AFTER_MS,
    };
    use crate::ErrorCode;

    const T0: u64 = 1_000_000;

    fn v(s: &str) -> String {
        s.to_owned()
    }

    #[test]
    fn the_policy_reads_the_catalogue_options() {
        assert_eq!(UpdatePolicy::from_id("auto"), Some(UpdatePolicy::Auto));
        assert_eq!(UpdatePolicy::from_id("ask"), Some(UpdatePolicy::Ask));
        assert_eq!(UpdatePolicy::from_id("off"), Some(UpdatePolicy::Off));
        assert_eq!(UpdatePolicy::from_id("weekly"), None);
        assert_eq!(UpdatePolicy::default(), UpdatePolicy::Ask);
    }

    #[test]
    fn a_build_without_the_key_never_checks_and_says_why() {
        let mut f = UpdateFlow::new(false, UpdatePolicy::Auto);
        assert_eq!(f.status().state, UpdateState::Unavailable);
        assert_eq!(f.tick(T0), None);
        assert_eq!(f.check_now(T0), Err(ErrorCode::UpdateUnavailable));
        assert_eq!(f.install(), Err(ErrorCode::UpdateUnavailable));
        assert!(!f.status().prompt);
    }

    #[test]
    fn auto_checks_at_start_downloads_then_offers_the_restart() {
        let mut f = UpdateFlow::new(true, UpdatePolicy::Auto);
        assert_eq!(f.status().state, UpdateState::Idle);
        assert_eq!(f.tick(T0), Some(UpdateAction::Check));
        assert_eq!(f.status().state, UpdateState::Checking);
        // No second check while one runs.
        assert_eq!(f.tick(T0 + 1), None);
        assert_eq!(
            f.checked(Ok(Some(v("0.0.2"))), T0 + 10),
            Some(UpdateAction::Download {
                version: v("0.0.2")
            })
        );
        assert!(!f.status().prompt, "not offered while it downloads");
        assert_eq!(f.downloaded(v("0.0.2"), Ok(())), None);
        let s = f.status();
        assert_eq!(
            s.state,
            UpdateState::Ready {
                version: v("0.0.2")
            }
        );
        assert!(s.prompt);
        assert_eq!(s.checked_at, Some(T0 + 10));
        assert_eq!(
            f.install(),
            Ok(Some(UpdateAction::Install {
                version: v("0.0.2")
            }))
        );
        assert!(!f.status().prompt);
    }

    #[test]
    fn ask_offers_first_and_downloads_on_restart_and_update() {
        let mut f = UpdateFlow::new(true, UpdatePolicy::Ask);
        assert_eq!(f.tick(T0), Some(UpdateAction::Check));
        assert_eq!(f.checked(Ok(Some(v("0.1.0"))), T0), None);
        assert!(f.status().prompt);
        assert_eq!(
            f.install(),
            Ok(Some(UpdateAction::Download {
                version: v("0.1.0")
            }))
        );
        // Pressed again while it downloads: nothing more.
        assert_eq!(f.install(), Ok(None));
        assert_eq!(
            f.downloaded(v("0.1.0"), Ok(())),
            Some(UpdateAction::Install {
                version: v("0.1.0")
            })
        );
        assert_eq!(
            f.status().state,
            UpdateState::Installing {
                version: v("0.1.0")
            }
        );
    }

    #[test]
    fn off_never_checks_by_itself_but_check_now_does() {
        let mut f = UpdateFlow::new(true, UpdatePolicy::Off);
        assert_eq!(f.tick(T0), None);
        assert_eq!(f.tick(T0 + CHECK_EVERY_MS * 3), None);
        assert_eq!(f.check_now(T0), Ok(Some(UpdateAction::Check)));
        assert_eq!(f.check_now(T0), Ok(None), "one at a time");
        assert_eq!(f.checked(Ok(Some(v("0.0.2"))), T0), None);
        assert!(f.status().prompt, "found by hand: offered, not downloaded");
    }

    #[test]
    fn checks_again_every_few_hours_and_sooner_after_a_failure() {
        let mut f = UpdateFlow::new(true, UpdatePolicy::Ask);
        assert_eq!(f.tick(T0), Some(UpdateAction::Check));
        assert_eq!(f.checked(Ok(None), T0), None);
        assert_eq!(f.status().state, UpdateState::UpToDate);
        assert_eq!(f.tick(T0 + CHECK_EVERY_MS - 1), None);
        assert_eq!(f.tick(T0 + CHECK_EVERY_MS), Some(UpdateAction::Check));
        let t1 = T0 + CHECK_EVERY_MS;
        let offline = ErrorCode::UpdateCheckFailed;
        assert_eq!(f.checked(Err(offline.clone()), t1), None);
        assert_eq!(f.status().state, UpdateState::Failed { error: offline });
        assert_eq!(
            f.status().checked_at,
            Some(T0),
            "a failed check is not a check"
        );
        assert_eq!(f.tick(t1 + RETRY_AFTER_MS - 1), None);
        assert_eq!(f.tick(t1 + RETRY_AFTER_MS), Some(UpdateAction::Check));
    }

    #[test]
    fn a_waiting_version_is_not_looked_for_again() {
        let mut f = UpdateFlow::new(true, UpdatePolicy::Ask);
        f.tick(T0);
        f.checked(Ok(Some(v("0.0.2"))), T0);
        assert_eq!(f.tick(T0 + CHECK_EVERY_MS * 2), None);
    }

    #[test]
    fn later_hides_the_offer_until_the_next_start_only_for_that_version() {
        let mut f = UpdateFlow::new(true, UpdatePolicy::Ask);
        f.tick(T0);
        f.checked(Ok(Some(v("0.0.2"))), T0);
        f.later();
        assert!(!f.status().prompt);
        // Still installable from «Controlla ora» → the same version is not offered again…
        assert_eq!(f.offered(), Some("0.0.2"));
        // …but a newer one found by hand is.
        assert_eq!(
            f.install(),
            Ok(Some(UpdateAction::Download {
                version: v("0.0.2")
            }))
        );
        f.downloaded(v("0.0.2"), Err(ErrorCode::UpdateDownloadFailed));
        assert_eq!(f.check_now(T0 + 5), Ok(Some(UpdateAction::Check)));
        f.checked(Ok(Some(v("0.0.3"))), T0 + 5);
        assert!(f.status().prompt);
    }

    #[test]
    fn a_failed_download_or_install_is_reported_and_nothing_is_lost() {
        let mut f = UpdateFlow::new(true, UpdatePolicy::Auto);
        f.tick(T0);
        f.checked(Ok(Some(v("0.0.2"))), T0);
        let bad = ErrorCode::UpdateDownloadFailed;
        assert_eq!(f.downloaded(v("0.0.2"), Err(bad.clone())), None);
        assert_eq!(f.status().state, UpdateState::Failed { error: bad });
        assert_eq!(f.install(), Err(ErrorCode::UpdateNotReady));
        // Tried again later, downloaded, the installer refuses to start: offered again.
        assert_eq!(f.tick(T0 + RETRY_AFTER_MS), Some(UpdateAction::Check));
        f.checked(Ok(Some(v("0.0.2"))), T0 + RETRY_AFTER_MS);
        f.downloaded(v("0.0.2"), Ok(()));
        f.later();
        f.install().unwrap();
        f.install_failed(v("0.0.2"), ErrorCode::UpdateInstallFailed);
        assert_eq!(
            f.status().state,
            UpdateState::Ready {
                version: v("0.0.2")
            }
        );
        assert!(f.status().prompt);
    }

    #[test]
    fn changing_the_policy_keeps_what_was_found() {
        let mut f = UpdateFlow::new(true, UpdatePolicy::Ask);
        f.tick(T0);
        f.checked(Ok(Some(v("0.0.2"))), T0);
        f.set_policy(UpdatePolicy::Off);
        assert_eq!(f.policy(), UpdatePolicy::Off);
        assert!(f.status().prompt);
    }

    #[test]
    fn the_status_reaches_the_interface_as_tagged_json() {
        let mut f = UpdateFlow::new(true, UpdatePolicy::Ask);
        f.tick(T0);
        f.checked(Ok(Some(v("0.0.2"))), 5);
        assert_eq!(
            serde_json::to_value(f.status()).unwrap(),
            serde_json::json!({
                "state": { "state": "available", "version": "0.0.2" },
                "checkedAt": 5,
                "prompt": true
            })
        );
    }
}

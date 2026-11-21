//! First run (Trello #35, boards PrimoAvvio and Tel-PrimoAvvio): which steps the flow shows and
//! whether it shows at all. Pure; the shell keeps the «done» marker on disk.
//!
//! Four steps: S01 Welcome (language), S02 the games folders with the scan, S04 the controller
//! (only while one is connected), S06 «Tutto pronto». The ids keep the boards' numbers.

use serde::Serialize;
use ts_rs::TS;

/// One step of the first run. TypeScript: `FirstRunStep` (generated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename = "FirstRunStep")]
pub enum Step {
    /// S01: welcome and language.
    Welcome,
    /// S02: games folders, with the scan in the same step.
    Games,
    /// S04: A/B layout and a press test.
    Controller,
    /// S06: «Tutto pronto», with the optional imports.
    Ready,
}

/// The steps in order: the controller step only while a controller is connected.
pub fn steps(controller_connected: bool) -> Vec<Step> {
    [Step::Welcome, Step::Games, Step::Controller, Step::Ready]
        .into_iter()
        .filter(|s| controller_connected || *s != Step::Controller)
        .collect()
}

/// Whether to show the first run: never once it was finished on this device, and never to a
/// library that is already set up (folders chosen or games in it: an existing user upgrading).
pub fn should_show(done_on_this_device: bool, library_set_up: bool) -> bool {
    !done_on_this_device && !library_set_up
}

#[cfg(test)]
mod tests {
    use super::{should_show, steps, Step};

    #[test]
    fn the_controller_step_is_there_only_with_a_controller() {
        assert_eq!(
            steps(true),
            vec![Step::Welcome, Step::Games, Step::Controller, Step::Ready]
        );
        assert_eq!(steps(false), vec![Step::Welcome, Step::Games, Step::Ready]);
    }

    #[test]
    fn shown_only_on_a_new_device_with_an_empty_library() {
        assert!(should_show(false, false));
        assert!(!should_show(true, false));
        assert!(!should_show(false, true));
        assert!(!should_show(true, true));
    }

    #[test]
    fn steps_serialize_as_lowercase_ids() {
        let ids: Vec<String> = steps(true)
            .iter()
            .filter_map(|s| serde_json::to_value(s).ok())
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect();
        assert_eq!(ids, ["welcome", "games", "controller", "ready"]);
    }
}

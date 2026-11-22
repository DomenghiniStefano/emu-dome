//! What opens when EmuDome starts: the game given on the command line or by Android's intent
//! (`startup_game`), else, with `general.start` = «last game», the library game played last if
//! its file is still there (`startup_last_game`), else the library. The interface asks for both
//! after its first screen is decided (`useGameLaunch.openStartupGame`).

use emudome_core::library::GameRecord;
use emudome_core::settings::{keys, Scope};
use tauri::AppHandle;

use crate::library::GameDto;

/// Whether to open the last game: `general.start` says so and no game was given at startup.
pub fn wants_last_game(start: &str, given: bool) -> bool {
    start == "last_game" && !given
}

/// The game played last among the ones the library lists and found at its last scan.
pub fn last_played<'a>(games: impl Iterator<Item = &'a GameRecord>) -> Option<&'a GameRecord> {
    games
        .filter(|g| g.available && g.listed && g.last_played.is_some())
        .max_by_key(|g| g.last_played)
}

/// Game given on the command line (`emudome game.gba`), opened at startup.
#[tauri::command]
pub(crate) fn startup_game() -> Option<String> {
    emudome_platform::startup_game()
}

/// `general.start` = «last game»: the library game to open at startup (its row, as the library
/// lists it), `None` with «library», with a game given at startup, or when there is none to open.
#[tauri::command]
pub(crate) fn startup_last_game(app: AppHandle) -> Option<GameDto> {
    let start = keys::Reader::new(crate::settings::resolver(&app, &Scope::Global))
        .text(keys::GENERAL_START);
    if !wants_last_game(&start, emudome_platform::startup_game().is_some()) {
        return None;
    }
    let game = crate::library::last_played_game(&app);
    log::info!(
        "general.start: last game {}",
        if game.is_some() {
            "opened"
        } else {
            "not found"
        }
    );
    game
}

#[cfg(test)]
mod tests {
    use super::{last_played, wants_last_game};
    use emudome_core::library::{CoverKind, GameRecord};
    use emudome_core::settings::ConsoleId;
    use emudome_core::GameKey;

    fn game(hex: &str, last_played: Option<u64>, available: bool, listed: bool) -> GameRecord {
        GameRecord {
            id: GameKey::new(ConsoleId::Gba, hex).unwrap(),
            console: ConsoleId::Gba,
            title: hex.into(),
            path: format!("C:/roms/{hex}.gba"),
            folder: None,
            size: 1,
            mtime: 1,
            available,
            listed,
            play_time_secs: 0,
            last_played,
            added_at: 0,
            cover: CoverKind::Generated,
            user_cover: None,
            cover_rev: 0,
            has_icon: false,
            favourite: false,
            hidden: false,
            code: None,
            version: None,
            name: None,
        }
    }

    #[test]
    fn the_game_played_last_that_is_still_there_opens() {
        let games = [
            game("aa", Some(10), true, true),
            game("bb", Some(30), false, true), // not found at the last scan
            game("cc", Some(40), true, false), // its folder left the library
            game("dd", Some(20), true, true),
            game("ee", None, true, true), // never played
        ];
        let last = last_played(games.iter()).map(|g| g.title.as_str());
        assert_eq!(last, Some("dd"));
        assert!(last_played([game("ee", None, true, true)].iter()).is_none());
    }

    #[test]
    fn a_game_given_at_startup_wins_and_library_opens_nothing() {
        assert!(wants_last_game("last_game", false));
        assert!(!wants_last_game("last_game", true));
        assert!(!wants_last_game("library", false));
    }
}

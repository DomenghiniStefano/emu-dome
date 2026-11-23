//! Tauri side of Google Drive (Trello #20, #31): the account (whether sync exists in this build,
//! whether this device is signed in, signing in and out), the Cloud screens' commands, and the
//! game flow's hooks. The sync itself is `emudome_sync::service::CloudService` (no Tauri there,
//! tested on `FakeDrive`); this module hands it the folders, the `cloud.*` settings and the
//! library, and turns its notices into events:
//!
//! - `cloud://status` (`CloudStatus`): the whole status, at every change;
//! - `cloud://progress` (`CloudProgress`): a pull before launch (G03), a push after exit (G14),
//!   a sync, a game file going up or down;
//! - `cloud://conflict` (`CloudGameConflict`): a game whose save changed on both sides (G04);
//! - `cloud://in-use` (`CloudInUseEvent`): a game opened while another device has it open,
//!   opened without the G05 question (startup, «Open with»).
//!
//! The game flow, while signed in with `cloud.ssaves` on and `cloud.sauto` = `game`:
//! - the interface asks `cloud_launch_check` before opening a library game (G05);
//! - opening it pulls its saves first ([`before_launch`], in the open's blocking thread; G03's
//!   «Gioca senza aspettare» is `cloud_skip_pull`), marks it pending, then claims the «in use»
//!   marker and keeps its heartbeat while it runs ([`heartbeat_hooks`]);
//! - once its thread ended (the exit writes done), its push starts in the background
//!   ([`after_exit`]); the close never waits for it. A push cut off (the app quitting, no
//!   network, a crash) is done at the next start ([`startup`]): the game stays pending until a
//!   push goes through.
//!
//! The refresh token lives in the system's credential store (`emudome_platform::cloud`), under
//! the service `it.domehub.emudome` and the account [`emudome_sync::auth::REFRESH_TOKEN_ACCOUNT`];
//! never in a file or a log.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use emudome_core::settings::keys::{self, Reader};
use emudome_core::settings::{ConsoleId, Scope};
use emudome_core::{ErrorCode, GameKey};
use emudome_platform::cloud as system;
use emudome_sync::auth::{Auth, SecretStore, REFRESH_TOKEN_ACCOUNT, SIGN_IN_TIMEOUT};
use emudome_sync::drive::rest::RestDrive;
use emudome_sync::loopback::{page, Pages};
use emudome_sync::marker::InUse;
use emudome_sync::oauth::{Endpoints, OAuthClient, Secret};
use emudome_sync::service::{
    Activity, Choice, CloudGame, CloudService, CloudStatus, DeviceDto, Flags, GameConflict,
    GameRef, GameState, Notice, Paths, Prefs,
};
use emudome_sync::sync::QuickSync;
use emudome_sync::SyncError;
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter, Manager};

use crate::game_session::{AfterEnd, Hooks};
use crate::{events, library, settings, AppState};

type Service = CloudService<RestDrive<Arc<Auth>>>;

/// The refresh token in the system's credential store.
struct SystemStore;

fn store_error(e: system::CloudError) -> SyncError {
    match e {
        system::CloudError::Unsupported => SyncError::SignInUnsupported,
        system::CloudError::Failed(detail) => SyncError::Credentials(detail),
    }
}

impl SecretStore for SystemStore {
    fn load(&self) -> Result<Option<Secret>, SyncError> {
        system::credential_get(REFRESH_TOKEN_ACCOUNT)
            .map(|t| t.map(Secret::new))
            .map_err(store_error)
    }

    fn save(&self, token: &Secret) -> Result<(), SyncError> {
        system::credential_set(REFRESH_TOKEN_ACCOUNT, token.expose()).map_err(store_error)
    }

    fn delete(&self) -> Result<(), SyncError> {
        system::credential_delete(REFRESH_TOKEN_ACCOUNT).map_err(store_error)
    }
}

/// The account and the sync of this run, made at their first use (the HTTP client reads the
/// system's certificates once).
#[derive(Default)]
pub struct Cloud {
    auth: Mutex<Option<Arc<Auth>>>,
    service: Mutex<Option<Arc<Service>>>,
    /// The last answer of the credential store: a sign-in is kept on this device.
    signed_in: AtomicBool,
    /// The heartbeat of the open game: set to stop it.
    heartbeat: Mutex<Option<Arc<AtomicBool>>>,
}

/// EmuDome's own games folder, `<app local data>/games/`: where «Scarica» puts a game from
/// Drive. The only folder «Togli dal telefono» deletes a game file from (`storage.rs`).
pub fn games_dir<R: tauri::Runtime, M: Manager<R>>(app: &M) -> tauri::Result<PathBuf> {
    Ok(crate::dirs::local(app)?.join("games"))
}

/// Whether the game file of `id` is on Drive now (asked of Drive: `cloudNotSignedIn` and the
/// other sign-in codes when sync cannot run).
pub async fn on_drive(app: &AppHandle, id: &GameKey) -> Result<bool, ErrorCode> {
    let service = app.state::<Cloud>().signed_service(app)?;
    Ok(service
        .games(&library::records(app))
        .await?
        .iter()
        .any(|g| &g.id == id && g.on_drive))
}

fn configured() -> bool {
    OAuthClient::from_build().is_some()
}

impl Cloud {
    fn auth(&self) -> Result<Arc<Auth>, ErrorCode> {
        let mut auth = self.auth.lock();
        if let Some(a) = auth.as_ref() {
            return Ok(a.clone());
        }
        let made = Arc::new(Auth::new(
            emudome_sync::http::client()?,
            Endpoints::google(),
            OAuthClient::from_build(),
            Arc::new(SystemStore),
        ));
        *auth = Some(made.clone());
        Ok(made)
    }

    /// The sync service, made at its first use with the app's folders.
    fn service(&self, app: &AppHandle) -> Result<Arc<Service>, ErrorCode> {
        if let Some(s) = self.service.lock().as_ref() {
            return Ok(s.clone());
        }
        let auth = self.auth()?;
        let folder = |e: tauri::Error| ErrorCode::SaveFileFailed {
            detail: format!("no data folder: {e}"),
        };
        let data = crate::dirs::data(app).map_err(folder)?;
        let paths = Paths {
            saves_root: data.join("saves"),
            settings: data.join("settings"),
            sync: data.join("sync"),
            games: games_dir(app).map_err(folder)?,
        };
        let notify_app = app.clone();
        let made = Arc::new(CloudService::new(
            RestDrive::new(auth.http().clone(), auth),
            paths,
            system::device_name(),
            Arc::new(move |notice| notify(&notify_app, notice)),
        ));
        // Made twice at once: the first one stays (one index per run).
        Ok(self.service.lock().get_or_insert(made).clone())
    }

    fn existing(&self) -> Option<Arc<Service>> {
        self.service.lock().clone()
    }

    fn flags(&self) -> Flags {
        Flags {
            configured: configured(),
            supported: system::SIGN_IN_SUPPORTED,
            signed_in: self.signed_in.load(Ordering::SeqCst),
        }
    }

    /// Sync can run: this build has the client, this system signs in, and a sign-in is kept.
    fn active(&self) -> bool {
        let f = self.flags();
        f.configured && f.supported && f.signed_in
    }

    fn status_now(&self) -> CloudStatus {
        match self.existing() {
            Some(s) => s.status(self.flags()),
            None => empty_status(self.flags()),
        }
    }

    /// Asks the credential store again whether a sign-in is kept.
    async fn refresh_signed_in(&self) -> Result<bool, ErrorCode> {
        let signed = if configured() && system::SIGN_IN_SUPPORTED {
            self.auth()?.signed_in().await?
        } else {
            false
        };
        self.signed_in.store(signed, Ordering::SeqCst);
        Ok(signed)
    }

    /// The service, or `cloudNotConfigured` / `cloudSignInUnsupported` / `cloudNotSignedIn`.
    fn signed_service(&self, app: &AppHandle) -> Result<Arc<Service>, ErrorCode> {
        if !configured() {
            return Err(ErrorCode::CloudNotConfigured);
        }
        if !system::SIGN_IN_SUPPORTED {
            return Err(ErrorCode::CloudSignInUnsupported);
        }
        if !self.signed_in.load(Ordering::SeqCst) {
            return Err(ErrorCode::CloudNotSignedIn);
        }
        self.service(app)
    }
}

/// The status before the service exists (nothing measured yet).
fn empty_status(flags: Flags) -> CloudStatus {
    CloudStatus {
        configured: flags.configured,
        supported: flags.supported,
        signed_in: flags.configured && flags.supported && flags.signed_in,
        account: None,
        quota: None,
        emudome_bytes: None,
        syncing: false,
        last_sync: None,
        conflicts: Vec::new(),
        pending: Vec::new(),
        error: None,
    }
}

/// A notice of the service, as its event.
fn notify(app: &AppHandle, notice: Notice) {
    let sent = match notice {
        Notice::Status => {
            let Some(cloud) = app.try_state::<Cloud>() else {
                return;
            };
            app.emit(events::CLOUD_STATUS, cloud.status_now())
        }
        Notice::Progress(p) => app.emit(events::CLOUD_PROGRESS, p),
        Notice::Conflict(c) => app.emit(events::CLOUD_CONFLICT, c),
        Notice::InUse(u) => app.emit(events::CLOUD_IN_USE, u),
    };
    if let Err(e) = sent {
        log::warn!("cloud event not sent to the interface: {e}");
    }
}

/// The `cloud.*` settings a run follows, at the console's level when it is about one game
/// (`cloud.squick` can be set per console).
fn prefs(app: &AppHandle, console: Option<ConsoleId>) -> Prefs {
    let scope = console.map_or(Scope::Global, |console| Scope::Console { console });
    let read = Reader::new(settings::resolver(app, &scope));
    Prefs {
        saves: read.bool(keys::CLOUD_SAVES),
        quick: QuickSync::from_id(&read.text(keys::CLOUD_QUICK)),
        settings: read.bool(keys::CLOUD_SETTINGS),
        auto: read.text(keys::CLOUD_AUTO) != "manual",
    }
}

/// The game `id` as the notices name it (the library's title and play time).
fn game_ref(app: &AppHandle, id: &GameKey) -> GameRef {
    let record = library::records(app).into_iter().find(|g| &g.id == id);
    GameRef {
        key: id.clone(),
        title: record
            .as_ref()
            .map_or_else(|| id.to_string(), |g| g.shown_title().to_owned()),
        play_time: record.map(|g| g.play_time_secs),
    }
}

fn open_game(app: &AppHandle) -> Option<GameKey> {
    app.try_state::<AppState>().and_then(|s| s.games.in_use())
}

/// The row of `id` in «Giochi su Google Drive».
async fn game_row(
    service: &Service,
    app: &AppHandle,
    id: &GameKey,
) -> Result<CloudGame, ErrorCode> {
    service
        .games(&library::records(app))
        .await?
        .into_iter()
        .find(|g| &g.id == id)
        .ok_or_else(|| ErrorCode::LibraryGameUnknown { id: id.to_string() })
}

// ---- The game flow ----

/// At start (and after a sign-in), off the event loop: whether a sign-in is kept, the account,
/// the pushes a previous run could not finish, the settings and this device's library list.
pub fn startup(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let cloud = app.state::<Cloud>();
        match cloud.refresh_signed_in().await {
            Ok(true) => {}
            Ok(false) => return,
            Err(e) => {
                log::warn!("cloud: sign-in state unknown at start: {e:?}");
                return;
            }
        }
        let service = match cloud.service(&app) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("cloud: sync unavailable: {e:?}");
                return;
            }
        };
        notify(&app, Notice::Status);
        // Failures are in the status and the activity (`cloud://status`).
        if service.refresh_account().await.is_err() {
            return;
        }
        let prefs = prefs(&app, None);
        if !prefs.auto {
            return;
        }
        let records = library::records(&app);
        if let Err(e) = service
            .sync_start(&prefs, &records, open_game(&app).as_ref())
            .await
        {
            log::info!("cloud: the sync at start did not finish: {e}");
        }
    });
}

/// Before `game` opens (in the open's blocking thread, after the previous game closed): its
/// newer saves come from Drive (G03), then it is pending until the push after its exit goes
/// through. A failure never stops the game: it opens with this device's files, the failure in
/// the status (`cloud://status`).
pub fn before_launch(app: &AppHandle, game: &GameKey) {
    let cloud = app.state::<Cloud>();
    if !cloud.active() {
        return;
    }
    let prefs = prefs(app, Some(game.console()));
    if !(prefs.saves && prefs.auto) {
        return;
    }
    let Ok(service) = cloud.service(app) else {
        return;
    };
    let game = game_ref(app, game);
    if let Err(e) = tauri::async_runtime::block_on(service.pull_game(&prefs, &game)) {
        log::info!("cloud: {} opens without its pull: {e}", game.key);
    }
    if let Err(e) = service.mark_pending(&game.key) {
        log::warn!("cloud: {} not marked pending: {e}", game.key);
    }
}

/// Once `game`'s thread ended (its exit writes done): its push, in the background. The close
/// flow never waits for it. `None` while sync is off.
pub fn after_exit(app: &AppHandle, game: &GameKey) -> Option<AfterEnd> {
    if !app.state::<Cloud>().active() {
        return None;
    }
    let (app, game) = (app.clone(), game.clone());
    Some(Box::new(move || {
        let prefs = prefs(&app, Some(game.console()));
        if !(prefs.saves && prefs.auto) {
            // `manual`: it stays pending for «Sincronizza ora».
            return;
        }
        let Ok(service) = app.state::<Cloud>().service(&app) else {
            return;
        };
        let game = game_ref(&app, &game);
        tauri::async_runtime::spawn(async move {
            if let Err(e) = service.push_game(&prefs, &game).await {
                log::info!("cloud: {} not pushed yet: {e}", game.key);
            }
        });
    }))
}

/// The «in use» marker while a game runs: claimed when it opens, again every heartbeat, until
/// it closes (its push after exit takes the marker off).
pub fn heartbeat_hooks(app: &AppHandle) -> Hooks {
    let (a, b) = (app.clone(), app.clone());
    Hooks::new(
        move |g| start_heartbeat(&a, &g.key),
        move || {
            if let Some(cloud) = b.try_state::<Cloud>() {
                if let Some(stop) = cloud.heartbeat.lock().take() {
                    stop.store(true, Ordering::SeqCst);
                }
            }
        },
    )
}

fn start_heartbeat(app: &AppHandle, game: &GameKey) {
    let Some(cloud) = app.try_state::<Cloud>() else {
        return;
    };
    if !cloud.active() {
        return;
    }
    let prefs = prefs(app, Some(game.console()));
    if !(prefs.saves && prefs.auto) {
        return;
    }
    let Ok(service) = cloud.service(app) else {
        return;
    };
    let stop = Arc::new(AtomicBool::new(false));
    if let Some(old) = cloud.heartbeat.lock().replace(stop.clone()) {
        old.store(true, Ordering::SeqCst);
    }
    let game = game.clone();
    tauri::async_runtime::spawn(async move {
        service.heartbeat(&prefs, &game, &stop).await;
    });
}

// ---- Commands ----

/// The whole Cloud status (asks the credential store again; the account and the space are
/// fetched in the background when not known yet, then sent on `cloud://status`).
#[tauri::command]
pub async fn cloud_status(app: AppHandle) -> Result<CloudStatus, ErrorCode> {
    let cloud = app.state::<Cloud>();
    if cloud.refresh_signed_in().await? {
        let service = cloud.service(&app)?;
        if service.status(cloud.flags()).account.is_none() {
            tauri::async_runtime::spawn(async move {
                // A failure is in the status (`cloud://status`).
                if let Err(e) = service.refresh_account().await {
                    log::info!("cloud: account not read: {e}");
                }
            });
        }
    }
    Ok(cloud.status_now())
}

/// Signs in to Google Drive: the system browser opens Google's page, the user signs in there
/// and the browser comes back to a local page showing `done` (or `failed`), the interface's
/// texts. Waits up to five minutes. Then the account, the space and the sync at start follow in
/// the background (`cloud://status`).
#[tauri::command]
pub async fn cloud_sign_in(
    app: AppHandle,
    done: String,
    failed: String,
) -> Result<CloudStatus, ErrorCode> {
    if !configured() {
        return Err(ErrorCode::CloudNotConfigured);
    }
    if !system::SIGN_IN_SUPPORTED {
        return Err(ErrorCode::CloudSignInUnsupported);
    }
    let cloud = app.state::<Cloud>();
    let pages = Pages {
        done: page(&done),
        failed: page(&failed),
    };
    cloud
        .auth()?
        .sign_in(
            |url| system::open_url(url).map_err(|e| e.to_string()),
            &pages,
            SIGN_IN_TIMEOUT,
        )
        .await?;
    cloud.signed_in.store(true, Ordering::SeqCst);
    startup(&app);
    Ok(cloud.status_now())
}

/// Signs this device out (the grant is revoked at Google when it can be reached). The files on
/// both sides stay as they are.
#[tauri::command]
pub async fn cloud_sign_out(app: AppHandle) -> Result<CloudStatus, ErrorCode> {
    let cloud = app.state::<Cloud>();
    if configured() && system::SIGN_IN_SUPPORTED {
        cloud.auth()?.sign_out().await?;
    }
    cloud.signed_in.store(false, Ordering::SeqCst);
    if let Some(s) = cloud.existing() {
        s.signed_out();
    }
    Ok(cloud.status_now())
}

/// «Sincronizza ora»: the settings, every game's saves both ways (not the open game's), this
/// device's library list, the account and its space. Answers the status after it.
#[tauri::command]
pub async fn cloud_sync_now(app: AppHandle) -> Result<CloudStatus, ErrorCode> {
    let cloud = app.state::<Cloud>();
    let service = cloud.signed_service(&app)?;
    let records = library::records(&app);
    service
        .sync_all(&prefs(&app, None), &records, open_game(&app).as_ref())
        .await?;
    Ok(cloud.status_now())
}

/// «Giochi su Google Drive»: every library game and every game file on Drive.
#[tauri::command]
pub async fn cloud_games(app: AppHandle) -> Result<Vec<CloudGame>, ErrorCode> {
    let service = app.state::<Cloud>().signed_service(&app)?;
    Ok(service.games(&library::records(&app)).await?)
}

/// «Carica»: the game file goes to Drive (streamed; E09 `cloudQuotaFull` first when it does not
/// fit). Answers its row.
#[tauri::command]
pub async fn cloud_game_upload(app: AppHandle, id: GameKey) -> Result<CloudGame, ErrorCode> {
    let service = app.state::<Cloud>().signed_service(&app)?;
    let record = library::records(&app)
        .into_iter()
        .find(|g| g.id == id && g.listed)
        .ok_or_else(|| ErrorCode::LibraryGameUnknown { id: id.to_string() })?;
    service.upload_game(&record).await?;
    game_row(&service, &app, &id).await
}

/// «Scarica» / «Scarica e gioca»: the game file comes from Drive into
/// `<app local data>/games/<console>/` (streamed, checked, never over another file) and joins
/// the library (`library://changed` when it is in). Answers its row, already «Su entrambi».
#[tauri::command]
pub async fn cloud_game_download(app: AppHandle, id: GameKey) -> Result<CloudGame, ErrorCode> {
    let service = app.state::<Cloud>().signed_service(&app)?;
    let path = service.download_game(&id).await?;
    library::add_file(&app, path);
    let mut row = game_row(&service, &app, &id).await?;
    row.here = true;
    if row.state == GameState::DriveOnly {
        row.state = GameState::Both;
    }
    Ok(row)
}

/// «Rimuovi da Drive»: the game file goes to Drive's trash; its saves and everything on this
/// device stay. Answers its row (`libraryGameUnknown` when nothing of it is left to list).
#[tauri::command]
pub async fn cloud_game_remove(app: AppHandle, id: GameKey) -> Result<CloudGame, ErrorCode> {
    let service = app.state::<Cloud>().signed_service(&app)?;
    service.remove_game(&id).await?;
    game_row(&service, &app, &id).await
}

/// «I tuoi dispositivi»: this device first, then the others by their last publication.
#[tauri::command]
pub async fn cloud_devices(app: AppHandle) -> Result<Vec<DeviceDto>, ErrorCode> {
    let service = app.state::<Cloud>().signed_service(&app)?;
    Ok(service.devices().await?)
}

/// «Attività recenti», newest first (this run's; empty before the first sync).
#[tauri::command]
pub fn cloud_activity(app: AppHandle) -> Vec<Activity> {
    app.state::<Cloud>()
        .existing()
        .map(|s| s.activity())
        .unwrap_or_default()
}

/// G04: the two versions of `id`; null when it has no conflict (any more).
#[tauri::command]
pub fn cloud_conflict(app: AppHandle, id: GameKey) -> Option<GameConflict> {
    let service = app.state::<Cloud>().existing()?;
    service.conflict(&game_ref(&app, &id))
}

/// G04's answer: «Continua con …» (`local` / `remote`) or «Tieni entrambe» (`both`). Refused
/// with `saveGameOpen` while the game is open (its files are in use). Answers the status (the
/// game leaves `conflicts`, unless a file changed since the question).
#[tauri::command]
pub async fn cloud_resolve(
    app: AppHandle,
    id: GameKey,
    choice: Choice,
) -> Result<CloudStatus, ErrorCode> {
    let cloud = app.state::<Cloud>();
    let service = cloud.signed_service(&app)?;
    if open_game(&app).as_ref() == Some(&id) {
        return Err(ErrorCode::SaveGameOpen);
    }
    let prefs = prefs(&app, Some(id.console()));
    service
        .resolve(&prefs, &game_ref(&app, &id), choice)
        .await?;
    Ok(cloud.status_now())
}

/// G05, before opening a library game: whether another device has it open. `free` whenever
/// sync is off, not signed in, or its saves do not sync automatically.
#[tauri::command]
pub async fn cloud_launch_check(app: AppHandle, id: GameKey) -> Result<InUse, ErrorCode> {
    let cloud = app.state::<Cloud>();
    let prefs = prefs(&app, Some(id.console()));
    let synced = cloud.active() && prefs.saves && prefs.auto;
    if !synced {
        return Ok(InUse::Free);
    }
    let service = cloud.service(&app)?;
    Ok(service.launch_check(&id).await?)
}

/// G03's «Gioca senza aspettare»: the pull before `id` opens stops before its next file and the
/// game opens with this device's files. False: no pull of it is running.
#[tauri::command]
pub fn cloud_skip_pull(app: AppHandle, id: GameKey) -> bool {
    app.state::<Cloud>()
        .existing()
        .is_some_and(|s| s.skip_pull(&id))
}

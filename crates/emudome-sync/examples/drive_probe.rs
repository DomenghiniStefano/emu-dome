//! Drive probe for the owner's UAT (Trello #20): the real sign-in and Drive calls, without the
//! app. Never run by the checks or the tests (it talks to Google).
//!
//! ```text
//! cargo run -p emudome-sync --example drive_probe -- <command>
//!   status                 build configured? signed in? account and Drive space
//!   sign-in [--print-url]  sign in (system browser; --print-url prints the URL to open instead)
//!   sign-out               revoke the grant and forget the refresh token
//!   list                   the EmuDome/ folder, as a tree
//!   quota                  the Drive space
//!   upload <file>          upload <file> into EmuDome-probe/ (resumable above 5 MiB)
//!   download <name> <out>  download EmuDome-probe/<name> to <out> (checked against its MD5)
//! ```
//!
//! Needs a build with `EMUDOME_GOOGLE_CLIENT_ID` (and `EMUDOME_GOOGLE_CLIENT_SECRET`) set at
//! compile time. The refresh token goes to the system's credential store, as in the app (service
//! `it.domehub.emudome`, account `google-drive`): signing in here signs the app in too. Nothing
//! printed holds a token or the client secret; the authorization URL printed by `--print-url`
//! holds the client id, as every Google sign-in URL does.

use std::path::Path;
use std::sync::Arc;

use emudome_platform::cloud as system;
use emudome_sync::auth::{Auth, SecretStore, REFRESH_TOKEN_ACCOUNT, SIGN_IN_TIMEOUT};
use emudome_sync::drive::rest::RestDrive;
use emudome_sync::drive::{md5_hex, Content, DriveApi, Properties, RemoteFile, ROOT};
use emudome_sync::loopback::Pages;
use emudome_sync::oauth::{Endpoints, OAuthClient, Secret};
use emudome_sync::sync::ROOT_FOLDER;
use emudome_sync::SyncError;

const PROBE_FOLDER: &str = "EmuDome-probe";

struct SystemStore;

fn store_error(e: system::CloudError) -> SyncError {
    match e {
        system::CloudError::Unsupported => SyncError::SignInUnsupported,
        system::CloudError::Failed(d) => SyncError::Credentials(d),
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

fn usage() -> ! {
    eprintln!(
        "usage: drive_probe status | sign-in [--print-url] | sign-out | list | quota | \
         upload <file> | download <name> <out>"
    );
    std::process::exit(2)
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = args.first() else { usage() };
    match run(command, &args[1..]).await {
        Ok(()) => {}
        Err(e) => {
            eprintln!("error: {e}");
            eprintln!(
                "as the app shows it: {:?}",
                emudome_core::ErrorCode::from(e)
            );
            std::process::exit(1);
        }
    }
}

async fn run(command: &str, rest: &[String]) -> Result<(), SyncError> {
    let client = OAuthClient::from_build();
    println!(
        "build: {}",
        if client.is_some() {
            "Google OAuth client present"
        } else {
            "no Google OAuth client (set EMUDOME_GOOGLE_CLIENT_ID when building)"
        }
    );
    let auth = Arc::new(Auth::new(
        emudome_sync::http::client()?,
        Endpoints::google(),
        client,
        Arc::new(SystemStore),
    ));
    let drive = RestDrive::new(auth.http().clone(), auth.clone());
    match command {
        "status" => {
            let signed_in = auth.signed_in().await?;
            println!("signed in on this device: {signed_in}");
            if signed_in {
                let account = drive.account().await?;
                println!(
                    "account: {}",
                    account.email.as_deref().unwrap_or("(no email given)")
                );
                print_quota(&drive).await?;
            }
        }
        "sign-in" => {
            let print = rest.iter().any(|a| a == "--print-url");
            let open = |url: &str| {
                if print {
                    println!("open this URL in a browser on this computer:\n{url}");
                    Ok(())
                } else {
                    println!("opening the system browser...");
                    system::open_url(url).map_err(|e| e.to_string())
                }
            };
            auth.sign_in(open, &Pages::plain(), SIGN_IN_TIMEOUT).await?;
            let account = drive.account().await?;
            println!(
                "signed in as {}",
                account.email.as_deref().unwrap_or("(no email given)")
            );
        }
        "sign-out" => {
            auth.sign_out().await?;
            println!("signed out: the refresh token is gone from the credential store");
        }
        "list" => match drive.find(ROOT, ROOT_FOLDER).await? {
            None => println!("no {ROOT_FOLDER}/ folder yet"),
            Some(root) => {
                println!("{ROOT_FOLDER}/");
                print_tree(&drive, &root.id, 1).await?;
            }
        },
        "quota" => print_quota(&drive).await?,
        "upload" => {
            let Some(file) = rest.first() else { usage() };
            let path = Path::new(file);
            let size = std::fs::metadata(path)
                .map_err(|e| SyncError::local(path, &e))?
                .len();
            let quota = drive.quota().await?;
            if !quota.fits(size) {
                return Err(SyncError::QuotaFull {
                    needed: size,
                    available: quota.available().unwrap_or(u64::MAX),
                });
            }
            let folder = probe_folder(&drive).await?;
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("probe.bin")
                .to_owned();
            let content = Content::File {
                path: path.to_path_buf(),
                size,
            };
            let started = std::time::Instant::now();
            let made = match drive.find(&folder, &name).await? {
                Some(f) => drive.update(&f.id, &Properties::new(), content).await?,
                None => {
                    drive
                        .create(&folder, &name, &Properties::new(), content)
                        .await?
                }
            };
            println!(
                "uploaded {PROBE_FOLDER}/{name}: {size} bytes in {:.1} s, md5 {}",
                started.elapsed().as_secs_f64(),
                made.md5.as_deref().unwrap_or("?")
            );
        }
        "download" => {
            let (Some(name), Some(out)) = (rest.first(), rest.get(1)) else {
                usage()
            };
            let folder = probe_folder(&drive).await?;
            let file = drive
                .find(&folder, name)
                .await?
                .ok_or_else(|| SyncError::NotFound(format!("{PROBE_FOLDER}/{name}")))?;
            let bytes = drive.download(&file.id).await?;
            let md5 = md5_hex(&bytes);
            if file.md5.as_deref() != Some(md5.as_str()) {
                return Err(SyncError::Request("download does not match its MD5".into()));
            }
            emudome_core::write_atomic(Path::new(out), &bytes)
                .map_err(|e| SyncError::local(Path::new(out), &e))?;
            println!(
                "downloaded {} bytes to {out}, md5 {md5} (matches Drive)",
                bytes.len()
            );
        }
        _ => usage(),
    }
    Ok(())
}

async fn print_quota(drive: &impl DriveApi) -> Result<(), SyncError> {
    let q = drive.quota().await?;
    let mb = |b: u64| b / (1024 * 1024);
    match q.limit {
        Some(limit) => println!(
            "Drive space: {} MB used of {} MB, {} MB free",
            mb(q.usage),
            mb(limit),
            mb(q.available().unwrap_or(0))
        ),
        None => println!("Drive space: {} MB used, no limit", mb(q.usage)),
    }
    Ok(())
}

async fn probe_folder(drive: &impl DriveApi) -> Result<String, SyncError> {
    Ok(match drive.find(ROOT, PROBE_FOLDER).await? {
        Some(f) => f.id,
        None => {
            drive
                .create_folder(ROOT, PROBE_FOLDER, &Properties::new())
                .await?
                .id
        }
    })
}

async fn print_tree(drive: &impl DriveApi, id: &str, depth: usize) -> Result<(), SyncError> {
    let mut stack: Vec<(RemoteFile, usize)> = drive
        .list(id)
        .await?
        .into_iter()
        .rev()
        .map(|f| (f, depth))
        .collect();
    while let Some((f, d)) = stack.pop() {
        let indent = "  ".repeat(d);
        if f.folder {
            println!("{indent}{}/", f.name);
            for child in drive.list(&f.id).await?.into_iter().rev() {
                stack.push((child, d + 1));
            }
        } else {
            println!(
                "{indent}{}  {} bytes  md5 {}  {}",
                f.name,
                f.size.unwrap_or(0),
                f.md5.as_deref().unwrap_or("?"),
                emudome_sync::time::format_rfc3339(f.modified)
            );
        }
    }
    Ok(())
}

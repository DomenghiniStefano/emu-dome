//! The log file (Trello #44, `about.log` exports it): every log line also goes to
//! `<app local data>/logs/emudome.log`, so a packaged build (no console on Windows, logcat on
//! Android) still has something to attach to a problem report.
//!
//! [`Tee`] wraps the system's logger (stderr / logcat): each line it lets through is formatted on
//! the calling thread and handed to the `emudome-log` thread, which writes it. No thread that logs
//! ever touches the disk (engineering rule 4: the emulation thread and the audio callback log);
//! when the queue is full, lines are dropped and counted. Until the shell knows the data folder
//! ([`attach`]), the lines wait in memory (at most [`EARLY_LINES`]). The previous run's file is
//! kept beside it as `emudome-previous.log`; a run writes at most [`MAX_BYTES`].

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// The file's name in the logs folder.
pub const FILE_NAME: &str = "emudome.log";
/// The previous run's file.
pub const PREVIOUS_NAME: &str = "emudome-previous.log";
/// Lines kept in memory before the folder is known.
pub const EARLY_LINES: usize = 2000;
/// The most one run writes; past it one line says so and the rest is dropped.
pub const MAX_BYTES: u64 = 16 * 1024 * 1024;
/// Lines waiting for the writer.
const QUEUE: usize = 4096;

enum Msg {
    Line(String),
    Attach(PathBuf, std::sync::mpsc::Sender<std::io::Result<PathBuf>>),
}

struct Shared {
    tx: SyncSender<Msg>,
    dropped: AtomicU64,
    path: Mutex<Option<PathBuf>>,
}

static SHARED: OnceLock<Shared> = OnceLock::new();

/// The system's logger, with every line it lets through also written to the file.
pub struct Tee {
    inner: Box<dyn log::Log>,
}

impl Tee {
    /// Wraps `inner` and starts the writer thread (once per process).
    pub fn new(inner: Box<dyn log::Log>) -> Self {
        SHARED.get_or_init(|| {
            let (tx, rx) = sync_channel(QUEUE);
            if let Err(e) = std::thread::Builder::new()
                .name("emudome-log".into())
                .spawn(move || write_loop(rx))
            {
                eprintln!("log file thread not started: {e}");
            }
            Shared {
                tx,
                dropped: AtomicU64::new(0),
                path: Mutex::new(None),
            }
        });
        Self { inner }
    }
}

impl log::Log for Tee {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.inner.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        if !self.inner.enabled(record.metadata()) {
            return;
        }
        self.inner.log(record);
        if let Some(shared) = SHARED.get() {
            let line = format_line(now_ms(), record.level(), record.target(), record.args());
            if let Err(TrySendError::Full(_)) = shared.tx.try_send(Msg::Line(line)) {
                shared.dropped.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    fn flush(&self) {
        self.inner.flush();
    }
}

/// Starts writing the log into `dir` (created if needed): the previous run's file becomes
/// [`PREVIOUS_NAME`], the lines logged so far go first. Waits for the writer; returns the file.
pub fn attach(dir: &Path) -> std::io::Result<PathBuf> {
    let shared = SHARED
        .get()
        .ok_or_else(|| std::io::Error::other("logging not started"))?;
    let (tx, rx) = std::sync::mpsc::channel();
    shared
        .tx
        .send(Msg::Attach(dir.to_path_buf(), tx))
        .map_err(|_| std::io::Error::other("log writer stopped"))?;
    let path = rx
        .recv()
        .map_err(|_| std::io::Error::other("log writer stopped"))??;
    if let Ok(mut slot) = shared.path.lock() {
        *slot = Some(path.clone());
    }
    Ok(path)
}

/// This run's log file, once [`attach`] succeeded.
pub fn current() -> Option<PathBuf> {
    SHARED.get()?.path.lock().ok()?.clone()
}

/// The bytes written so far to this run's file (everything the writer has received is flushed
/// after each line).
pub fn read_current() -> std::io::Result<Vec<u8>> {
    let path = current().ok_or_else(|| std::io::Error::other("no log file in this run"))?;
    std::fs::read(path)
}

fn write_loop(rx: Receiver<Msg>) {
    let mut sink = Sink::default();
    while let Ok(msg) = rx.recv() {
        match msg {
            Msg::Line(line) => sink.line(line),
            Msg::Attach(dir, reply) => {
                // The reply's receiver may be gone: nothing else to tell.
                let _ = reply.send(sink.attach(&dir));
            }
        }
        if let Some(shared) = SHARED.get() {
            let dropped = shared.dropped.swap(0, Ordering::Relaxed);
            if dropped > 0 {
                sink.line(format!(
                    "[log] {dropped} lines dropped (the writer fell behind)\n"
                ));
            }
        }
    }
}

/// Where the writer puts the lines: memory until a file exists.
#[derive(Default)]
struct Sink {
    early: Vec<String>,
    file: Option<File>,
    written: u64,
    full: bool,
}

impl Sink {
    fn line(&mut self, line: String) {
        match &mut self.file {
            None => {
                if self.early.len() < EARLY_LINES {
                    self.early.push(line);
                }
            }
            Some(file) => {
                if self.full {
                    return;
                }
                let (bytes, last) = if self.written + line.len() as u64 > MAX_BYTES {
                    self.full = true;
                    (
                        format!("[log] stopped: this run reached {MAX_BYTES} bytes\n"),
                        true,
                    )
                } else {
                    (line, false)
                };
                // A log line that cannot be written has nowhere else to go but stderr.
                if let Err(e) = file.write_all(bytes.as_bytes()).and_then(|()| file.flush()) {
                    eprintln!("log file not written: {e}");
                }
                if !last {
                    self.written += bytes.len() as u64;
                }
            }
        }
    }

    fn attach(&mut self, dir: &Path) -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(FILE_NAME);
        if path.exists() {
            // The previous run's file is only for the user to look at: losing it is fine.
            let _ = std::fs::rename(&path, dir.join(PREVIOUS_NAME));
        }
        self.file = Some(File::create(&path)?);
        for line in std::mem::take(&mut self.early) {
            self.line(line);
        }
        Ok(path)
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// One line of the file: `2026-11-22T10:15:00.123Z INFO  target: message`.
fn format_line(ms: u64, level: log::Level, target: &str, args: &std::fmt::Arguments<'_>) -> String {
    format!("{} {level:<5} {target}: {args}\n", utc(ms))
}

/// `ms` since the Unix epoch as an ISO 8601 UTC time (no time-zone library: days to a civil
/// date, Howard Hinnant's algorithm).
fn utc(ms: u64) -> String {
    let secs = ms / 1000;
    let days = i64::try_from(secs / 86_400).unwrap_or(0);
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60,
        ms % 1000
    )
}

#[cfg(test)]
mod tests {
    use super::{format_line, utc, Sink, EARLY_LINES, FILE_NAME, MAX_BYTES, PREVIOUS_NAME};

    #[test]
    fn times_are_utc_iso_8601() {
        assert_eq!(utc(0), "1970-01-01T00:00:00.000Z");
        // 2026-11-22T10:15:30.250Z
        assert_eq!(utc(1_795_342_530_250), "2026-11-22T10:15:30.250Z");
        // A leap day.
        assert_eq!(utc(951_782_400_000), "2000-02-29T00:00:00.000Z");
    }

    #[test]
    fn a_line_has_the_time_level_target_and_message() {
        let line = format_line(0, log::Level::Warn, "emudome", &format_args!("x = {}", 1));
        assert_eq!(line, "1970-01-01T00:00:00.000Z WARN  emudome: x = 1\n");
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("emudome-log-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn early_lines_go_first_and_the_previous_file_is_kept() {
        let dir = temp_dir("early");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(FILE_NAME), "old run\n").unwrap();
        let mut sink = Sink::default();
        sink.line("one\n".into());
        sink.line("two\n".into());
        let path = sink.attach(&dir).unwrap();
        sink.line("three\n".into());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "one\ntwo\nthree\n");
        assert_eq!(
            std::fs::read_to_string(dir.join(PREVIOUS_NAME)).unwrap(),
            "old run\n"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn memory_and_the_file_are_bounded() {
        let mut sink = Sink::default();
        for _ in 0..EARLY_LINES + 10 {
            sink.line("x\n".into());
        }
        assert_eq!(sink.early.len(), EARLY_LINES);

        let dir = temp_dir("full");
        let mut sink = Sink::default();
        let path = sink.attach(&dir).unwrap();
        let big = "y".repeat(1024 * 1024) + "\n";
        for _ in 0..(MAX_BYTES / big.len() as u64) + 3 {
            sink.line(big.clone());
        }
        let len = std::fs::metadata(&path).unwrap().len();
        assert!(len <= MAX_BYTES + 100, "{len}");
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.ends_with("bytes\n"), "the last line says it stopped");
        drop(sink);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

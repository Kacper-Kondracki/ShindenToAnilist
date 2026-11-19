use std::{
    env,
    fs::{
        self,
        File,
        OpenOptions,
    },
    io::{
        self,
        Write,
    },
    path::{
        Path,
        PathBuf,
    },
    sync::{
        Arc,
        Mutex,
    },
};

use tracing::{
    error,
    info,
};
use tracing_subscriber::{
    EnvFilter,
    fmt,
};

const MAX_LOG_BYTES: u64 = 1024 * 1024;

pub fn init(component: &'static str, version: &'static str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(
            "off,shinden_to_anilist_core=warn,shinden_to_anilist_grpc=info,shinden_to_anilist_tauri=info",
        )
    });
    let log = log_directory().and_then(|dir| {
        fs::create_dir_all(&dir)?;
        LogFile::open(dir.join(format!("{component}.log")))
    });
    let log = match log {
        Ok(file) => Some(Arc::new(Mutex::new(file))),
        Err(error) => {
            eprintln!("Could not open {component} log: {error}");
            None
        },
    };
    let initialized = fmt()
        .with_ansi(false)
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(move || LogWriter(log.clone()))
        .try_init();
    if initialized.is_err() {
        return;
    }

    let existing_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        let message = panic.payload_as_str().unwrap_or("non-string panic payload");
        if let Some(location) = panic.location() {
            error!(file = location.file(), line = location.line(), %message, "Rust panic");
        } else {
            error!(%message, "Rust panic");
        }
        existing_hook(panic);
    }));

    info!(
        target: "shinden_to_anilist_grpc::diagnostics",
        component,
        version,
        revision = env!("SHINDEN_BUILD_REVISION"),
        os = env::consts::OS,
        architecture = env::consts::ARCH,
        "session started"
    );
}

fn log_directory() -> io::Result<PathBuf> {
    let home = || env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from);
    let local = if cfg!(windows) {
        env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .or_else(|| home().map(|path| path.join("AppData/Local")))
    } else if cfg!(target_os = "macos") {
        home().map(|path| path.join("Library/Logs"))
    } else {
        env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| home().map(|path| path.join(".local/state")))
    };
    local
        .map(|path| path.join("ShindenToAnilist"))
        .map(|path| {
            if cfg!(target_os = "macos") {
                path
            } else {
                path.join("logs")
            }
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no per-user log directory"))
}

struct LogFile {
    path: PathBuf,
}

impl LogFile {
    fn open(path: PathBuf) -> io::Result<Self> {
        append(&path)?;
        Ok(Self { path })
    }

    fn write(&self, entry: &[u8]) -> io::Result<()> {
        let bytes = fs::metadata(&self.path).map_or(0, |metadata| metadata.len());
        if bytes.saturating_add(entry.len() as u64) > MAX_LOG_BYTES {
            let previous = self.path.with_extension("log.1");
            #[cfg(windows)]
            if previous.exists() {
                fs::remove_file(&previous)?;
            }
            fs::rename(&self.path, previous)?;
        }
        append(&self.path)?.write_all(entry)
    }
}

fn append(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

struct LogWriter(Option<Arc<Mutex<LogFile>>>);

impl Write for LogWriter {
    fn write(&mut self, entry: &[u8]) -> io::Result<usize> {
        let persisted = self
            .0
            .as_ref()
            .and_then(|log| log.lock().ok())
            .is_some_and(|log| log.write(entry).is_ok());
        if !persisted || cfg!(debug_assertions) {
            io::stderr().write_all(entry)?;
        }
        Ok(entry.len())
    }

    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

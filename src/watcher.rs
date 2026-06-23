use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Duration,
};

use crossbeam_channel::{bounded, Receiver, Sender};

#[cfg(not(target_arch = "wasm32"))]
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::{error::ConfigError, hot_reload::HotReloadable};

#[derive(Debug, Clone)]
pub enum ReloadSignal {
    Updated,
    ParseError(String),
    PartialUpdate { skipped_fields: Vec<String> },
}

type Callback<T> = Box<dyn Fn(&T) + Send + Sync + 'static>;

pub struct ConfigWatcher<T> {
    state: Arc<RwLock<T>>,
    pub signal_rx: Receiver<ReloadSignal>,
    signal_tx: Sender<ReloadSignal>,
    #[cfg(not(target_arch = "wasm32"))]
    _watcher: RecommendedWatcher,
    pub path: PathBuf,
    callbacks: Arc<RwLock<Vec<Callback<T>>>>,
}

impl<T> ConfigWatcher<T>
where
    T: HotReloadable + serde::de::DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub fn new(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref().to_path_buf();

        #[cfg(not(target_arch = "wasm32"))]
        let initial_config = load_config_from_file::<T>(&path)?;
        #[cfg(target_arch = "wasm32")]
        let initial_config = T::default();

        let state = Arc::new(RwLock::new(initial_config));
        let (signal_tx, signal_rx) = bounded::<ReloadSignal>(4);
        let callbacks = Arc::new(RwLock::new(Vec::new()));

        #[cfg(not(target_arch = "wasm32"))]
        let watcher = {
            let state_clone = Arc::clone(&state);
            let path_clone = path.clone();
            let signal_clone = signal_tx.clone();
            let callbacks_clone = Arc::clone(&callbacks);

            let mut w = notify::recommended_watcher(move |res: notify::Result<Event>| {
                handle_notify_event::<T>(
                    res,
                    &path_clone,
                    &state_clone,
                    &signal_clone,
                    &callbacks_clone,
                );
            })?;
            w.watch(&path, RecursiveMode::NonRecursive)?;
            w
        };

        log::info!(
            "[snapcfg] Watcher initialized (WASM: {}): {}",
            cfg!(target_arch = "wasm32"),
            path.display()
        );

        Ok(Self {
            state,
            signal_rx,
            signal_tx,
            #[cfg(not(target_arch = "wasm32"))]
            _watcher: watcher,
            path,
            callbacks,
        })
    }

    #[inline]
    pub fn config_snapshot(&self) -> T {
        self.state.read().expect("RwLock poisoned").clone()
    }

    #[inline]
    pub fn try_recv(&self) -> Option<ReloadSignal> {
        self.signal_rx.try_recv().ok()
    }

    #[inline]
    pub fn state(&self) -> &Arc<RwLock<T>> {
        &self.state
    }

    pub fn on_reload(&self, callback: impl Fn(&T) + Send + Sync + 'static) {
        self.callbacks
            .write()
            .expect("Callbacks RwLock poisoned")
            .push(Box::new(callback));
    }

    pub fn reload_from_str(&self, raw: &str) -> Result<(), ConfigError> {
        let table: toml::Table = toml::from_str(raw).map_err(ConfigError::Parse)?;

        let cfg_clone = {
            let mut cfg = self.state.write().expect("RwLock poisoned");
            cfg.apply_toml(&table);
            cfg.clone()
        };

        let callbacks = self.callbacks.read().expect("Callbacks RwLock poisoned");
        for cb in callbacks.iter() {
            cb(&cfg_clone);
        }

        let _ = self.signal_tx.try_send(ReloadSignal::Updated);
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn handle_notify_event<T>(
    res: notify::Result<Event>,
    path: &Path,
    state: &Arc<RwLock<T>>,
    signal: &Sender<ReloadSignal>,
    callbacks: &Arc<RwLock<Vec<Callback<T>>>>,
) where
    T: HotReloadable + Clone + Send + Sync + 'static,
{
    let event = match res {
        Ok(e) => e,
        Err(e) => {
            log::error!("[snapcfg] notify error: {e}");
            return;
        }
    };

    let is_write = matches!(
        event.kind,
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
    );

    if !is_write {
        return;
    }

    if matches!(event.kind, EventKind::Remove(_)) {
        std::thread::sleep(Duration::from_millis(80));
    }

    std::thread::sleep(Duration::from_millis(30));

    let raw = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            log::warn!("[snapcfg] Failed to read file: {e}");
            return;
        }
    };

    let table: toml::Table = match toml::from_str(&raw) {
        Ok(t) => t,
        Err(e) => {
            let msg = e.to_string();
            log::warn!("[snapcfg] TOML syntax error — KEPT ALL OLD VALUES: {msg}");
            let _ = signal.try_send(ReloadSignal::ParseError(msg));
            return;
        }
    };

    let cfg_clone = {
        let mut cfg = state.write().expect("RwLock poisoned");
        cfg.apply_toml(&table);
        cfg.clone()
    };

    let callbacks_lock = callbacks.read().expect("Callbacks RwLock poisoned");
    for cb in callbacks_lock.iter() {
        cb(&cfg_clone);
    }

    log::info!(
        "[snapcfg] Config field-level updated: {}",
        path.display()
    );
    let _ = signal.try_send(ReloadSignal::Updated);
}


#[cfg(not(target_arch = "wasm32"))]
fn load_config_from_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ConfigError> {
    let raw = fs::read_to_string(path)?;
    let cfg = toml::from_str(&raw)?;
    Ok(cfg)
}

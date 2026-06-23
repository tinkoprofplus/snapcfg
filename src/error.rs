use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    Parse(#[from] toml::de::Error),

    #[cfg(not(target_arch = "wasm32"))]
    #[error("Failed to initialize file watcher: {0}")]
    Watcher(#[from] notify::Error),

    #[error("Watcher thread crashed")]
    WatcherDead,

}

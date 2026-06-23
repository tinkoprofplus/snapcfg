extern crate self as snapcfg;

pub mod config;
pub mod error;
pub mod hot_reload;
pub mod watcher;

pub use config::{GameConfig, PlayerConfig, RendererConfig, WorldConfig};
pub use error::ConfigError;
pub use hot_reload::HotReloadable;
pub use watcher::{ConfigWatcher, ReloadSignal};

pub use snapcfg_macros::HotReload;

#[doc(hidden)]
pub mod __private {
    pub use crate::hot_reload::HotReloadable;
}

use serde::Deserialize;
use snapcfg_macros::HotReload;

#[derive(Debug, Clone, Deserialize, Default, HotReload)]
pub struct GameConfig {
    #[nested]
    pub player: PlayerConfig,
    #[nested]
    pub world: WorldConfig,
    #[nested]
    pub renderer: RendererConfig,
}

#[derive(Debug, Clone, Deserialize, HotReload)]
pub struct PlayerConfig {
    pub health: f32,
    pub speed: f32,
    pub jump_force: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            health: 100.0,
            speed: 5.0,
            jump_force: 8.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, HotReload)]
pub struct WorldConfig {
    pub enemy_count: u32,
    pub gravity: f32,
    pub debug_enabled: bool,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            enemy_count: 10,
            gravity: -9.81,
            debug_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, HotReload)]
pub struct RendererConfig {
    pub target_fps: u32,
    pub fov: f32,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            target_fps: 60,
            fov: 90.0,
        }
    }
}

impl GameConfig {
    pub fn from_toml_str(raw: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(raw)
    }
}

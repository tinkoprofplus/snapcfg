# snapcfg 🔥

[![Rust CI](https://github.com/your-username/snapcfg/actions/workflows/ci.yml/badge.svg)](https://github.com/your-username/snapcfg/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A minimal, zero-overhead **Hot-Reloading Config** library for game loops.

*Read this in other languages: [English](README.md), [Türkçe](README.tr.md), [Русский](README.ru.md), [Français](README.fr.md), [Español](README.es.md)*

```
edit config.toml while the game is running → changes are reflected in the game loop instantly
```

## Architecture

```
┌─────────────┐  inotify event  ┌─────────────────────────┐
│  OS / inode │ ──────────────► │  notify (watcher thread) │
└─────────────┘                 │  ① reads file            │
                                │  ② parses TOML           │
                                │  ③ writes to RwLock      │
                                └────────────┬────────────┘
                                             │ crossbeam signal (non-blocking)
                                ┌────────────▼────────────┐
                                │  Game Loop (main thread) │
                                │  try_recv() → instant    │
                                │  reads config_snapshot() │
                                └─────────────────────────┘
```

**The main thread is never blocked by file I/O.**

## Quick Start

```bash
git clone <repo>
cd snapcfg
cargo run
```

While the program is running, in another terminal:

```bash
sed -i 's/speed      = 5.5/speed      = 22.0/' config.toml
```

You will immediately see the message `🔥 CONFIG GÜNCELLENDİ!` (Config updated) in the terminal.

## Dependencies

| Crate | Version | Reason |
|---|---|---|
| `notify` | 6.x | Cross-platform inotify/FSEvents/ReadDirectoryChangesW wrapper |
| `serde` | 1.x | Zero-cost TOML deserialization |
| `toml` | 0.8 | TOML 1.0 compliant parser |
| `crossbeam-channel` | 0.5 | Capacity-limited, lock-free signal channel |
| `thiserror` | 1.x | Ergonomic error types |
| `log` + `env_logger` | 0.4/0.11 | Configurable logging |

## Directory Structure

```
snapcfg/
├── Cargo.toml
├── config.toml          ← Monitored game configuration
└── src/
    ├── lib.rs           ← Public API re-exports
    ├── main.rs          ← Simulated game loop demo
    ├── config.rs        ← GameConfig, PlayerConfig, etc. structs
    ├── watcher.rs       ← ConfigWatcher engine
    └── error.rs         ← ConfigError type
```

## Usage (as a Library)

### 1. Define Your Configuration

Derive `HotReload` on your custom configuration structs. Use the `#[nested]` attribute for child configuration structures to propagate field-level graceful reload.

```rust
use serde::Deserialize;
use snapcfg::HotReload;

#[derive(Debug, Clone, Deserialize, Default, HotReload)]
struct MyConfig {
    #[nested]
    player: PlayerConfig,
    debug: bool,
}

#[derive(Debug, Clone, Deserialize, Default, HotReload)]
struct PlayerConfig {
    speed: f32,
    health: f32,
}
```

### 2. Monitor and Reactive Callbacks

Initialize a generic `ConfigWatcher` and optionally register callbacks that trigger on reload:

```rust
use snapcfg::{ConfigWatcher, ReloadSignal};

fn main() {
    // Specify your custom configuration type T in the turbofish
    let watcher = ConfigWatcher::<MyConfig>::new("config.toml").unwrap();
    let mut cfg = watcher.config_snapshot();

    // Register active reactive hooks
    watcher.on_reload(|new_cfg| {
        println!("Config updated reactively in background! New speed: {}", new_cfg.player.speed);
    });

    loop { // game loop
        // Non-blocking channel signal check
        if let Some(ReloadSignal::Updated) = watcher.try_recv() {
            cfg = watcher.config_snapshot();
        }

        // ... game logic using cfg.player.speed ...
    }
}
```

### 3. WASM & Memory Reloading (Web / Testing)

`snapcfg` compiles out-of-the-box on WASM targets (where filesystem OS watchers are disabled). You can manually push TOML data fetched over HTTP/network using `reload_from_str`:

```rust
// Works on WASM & native test environments
watcher.reload_from_str("debug = true\n[player]\nspeed = 42.0").unwrap();
```

## Features & Roadmap

- [x] Generic `ConfigWatcher<T>` supporting any custom struct
- [x] Auto-struct derivation macro `#[derive(HotReload)]` with `#[nested]` attribute support
- [x] `on_reload(callback)` reactive hooks
- [x] WASM target compatibility (non-blocking manual string reloading API)
- [ ] Multi-file monitoring support
- [ ] Lua/Rhai scripting integration

## Testing

You can run the full suite of integration tests covering the graceful fallback, field-level parsing validation, nested struct propagation, and reactive callback systems:

```bash
cargo test
```

Test suite execution output:
```text
     Running tests/derive_hot_reload.rs (target/debug/deps/derive_hot_reload-2404254ac3f414f1)

running 7 tests
test test_empty_toml_noop ... ok
test test_apply_toml_str_wrapper ... ok
test test_missing_field_keeps_old_value ... ok
test test_basic_field_update ... ok
test test_nested_struct_reload ... ok
test test_wrong_type_keeps_old_value ... ok
test test_watcher_callbacks_and_generic_reload ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## License

MIT


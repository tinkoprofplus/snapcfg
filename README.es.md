# snapcfg 🔥

[![Rust CI](https://github.com/your-username/snapcfg/actions/workflows/ci.yml/badge.svg)](https://github.com/your-username/snapcfg/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Una biblioteca de **recarga en caliente (Hot-Reloading)** de configuración mínima y sin sobrecoste (zero-overhead) para bucles de juego.

*Leer en otros idiomas: [English](README.md), [Türkçe](README.tr.md), [Русский](README.ru.md), [Français](README.fr.md), [Español](README.es.md)*

```
edita config.toml mientras el juego se ejecuta → los cambios se reflejan en el bucle de juego al instante
```

## Arquitectura

```
┌─────────────┐  evento inotify   ┌─────────────────────────┐
│  SO / inode │ ────────────────► │  notify (hilo observad.)│
└─────────────┘                   │  ① lee el archivo       │
                                  │  ② analiza el TOML      │
                                  │  ③ escribe en RwLock    │
                                  └────────────┬────────────┘
                                               │ señal crossbeam (no bloqueante)
                                  ┌────────────▼────────────┐
                                  │ Bucle de juego (principal)
                                  │  try_recv() → instantáneo
                                  │ lee config_snapshot()   │
                                  └─────────────────────────┘
```

**El hilo principal nunca se bloquea por la E/S de archivos.**

## Inicio rápido

```bash
git clone <repo>
cd snapcfg
cargo run
```

Mientras el programa se está ejecutando, en otra terminal:

```bash
sed -i 's/speed      = 5.5/speed      = 22.0/' config.toml
```

Verás inmediatamente el mensaje `🔥 CONFIG GÜNCELLENDİ!` (Configuración actualizada) en la terminal.

## Dependencias

| Crate | Versión | Razón |
|---|---|---|
| `notify` | 6.x | Wrapper multiplataforma para inotify/FSEvents/ReadDirectoryChangesW |
| `serde` | 1.x | Deserialización TOML sin sobrecoste (zero-cost) |
| `toml` | 0.8 | Analizador compatible con TOML 1.0 |
| `crossbeam-channel` | 0.5 | Canal de señalización sin bloqueos (lock-free) y de capacidad limitada |
| `thiserror` | 1.x | Tipos de error ergonómicos |
| `log` + `env_logger` | 0.4/0.11 | Registro (logging) configurable |

## Estructura de archivos

```
snapcfg/
├── Cargo.toml
├── config.toml          ← Configuración de juego monitoreada
└── src/
    ├── lib.rs           ← Reexportaciones de la API pública
    ├── main.rs          ← Demo del bucle de juego simulado
    ├── config.rs        ← Estructuras GameConfig, PlayerConfig, etc.
    ├── watcher.rs       ← Motor ConfigWatcher
    └── error.rs         ← Tipo ConfigError
```

## Uso (como biblioteca)

### 1. Defina su configuración

Derive `HotReload` en sus estructuras de configuración personalizadas. Utilice el atributo `#[nested]` para las estructuras de configuración hijas para propagar la recarga tolerante a fallos a nivel de campo.

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

### 2. Monitoreo y callbacks reactivos

Inicialice un `ConfigWatcher` genérico y, opcionalmente, registre callbacks que se activen al recargar:

```rust
use snapcfg::{ConfigWatcher, ReloadSignal};

fn main() {
    // Especifique su tipo de configuración personalizada T en el turbofish
    let watcher = ConfigWatcher::<MyConfig>::new("config.toml").unwrap();
    let mut cfg = watcher.config_snapshot();

    // Registrar hooks reactivos activos
    watcher.on_reload(|new_cfg| {
        println!("¡Configuración actualizada de forma reactiva en segundo plano! Nueva velocidad: {}", new_cfg.player.speed);
    });

    loop { // bucle de juego
        // Comprobación no bloqueante del canal de señal
        if let Some(ReloadSignal::Updated) = watcher.try_recv() {
            cfg = watcher.config_snapshot();
        }

        // ... lógica del juego con cfg.player.speed ...
    }
}
```

### 3. WASM y recarga en memoria (Web / Pruebas)

`snapcfg` se compila sin problemas en objetivos WASM (donde los observadores del sistema de archivos del SO están desactivados). Puede enviar manualmente datos TOML obtenidos a través de HTTP/red utilizando `reload_from_str`:

```rust
// Funciona en entornos WASM y de prueba nativos
watcher.reload_from_str("debug = true\n[player]\nspeed = 42.0").unwrap();
```

## Características y hoja de ruta

- [x] `ConfigWatcher<T>` genérico que admite cualquier estructura personalizada
- [x] Macro de derivación automática de estructuras `#[derive(HotReload)]` con soporte para el atributo `#[nested]`
- [x] Ganchos (hooks) reactivos `on_reload(callback)`
- [x] Compatibilidad con WASM (API de recarga manual de cadenas no bloqueante)
- [ ] Soporte para monitoreo de múltiples archivos
- [ ] Integración con scripts de Lua/Rhai

## Pruebas

Puede ejecutar el conjunto completo de pruebas de integración que cubren la tolerancia a fallos, la validación del análisis a nivel de campo, la propagación de estructuras anidadas y los sistemas de callbacks reactivos:

```bash
cargo test
```

Resultado de la ejecución de las pruebas:
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

## Licencia

MIT


# snapcfg 🔥

[![Rust CI](https://github.com/your-username/snapcfg/actions/workflows/ci.yml/badge.svg)](https://github.com/your-username/snapcfg/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Une bibliothèque de **rechargement à chaud (Hot-Reloading)** de configuration minimale et sans surcoût (zero-overhead) pour les boucles de jeu.

*Lire en d'autres langues: [English](README.md), [Türkçe](README.tr.md), [Русский](README.ru.md), [Français](README.fr.md), [Español](README.es.md)*

```
modifiez config.toml pendant que le jeu tourne → les modifications sont appliquées instantanément dans la boucle de jeu
```

## Architecture

```
┌─────────────┐  événement inotify  ┌─────────────────────────┐
│  OS / inode │ ──────────────────► │  notify (thread d'obs.) │
└─────────────┘                     │  ① lit le fichier       │
                                    │  ② analyse le TOML      │
                                    │  ③ écrit dans RwLock    │
                                    └────────────┬────────────┘
                                                 │ signal crossbeam (non bloquant)
                                    ┌────────────▼────────────┐
                                    │ Boucle de jeu (principal)│
                                    │  try_recv() → instantané│
                                    │ lit config_snapshot()   │
                                    └─────────────────────────┘
```

**Le thread principal n'est jamais bloqué par les E/S de fichiers.**

## Démarrage rapide

```bash
git clone <repo>
cd snapcfg
cargo run
```

Pendant que le programme s'exécute, dans un autre terminal :

```bash
sed -i 's/speed      = 5.5/speed      = 22.0/' config.toml
```

Vous verrez immédiatement le message `🔥 CONFIG GÜNCELLENDİ!` (Configuration mise à jour) dans le terminal.

## Dépendances

| Crate | Version | Raison |
|---|---|---|
| `notify` | 6.x | Wrapper multiplateforme pour inotify/FSEvents/ReadDirectoryChangesW |
| `serde` | 1.x | Désérialisation TOML sans surcoût (zero-cost) |
| `toml` | 0.8 | Analyseur compatible TOML 1.0 |
| `crossbeam-channel` | 0.5 | Canal de signalisation sans verrouillage (lock-free) à capacité limitée |
| `thiserror` | 1.x | Types d'erreurs ergonomiques |
| `log` + `env_logger` | 0.4/0.11 | Journalisation configurable |

## Structure des fichiers

```
snapcfg/
├── Cargo.toml
├── config.toml          ← Configuration du jeu surveillée
└── src/
    ├── lib.rs           ← Ré-exports de l'API publique
    ├── main.rs          ← Démo de la boucle de jeu simulée
    ├── config.rs        ← Structures GameConfig, PlayerConfig, etc.
    ├── watcher.rs       ← Moteur ConfigWatcher
    └── error.rs         ← Type ConfigError
```

## Utilisation (en tant que bibliothèque)

### 1. Définissez votre configuration

Dérivez `HotReload` sur vos structures de configuration personnalisées. Utilisez l'attribut `#[nested]` pour les structures de configuration enfants afin de propager le rechargement tolérant aux pannes au niveau du champ.

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

### 2. Surveillance et callbacks réactifs

Initialisez un `ConfigWatcher` générique et, en option, enregistrez des callbacks qui se déclenchent lors du rechargement :

```rust
use snapcfg::{ConfigWatcher, ReloadSignal};

fn main() {
    // Spécifiez votre type de configuration personnalisée T dans le turbofish
    let watcher = ConfigWatcher::<MyConfig>::new("config.toml").unwrap();
    let mut cfg = watcher.config_snapshot();

    // Enregistrer des hooks réactifs actifs
    watcher.on_reload(|new_cfg| {
        println!("Configuration mise à jour de manière réactive en arrière-plan ! Nouvelle vitesse : {}", new_cfg.player.speed);
    });

    loop { // boucle de jeu
        // Contrôle non bloquant du canal de signal
        if let Some(ReloadSignal::Updated) = watcher.try_recv() {
            cfg = watcher.config_snapshot();
        }

        // ... logique du jeu avec cfg.player.speed ...
    }
}
```

### 3. WASM & Rechargement en mémoire (Web / Tests)

`snapcfg` se compile sans problème sur les cibles WASM (où les observateurs du système de fichiers de l'OS sont désactivés). Vous pouvez envoyer manuellement des données TOML récupérées par HTTP/réseau en utilisant `reload_from_str`:

```rust
// Fonctionne dans les environnements WASM et de test natifs
watcher.reload_from_str("debug = true\n[player]\nspeed = 42.0").unwrap();
```

## Fonctionnalités et feuille de route

- [x] `ConfigWatcher<T>` générique prenant en charge n'importe quelle structure personnalisée
- [x] Macro de dérivation automatique de structure `#[derive(HotReload)]` avec prise en charge de l'attribut `#[nested]`
- [x] Hooks réactifs `on_reload(callback)`
- [x] Compatibilité avec la cible WASM (API de rechargement manuel de chaînes non bloquante)
- [ ] Prise en charge de la surveillance de plusieurs fichiers
- [ ] Intégration de scripts Lua/Rhai

## Tests

Vous pouvez exécuter la suite complète de tests d'intégration couvrant le rechargement tolérant aux pannes, la validation de l'analyse au niveau du champ, la propagation des structures imbriquées et les systèmes de callbacks réactifs :

```bash
cargo test
```

Résultat de l'exécution de la suite de tests :
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

## Licence

MIT


# snapcfg 🔥

[![Rust CI](https://github.com/your-username/snapcfg/actions/workflows/ci.yml/badge.svg)](https://github.com/your-username/snapcfg/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Минималистичная библиотека **Hot-Reloading конфигурации** с нулевыми накладными расходами (zero-overhead) для игровых циклов.

*Читать на других языках: [English](README.md), [Türkçe](README.tr.md), [Русский](README.ru.md), [Français](README.fr.md), [Español](README.es.md)*

```
редактируйте config.toml во время игры → изменения мгновенно отражаются в игровом цикле
```

## Архитектура

```
┌─────────────┐  событие inotify  ┌─────────────────────────┐
│  ОС / inode │ ────────────────► │  notify (поток наблюд.) │
└─────────────┘                   │  ① читает файл          │
                                  │  ② парсит TOML          │
                                  │  ③ пишет в RwLock       │
                                  └────────────┬────────────┘
                                               │ сигнал crossbeam (неблокирующий)
                                  ┌────────────▼────────────┐
                                  │ Игровой цикл (главный)  │
                                  │  try_recv() → мгновенно │
                                  │ читает config_snapshot()│
                                  └─────────────────────────┘
```

**Главный поток никогда не блокируется файловым вводом-выводом (I/O).**

## Быстрый старт

```bash
git clone <repo>
cd snapcfg
cargo run
```

Пока программа работает, в другом терминале:

```bash
sed -i 's/speed      = 5.5/speed      = 22.0/' config.toml
```

Вы сразу увидите сообщение `🔥 CONFIG GÜNCELLENDİ!` (Конфигурация обновлена) в терминале.

## Зависимости

| Crate | Версия | Причина |
|---|---|---|
| `notify` | 6.x | Кроссплатформенная обертка для inotify/FSEvents/ReadDirectoryChangesW |
| `serde` | 1.x | Десериализация TOML с нулевой стоимостью (zero-cost) |
| `toml` | 0.8 | Парсер, совместимый с TOML 1.0 |
| `crossbeam-channel` | 0.5 | Неблокирующий канал сигналов с ограниченной емкостью |
| `thiserror` | 1.x | Эргономичные типы ошибок |
| `log` + `env_logger` | 0.4/0.11 | Настраиваемое логирование |

## Структура проекта

```
snapcfg/
├── Cargo.toml
├── config.toml          ← Отслеживаемые настройки игры
└── src/
    ├── lib.rs           ← Экспорт публичного API
    ├── main.rs          ← Демо игрового цикла
    ├── config.rs        ← Структуры GameConfig, PlayerConfig и др.
    ├── watcher.rs       ← Движок ConfigWatcher
    └── error.rs         ← Тип ошибок ConfigError
```

## Использование (как библиотека)

### 1. Определение конфигурации

Добавьте макрос `HotReload` для ваших структур конфигурации. Используйте атрибут `#[nested]` для вложенных структур, чтобы распространить пошаговое восстановление ошибок на уровне полей.

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

### 2. Отслеживание и реактивные колбэки

Инициализируйте общий `ConfigWatcher` и при необходимости зарегистрируйте колбэки, запускаемые при перезагрузке:

```rust
use snapcfg::{ConfigWatcher, ReloadSignal};

fn main() {
    // Укажите свой тип конфигурации T в турбо-рыбе (turbofish)
    let watcher = ConfigWatcher::<MyConfig>::new("config.toml").unwrap();
    let mut cfg = watcher.config_snapshot();

    // Регистрация активных реактивных хуков
    watcher.on_reload(|new_cfg| {
        println!("Конфигурация реактивно обновлена в фоне! Новая скорость: {}", new_cfg.player.speed);
    });

    loop { // игровой цикл
        // Неблокирующая проверка канала сигналов
        if let Some(ReloadSignal::Updated) = watcher.try_recv() {
            cfg = watcher.config_snapshot();
        }

        // ... игровая логика с использованием cfg.player.speed ...
    }
}
```

### 3. WASM и перезагрузка из памяти (Web / Тестирование)

`snapcfg` собирается без проблем на целях WASM (где наблюдатели файловой системы ОС отключены). Вы можете вручную передавать данные TOML, полученные по сети/HTTP, с помощью `reload_from_str`:

```rust
// Работает в WASM и нативных средах тестирования
watcher.reload_from_str("debug = true\n[player]\nspeed = 42.0").unwrap();
```

## Функции и план развития

- [x] Универсальный `ConfigWatcher<T>` с поддержкой любых пользовательских структур
- [x] Макрос авто-вывода `#[derive(HotReload)]` с поддержкой атрибута `#[nested]`
- [x] Реактивные хуки `on_reload(callback)`
- [x] Совместимость с WASM (неблокирующий API ручной перезагрузки из TOML-строки)
- [ ] Поддержка отслеживания нескольких файлов
- [ ] Интеграция со скриптами Lua/Rhai

## Тестирование

Вы можете запустить полный набор интеграционных тестов, проверяющих мягкое восстановление (graceful fallback), валидацию типов на уровне полей, вложенные структуры и реактивные колбэки:

```bash
cargo test
```

Результат выполнения тестов:
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

## Лицензия

MIT


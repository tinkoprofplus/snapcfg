# snapcfg 🔥

[![Rust CI](https://github.com/your-username/snapcfg/actions/workflows/ci.yml/badge.svg)](https://github.com/your-username/snapcfg/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Oyun döngüleri (game loops) için minimal, sıfır-overhead **Sıcak-Yüklemeli (Hot-Reloading) Config** kütüphanesi.

*Diğer dillerde okuyun: [English](README.md), [Türkçe](README.tr.md), [Русский](README.ru.md), [Français](README.fr.md), [Español](README.es.md)*

```
oyun çalışırken config.toml dosyasını düzenle → değişiklikler anında oyun döngüsüne yansısın
```

## Mimari

```
┌─────────────┐  inotify olayı  ┌─────────────────────────┐
│  OS / inode │ ──────────────► │  notify (izleyici thread)│
└─────────────┘                 │  ① dosyayı okur          │
                                │  ② TOML ayrıştırır       │
                                │  ③ RwLock'a yazar        │
                                └────────────┬────────────┘
                                             │ crossbeam sinyali (bloklamayan)
                                ┌────────────▼────────────┐
                                │  Game Loop (ana thread)  │
                                │  try_recv() → anında     │
                                │  config_snapshot() okur  │
                                └─────────────────────────┘
```

**Ana thread hiçbir zaman dosya IO'sunda bloklanmaz.**

## Hızlı Başlangıç

```bash
git clone <repo>
cd snapcfg
cargo run
```

Program çalışırken başka bir terminalde:

```bash
sed -i 's/speed      = 5.5/speed      = 22.0/' config.toml
```

Terminalde `🔥 CONFIG GÜNCELLENDİ!` mesajını görürsünüz.

## Bağımlılıklar

| Crate | Versiyon | Neden |
|---|---|---|
| `notify` | 6.x | Cross-platform inotify/FSEvents/ReadDirectoryChangesW sarmalayıcısı |
| `serde` | 1.x | Sıfır-maliyetli (zero-cost) TOML serileştirmeden çıkarma (deserialization) |
| `toml` | 0.8 | TOML 1.0 uyumlu ayrıştırıcı (parser) |
| `crossbeam-channel` | 0.5 | Kapasite-sınırlı, kilit gerektirmeyen (lock-free) sinyal kanalı |
| `thiserror` | 1.x | Ergonomik hata türleri |
| `log` + `env_logger` | 0.4/0.11 | Yapılandırılabilir günlükleme (logging) |

## Dosya Yapısı

```
snapcfg/
├── Cargo.toml
├── config.toml          ← İzlenen oyun ayarları
└── src/
    ├── lib.rs           ← Genel API dışa aktarımları
    ├── main.rs          ← Örnek oyun döngüsü demosu
    ├── config.rs        ← GameConfig, PlayerConfig vb. yapılar (structs)
    ├── watcher.rs       ← ConfigWatcher motoru
    └── error.rs         ← ConfigError türü
```

## Kullanım (Kütüphane Olarak)

### 1. Yapılandırmanızı Tanımlayın

Kendi yapılandırma struct'larınız için `HotReload` özelliğini derive edin. Alt yapılandırma alanları için `#[nested]` niteliğini kullanarak alan düzeyinde hata toleranslı yüklemeyi aktifleştirin.

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

### 2. İzleme ve Reaktif Callback'ler

Generic `ConfigWatcher` yapısını başlatın ve isteğe bağlı olarak yükleme anında tetiklenecek callback'leri kaydedin:

```rust
use snapcfg::{ConfigWatcher, ReloadSignal};

fn main() {
    // Turbofish ile kendi config struct tipinizi belirtin
    let watcher = ConfigWatcher::<MyConfig>::new("config.toml").unwrap();
    let mut cfg = watcher.config_snapshot();

    // Reaktif callback kaydedin
    watcher.on_reload(|new_cfg| {
        println!("Config arka planda güncellendi! Yeni hız: {}", new_cfg.player.speed);
    });

    loop { // oyun döngüsü
        // Bloklamayan sinyal kontrolü
        if let Some(ReloadSignal::Updated) = watcher.try_recv() {
            cfg = watcher.config_snapshot();
        }

        // ... oyun mantığı cfg.player.speed ile ...
    }
}
```

### 3. WASM ve Bellek İçi Yükleme (Web / Test)

`snapcfg` WASM hedeflerinde (dosya sistemi izleyicisi devre dışıyken) doğrudan derlenir. HTTP üzerinden çektiğiniz TOML verisini `reload_from_str` ile elinizle besleyebilirsiniz:

```rust
// WASM ve yerel test ortamlarında çalışır
watcher.reload_from_str("debug = true\n[player]\nspeed = 42.0").unwrap();
```

## Özellikler & Yol Haritası

- [x] Herhangi bir özel struct'ı destekleyen generic `ConfigWatcher<T>`
- [x] `#[nested]` niteliği destekli otomatik struct türetme makrosu `#[derive(HotReload)]`
- [x] Reaktif hook'lar sunan `on_reload(callback)` API'si
- [x] WASM hedef uyumluluğu (bloklamayan manuel string yükleme API'si)
- [ ] Çoklu dosya izleme desteği
- [ ] Lua/Rhai betik dili entegrasyonu

## Testler

Hata toleransı (graceful fallback), alan düzeyinde ayrıştırma doğrulama, iç içe geçmiş (nested) yapıların tetiklenmesi ve reaktif callback sistemlerini kapsayan entegrasyon testlerini çalıştırabilirsiniz:

```bash
cargo test
```

Test çıktısı:
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

## Lisans

MIT


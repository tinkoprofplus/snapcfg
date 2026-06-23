use snapcfg::{HotReload, HotReloadable};

#[derive(Debug, Clone, Default, HotReload, serde::Deserialize)]
struct EnemyConfig {
    count: u32,
    speed: f32,
    aggro_range: f32,
}

#[test]
fn test_basic_field_update() {
    let mut cfg = EnemyConfig {
        count: 10,
        speed: 3.0,
        aggro_range: 15.0,
    };

    let toml_str = r#"
        count       = 25
        speed       = 7.5
        aggro_range = 30.0
    "#;

    let table: toml::Table = toml::from_str(toml_str).unwrap();
    cfg.apply_toml(&table);

    assert_eq!(cfg.count, 25);
    assert_eq!(cfg.speed, 7.5);
    assert_eq!(cfg.aggro_range, 30.0);
}

#[test]
fn test_wrong_type_keeps_old_value() {
    let mut cfg = EnemyConfig {
        count: 10,
        speed: 3.0,
        aggro_range: 15.0,
    };

    let toml_str = r#"
        count = 99
        speed = "yirmiki"
        aggro_range = 20.0
    "#;

    let table: toml::Table = toml::from_str(toml_str).unwrap();
    cfg.apply_toml(&table);

    assert_eq!(cfg.count, 99);
    assert_eq!(cfg.aggro_range, 20.0);
    assert_eq!(cfg.speed, 3.0);
}

#[test]
fn test_missing_field_keeps_old_value() {
    let mut cfg = EnemyConfig {
        count: 10,
        speed: 3.0,
        aggro_range: 15.0,
    };

    let toml_str = r#"count = 50"#;
    let table: toml::Table = toml::from_str(toml_str).unwrap();
    cfg.apply_toml(&table);

    assert_eq!(cfg.count, 50);
    assert_eq!(cfg.speed, 3.0);
    assert_eq!(cfg.aggro_range, 15.0);
}

#[test]
fn test_empty_toml_noop() {
    let mut cfg = EnemyConfig {
        count: 10,
        speed: 3.0,
        aggro_range: 15.0,
    };
    let table: toml::Table = toml::from_str("").unwrap();
    cfg.apply_toml(&table);

    assert_eq!(cfg.count, 10);
    assert_eq!(cfg.speed, 3.0);
    assert_eq!(cfg.aggro_range, 15.0);
}

#[test]
fn test_apply_toml_str_wrapper() {
    let mut cfg = EnemyConfig::default();
    cfg.apply_toml_str("count = 42\nspeed = 9.9").unwrap();
    assert_eq!(cfg.count, 42);
    assert_eq!(cfg.speed, 9.9);
}

#[derive(Debug, Clone, Default, HotReload, serde::Deserialize)]
struct NestedGameConfig {
    #[nested]
    enemy: EnemyConfig,
    debug: bool,
}

#[test]
fn test_nested_struct_reload() {
    let mut cfg = NestedGameConfig::default();
    let toml_str = r#"
        debug = true
        [enemy]
        count = 77
        speed = "incorrect-type-should-keep-old-value"
        aggro_range = 88.8
    "#;
    let table: toml::Table = toml::from_str(toml_str).unwrap();
    cfg.apply_toml(&table);

    assert!(cfg.debug);
    assert_eq!(cfg.enemy.count, 77);
    assert_eq!(cfg.enemy.speed, 0.0);
    assert_eq!(cfg.enemy.aggro_range, 88.8);
}

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[test]
fn test_watcher_callbacks_and_generic_reload() {
    let tmp = tempfile::NamedTempFile::with_suffix(".toml").unwrap();
    std::io::Write::write_all(
        &mut tmp.as_file(),
        b"count = 1\nspeed = 1.0\naggro_range = 5.0",
    )
    .unwrap();

    let watcher = snapcfg::ConfigWatcher::<EnemyConfig>::new(tmp.path()).unwrap();
    let callback_count = Arc::new(AtomicU32::new(0));

    let callback_count_clone = Arc::clone(&callback_count);
    watcher.on_reload(move |new_cfg| {
        assert_eq!(new_cfg.count, 12);
        callback_count_clone.fetch_add(1, Ordering::SeqCst);
    });

    watcher
        .reload_from_str("count = 12\nspeed = 3.0\naggro_range = 10.0")
        .unwrap();

    assert_eq!(callback_count.load(Ordering::SeqCst), 1);
    let snapshot = watcher.config_snapshot();
    assert_eq!(snapshot.count, 12);
    assert_eq!(snapshot.speed, 3.0);
}

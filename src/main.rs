use std::{
    thread,
    time::{Duration, Instant},
};

use snapcfg::{ConfigWatcher, GameConfig, ReloadSignal};

const TARGET_FPS: u64 = 10;
const RUN_DURATION_SECS: u64 = 60;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    print_banner();

    let watcher = ConfigWatcher::<GameConfig>::new("config.toml").unwrap_or_else(|e| {
        eprintln!("[ERROR] Failed to start ConfigWatcher: {e}");
        std::process::exit(1);
    });

    watcher.on_reload(|new_config| {
        log::info!(
            "[CALLBACK TRIGGERED] New player speed updated in background: {}",
            new_config.player.speed
        );
    });

    let mut config: GameConfig = watcher.config_snapshot();
    println!("  Initial config (Serde full-parse):\n{:#?}\n", config);

    let frame_duration = Duration::from_millis(1000 / TARGET_FPS);
    let run_until = Instant::now() + Duration::from_secs(RUN_DURATION_SECS);
    let mut frame: u64 = 0;

    while Instant::now() < run_until {
        let frame_start = Instant::now();
        frame += 1;

        match watcher.try_recv() {
            Some(ReloadSignal::Updated) => {
                config = watcher.config_snapshot();
                println!(
                    "\n  🔥 [Frame {frame}] CONFIG UPDATED (field-based)!\n\
                     \t  player.speed      = {}\n\
                     \t  player.health     = {}\n\
                     \t  world.enemy_count = {}\n\
                     \t  renderer.fov      = {}\n",
                    config.player.speed,
                    config.player.health,
                    config.world.enemy_count,
                    config.renderer.fov,
                );
            }

            Some(ReloadSignal::ParseError(msg)) => {
                println!(
                    "\n  🚨 [Frame {frame}] TOML SYNTAX ERROR — \
                     ALL PREVIOUS VALUES PRESERVED!\n  Error: {msg}\n"
                );
            }

            Some(ReloadSignal::PartialUpdate { skipped_fields }) => {
                config = watcher.config_snapshot();
                println!(
                    "\n  ⚠️  [Frame {frame}] PARTIAL UPDATE — \
                     Skipped fields: {:?}\n  \
                     Other fields have been updated.\n",
                    skipped_fields
                );
            }

            None => {}
        }

        simulate_game_tick(frame, &config);

        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            thread::sleep(frame_duration - elapsed);
        }
    }

    println!("\n  Demo completed ({RUN_DURATION_SECS}s).");
}

fn simulate_game_tick(frame: u64, config: &GameConfig) {
    if frame % 10 == 0 {
        let pos_x = (frame as f32) * config.player.speed * 0.016;
        println!(
            "  [Frame {:>4}] pos_x={:.1}  hp={:.0}  enemies={}  fps={}",
            frame,
            pos_x,
            config.player.health,
            config.world.enemy_count,
            config.renderer.target_fps,
        );
    }

    if config.world.debug_enabled && frame % 30 == 0 {
        println!(
            "  [DEBUG] gravity={:.2}  jump={:.2}  fov={:.1}",
            config.world.gravity, config.player.jump_force, config.renderer.fov,
        );
    }
}

fn print_banner() {
    println!("\n╔══════════════════════════════════════════════════════╗");
    println!("║   snapcfg v2 — #[derive(HotReload)] + Graceful     ║");
    println!("║   Fallback Demo                                      ║");
    println!("╠══════════════════════════════════════════════════════╣");
    println!("║  Valid value:    speed = 22.0                       ║");
    println!("║  Invalid type:   speed = \"twentytwo\" → old value!   ║");
    println!("║  Syntax error:   speed = [         → nothing!       ║");
    println!("╚══════════════════════════════════════════════════════╝\n");
}
//! `Notchi.exe`: titik masuk, argumen CLI, dan wiring (PRD §6.2).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

use notchi_island::{IslandApp, Paths, config_path, reset_config};
use notchi_platform_win::sys::{InstanceLock, attach_parent_console, init_process};
use notchi_platform_win::window::{create_overlay, error_box, notify_existing_instance, run_message_loop};

const INSTANCE: &str = "Notchi.Island.v1";

const HELP: &str = "\
Notchi — Island interaktif bergaya macOS untuk Windows 11

Pemakaian: Notchi.exe [opsi]

Tanpa opsi, Notchi berjalan di latar belakang. Menjalankannya lagi saat
sudah berjalan akan membuka Island.

Opsi:
  --config-path    Tampilkan lokasi config.toml
  --reset-config   Tulis ulang config.toml ke nilai default
  --version        Tampilkan versi
  --help           Tampilkan bantuan ini
";

fn cli(args: &[String]) -> i32 {
    attach_parent_console();
    match args.first().map(String::as_str) {
        Some("--help" | "-h" | "/?") => {
            print!("{HELP}");
            0
        }
        Some("--version" | "-V") => {
            println!("Notchi {}", env!("CARGO_PKG_VERSION"));
            0
        }
        Some("--config-path") => {
            println!("{}", config_path().display());
            0
        }
        Some("--reset-config") => match reset_config(&config_path()) {
            Ok(()) => {
                println!("config.toml ditulis ulang: {}", config_path().display());
                0
            }
            Err(e) => {
                eprintln!("gagal menulis config.toml: {e}");
                1
            }
        },
        Some(other) => {
            eprintln!("opsi tidak dikenal: {other}\n\n{HELP}");
            2
        }
        None => 0,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        std::process::exit(cli(&args));
    }

    let _hardening = init_process();
    let Some(_lock) = InstanceLock::acquire(INSTANCE) else {
        // Sudah berjalan: minta instans pertama membuka Island.
        notify_existing_instance();
        return;
    };

    let mut window = match create_overlay("Notchi") {
        Ok(w) => w,
        Err(e) => {
            error_box("Notchi", &format!("Gagal membuat jendela: {e}"));
            return;
        }
    };
    let exe = std::env::current_exe().unwrap_or_default();
    if let Err(e) = IslandApp::install(&mut window, Paths::default_location(), exe) {
        error_box("Notchi", &format!("Gagal memulai Island: {e}\n\nNotchi memerlukan Windows 11 22H2 atau lebih baru."));
        return;
    }
    run_message_loop();
    drop(window);
}

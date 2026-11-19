// Prevents an additional console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(err) = tyoaikalaskuri_lib::run() {
        eprintln!("Työaikalaskuri failed to start: {err}");
        std::process::exit(1);
    }
}

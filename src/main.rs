#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    if let Err(error) = lightdo::app::run() {
        eprintln!("{error}");
    }
}

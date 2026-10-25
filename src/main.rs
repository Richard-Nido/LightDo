#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    if std::env::args().any(|arg| arg == "--diagnose") {
        println!(
            "{}",
            serde_json::to_string_pretty(&lightdo::diagnostics::inspect()).unwrap()
        );
        return;
    }
    if std::env::args().any(|arg| arg == "--version") {
        println!("LightDo {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if let Err(error) = lightdo::app::run() {
        eprintln!("{error}");
        unsafe {
            windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
                None,
                &windows::core::HSTRING::from(error),
                windows::core::w!("轻待启动失败"),
                windows::Win32::UI::WindowsAndMessaging::MB_OK
                    | windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR,
            );
        }
    }
}

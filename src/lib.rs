#[cfg(windows)]
pub mod app;
pub mod geometry;
#[cfg(windows)]
pub mod material;
#[cfg(windows)]
pub mod menu_ui;
pub mod model;
#[cfg(windows)]
pub mod platform;
#[cfg(windows)]
pub mod settings_ui;
pub mod state;
pub mod storage;
#[cfg(windows)]
pub mod ui;

/// Background acquisition diagnostics never include pixels or task text.
#[cfg(windows)]
pub mod diagnostics {
    pub fn note_sparingly(_kind: &str, _message: impl FnOnce() -> String) {}
    pub fn mark(_message: impl FnOnce() -> String) {}
    pub struct Timed;
    impl Timed {
        pub fn start(_kind: &str, _message: impl FnOnce() -> String) -> Self {
            Self
        }
    }
    /// Explicit, local-only diagnostics. No task text, usernames, paths,
    /// screenshots, or credentials are returned.
    pub fn inspect() -> serde_json::Value {
        unsafe {
            use windows::{
                Win32::{
                    Foundation::*,
                    Graphics::Dwm::*,
                    System::RemoteDesktop::*,
                    UI::{HiDpi::GetDpiForWindow, WindowsAndMessaging::*},
                },
                core::{PWSTR, w},
            };
            let hwnd = FindWindowW(w!("LightDo.Panel"), None).ok();
            let mut panel = serde_json::Value::Null;
            if let Some(hwnd) = hwnd {
                let mut placement = WINDOWPLACEMENT {
                    length: size_of::<WINDOWPLACEMENT>() as u32,
                    ..Default::default()
                };
                let _ = GetWindowPlacement(hwnd, &mut placement);
                let mut r = RECT::default();
                let _ = GetWindowRect(hwnd, &mut r);
                let mut cloaked = 0u32;
                let _ =
                    DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut cloaked as *mut _ as *mut _, 4);
                let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
                let popup = GetLastActivePopup(hwnd);
                panel = serde_json::json!({"visible":IsWindowVisible(hwnd).as_bool(),"minimized":IsIconic(hwnd).as_bool(),"showCommand":placement.showCmd,"dpi":GetDpiForWindow(hwnd),"cloaked":cloaked,"rect":[r.left,r.top,r.right,r.bottom],"foreground":GetForegroundWindow()==hwnd,"topmost":ex & WS_EX_TOPMOST.0 != 0,"toolWindow":ex & WS_EX_TOOLWINDOW.0 != 0,"visibleOwnedPopup":popup != hwnd && IsWindowVisible(popup).as_bool()});
            }
            let mut session = serde_json::Value::Null;
            let mut p = PWSTR::null();
            let mut bytes = 0;
            if WTSQuerySessionInformationW(
                Some(WTS_CURRENT_SERVER_HANDLE),
                WTS_CURRENT_SESSION,
                WTSSessionInfoEx,
                &mut p,
                &mut bytes,
            )
            .is_ok()
            {
                if bytes as usize >= size_of::<WTSINFOEXW>() {
                    let info = &*(p.0 as *const WTSINFOEXW);
                    if info.Level == 1 {
                        let data = info.Data.WTSInfoExLevel1;
                        session = serde_json::json!({"state":data.SessionState.0,"lockFlag":data.SessionFlags,"locked":data.SessionFlags==WTS_SESSIONSTATE_LOCK as i32});
                    }
                }
                WTSFreeMemory(p.0.cast());
            }
            serde_json::json!({"version":env!("CARGO_PKG_VERSION"),"panel":panel,"session":session})
        }
    }
}

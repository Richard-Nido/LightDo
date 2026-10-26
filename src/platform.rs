use windows::{
    Win32::{Foundation::*, Graphics::Gdi::*, System::Registry::*, UI::WindowsAndMessaging::*},
    core::{BOOL, HSTRING, PCWSTR, w},
};

pub fn monitors() -> Vec<RECT> {
    unsafe extern "system" fn each(_: HMONITOR, _: HDC, r: *mut RECT, p: LPARAM) -> BOOL {
        unsafe { (*(p.0 as *mut Vec<RECT>)).push(*r) };
        true.into()
    }
    let mut v = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(None, None, Some(each), LPARAM(&mut v as *mut _ as isize));
    }
    v
}
pub fn desktop() -> RECT {
    let all = monitors();
    all.into_iter()
        .reduce(|a, b| RECT {
            left: a.left.min(b.left),
            top: a.top.min(b.top),
            right: a.right.max(b.right),
            bottom: a.bottom.max(b.bottom),
        })
        .unwrap_or(RECT {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        })
}
pub fn primary() -> MONITORINFO {
    unsafe {
        let monitor = MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY);
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(monitor, &mut info);
        info
    }
}
pub fn clamp_position(x: i32, y: i32, _w: i32, h: i32) -> (i32, i32) {
    let all = monitors();
    if all
        .iter()
        .any(|r| x + 80 > r.left && x < r.right - 40 && y + 60 > r.top && y < r.bottom - 40)
    {
        (x, y)
    } else {
        let r = primary().rcWork;
        (r.left + 24, r.top + 24.min((r.bottom - r.top - h).max(0)))
    }
}
pub fn system_dark() -> bool {
    unsafe {
        let mut value = 1u32;
        let mut size = 4;
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut value as *mut _ as *mut _),
            Some(&mut size),
        )
        .is_ok()
            && value == 0
    }
}
pub fn autostart(enabled: bool) -> Result<(), String> {
    unsafe {
        let mut key = HKEY::default();
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            None,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            None,
            &mut key,
            None,
        )
        .ok()
        .map_err(|e| e.to_string())?;
        let result = if enabled {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let text: Vec<u16> = format!("\"{}\" --startup", exe.display())
                .encode_utf16()
                .chain(Some(0))
                .collect();
            let bytes = std::slice::from_raw_parts(text.as_ptr().cast(), text.len() * 2);
            RegSetValueExW(key, w!("LightDo"), None, REG_SZ, Some(bytes))
                .ok()
                .map_err(|e| e.to_string())
        } else {
            match RegDeleteValueW(key, w!("LightDo")).ok() {
                Ok(()) => Ok(()),
                Err(e) if e.code().0 as u32 == 0x80070002 => Ok(()),
                Err(e) => Err(e.to_string()),
            }
        };
        let _ = RegCloseKey(key);
        result
    }
}
pub fn autostart_enabled() -> bool {
    unsafe {
        let mut size = 0;
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            w!("LightDo"),
            RRF_RT_REG_SZ,
            None,
            None,
            Some(&mut size),
        )
        .is_ok()
    }
}

/// Desktop shell lookup and managed Z order, adapted from Glance
/// ui/window.rs (MIT, lulu-loopp). The panel remains top-level so the
/// desktop can be sampled without capturing the panel as a child of it.
pub fn desktop_host() -> Option<HWND> {
    unsafe {
        let progman = FindWindowW(w!("Progman"), PCWSTR::null()).ok()?;
        if FindWindowExW(Some(progman), None, w!("SHELLDLL_DefView"), PCWSTR::null()).is_ok() {
            return Some(progman);
        }
        unsafe extern "system" fn each(window: HWND, p: LPARAM) -> BOOL {
            unsafe {
                if FindWindowExW(Some(window), None, w!("SHELLDLL_DefView"), PCWSTR::null()).is_ok()
                {
                    *(p.0 as *mut HWND) = window;
                    return false.into();
                }
            }
            true.into()
        }
        let mut found = HWND::default();
        let _ = EnumWindows(Some(each), LPARAM(&mut found as *mut _ as isize));
        if !found.is_invalid() {
            Some(found)
        } else {
            Some(progman)
        }
    }
}
pub fn set_desktop(hwnd: HWND, host: Option<HWND>) -> Result<(), String> {
    unsafe {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        let ex = if host.is_some() {
            (ex & !WS_EX_APPWINDOW.0) | WS_EX_TOOLWINDOW.0
        } else {
            (ex & !WS_EX_TOOLWINDOW.0) | WS_EX_APPWINDOW.0
        };
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex as isize);
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
        if let Some(host) = host {
            sink_to_desktop(hwnd, host);
        }
        Ok(())
    }
}
pub fn sink_to_desktop(hwnd: HWND, host: HWND) {
    unsafe {
        let flags = SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE;
        if GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOPMOST.0 != 0 {
            let _ = SetWindowPos(hwnd, Some(HWND_NOTOPMOST), 0, 0, 0, 0, flags);
        }
        let after = match GetWindow(host, GW_HWNDPREV) {
            Ok(above) if above == hwnd => return,
            Ok(above) if GetWindowLongPtrW(above, GWL_EXSTYLE) as u32 & WS_EX_TOPMOST.0 == 0 => {
                above
            }
            _ => HWND_TOP,
        };
        if !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        }
        let _ = SetWindowPos(hwnd, Some(after), 0, 0, 0, 0, flags);
    }
}
pub fn fullscreen_other(hwnd: HWND) -> bool {
    unsafe {
        let front = GetForegroundWindow();
        if front == hwnd || front.is_invalid() {
            return false;
        }
        // A regular maximized window can cover the monitor when the taskbar
        // auto-hides. Its caption/maximized state distinguishes it from a
        // borderless fullscreen surface.
        let style = GetWindowLongPtrW(front, GWL_STYLE) as u32;
        if IsZoomed(front).as_bool() && style & WS_CAPTION.0 == WS_CAPTION.0 {
            return false;
        }
        let mut r = RECT::default();
        if GetWindowRect(front, &mut r).is_err() {
            return false;
        }
        let screen = primary().rcMonitor;
        r.left <= screen.left
            && r.top <= screen.top
            && r.right >= screen.right
            && r.bottom >= screen.bottom
            && front != GetShellWindow()
    }
}
pub fn wide(text: &str) -> HSTRING {
    HSTRING::from(text)
}

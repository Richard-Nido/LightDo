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

/// Explorer desktop embedding is isolated here; callers must recover to a
/// normal top-level window if the host vanishes or parenting fails.
pub fn desktop_host() -> Option<HWND> {
    unsafe {
        let progman = FindWindowW(w!("Progman"), PCWSTR::null()).ok()?;
        let mut result = 0;
        let _ = SendMessageTimeoutW(
            progman,
            0x052c,
            WPARAM(0xD),
            LPARAM(0),
            SMTO_ABORTIFHUNG,
            500,
            Some(&mut result),
        );
        let _ = SendMessageTimeoutW(
            progman,
            0x052c,
            WPARAM(0xD),
            LPARAM(1),
            SMTO_ABORTIFHUNG,
            500,
            Some(&mut result),
        );
        // Windows 11 24H2 may place a WorkerW inside Progman. A sibling behind
        // the icon host is preferred on older desktop arrangements.
        if let Ok(child) = FindWindowExW(Some(progman), None, w!("WorkerW"), PCWSTR::null()) {
            return Some(child);
        }
        unsafe extern "system" fn each(window: HWND, p: LPARAM) -> BOOL {
            unsafe {
                if FindWindowExW(Some(window), None, w!("SHELLDLL_DefView"), PCWSTR::null()).is_ok()
                {
                    if let Ok(worker) =
                        FindWindowExW(None, Some(window), w!("WorkerW"), PCWSTR::null())
                    {
                        *(p.0 as *mut HWND) = worker;
                        return false.into();
                    }
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
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        let style = if host.is_some() {
            (style & !WS_POPUP.0) | WS_CHILD.0
        } else {
            (style & !WS_CHILD.0) | WS_POPUP.0
        };
        SetWindowLongPtrW(hwnd, GWL_STYLE, style as isize);
        SetParent(hwnd, host).map_err(|e| format!("桌面嵌入失败：{e}"))?;
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
        Ok(())
    }
}
pub fn fullscreen_other(hwnd: HWND) -> bool {
    unsafe {
        let front = GetForegroundWindow();
        if front == hwnd || front.is_invalid() {
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

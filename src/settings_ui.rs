//! Native modeless settings dialog. It edits settings only, never a second
//! todo list. System controls provide keyboard and accessibility support.
use crate::model::{Mode, Settings, Side, Theme};
use std::cell::RefCell;
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{Controls::*, HiDpi::GetDpiForWindow, WindowsAndMessaging::*},
    },
    core::{HSTRING, Result, w},
};
pub const CHANGED: u32 = WM_APP + 12;
const THEME: i32 = 301;
const OPACITY: i32 = 302;
const MODE: i32 = 303;
const SIDE: i32 = 304;
const AUTO: i32 = 305;
const HIDE: i32 = 306;
const DWELL: i32 = 307;
const PIN: i32 = 308;
const LOCK: i32 = 309;
const TOP: i32 = 310;
const TITLE: i32 = 311;
const FULL: i32 = 312;
const KEY: i32 = 313;
const START: i32 = 314;
thread_local! {static UI:RefCell<Option<Ui>>=const{RefCell::new(None)};static PENDING:RefCell<Option<Settings>>=const{RefCell::new(None)};}
struct Ui {
    hwnd: HWND,
    owner: HWND,
    font: HFONT,
    settings: Settings,
}
pub fn take() -> Option<Settings> {
    PENDING.with(|p| p.borrow_mut().take())
}
pub fn is_open() -> bool {
    UI.with(|u| u.borrow().is_some())
}
pub fn close() {
    let old = UI.with(|u| u.borrow_mut().take());
    if let Some(ui) = old {
        unsafe {
            let _ = DestroyWindow(ui.hwnd);
            let _ = DeleteObject(ui.font.into());
        }
    }
}
pub fn dialog_message(msg: &MSG) -> bool {
    UI.with(|ui| {
        ui.borrow()
            .as_ref()
            .is_some_and(|ui| unsafe { IsDialogMessageW(ui.hwnd, msg).as_bool() })
    })
}
unsafe fn control(hwnd: HWND, id: i32) -> HWND {
    unsafe { GetDlgItem(Some(hwnd), id).unwrap_or_default() }
}
unsafe fn send(hwnd: HWND, id: i32, msg: u32, w: usize, l: isize) -> LRESULT {
    unsafe { SendMessageW(control(hwnd, id), msg, Some(WPARAM(w)), Some(LPARAM(l))) }
}
unsafe fn checked(hwnd: HWND, id: i32) -> bool {
    unsafe { send(hwnd, id, BM_GETCHECK, 0, 0).0 == BST_CHECKED.0 as isize }
}
unsafe fn set_check(hwnd: HWND, id: i32, on: bool) {
    unsafe {
        send(
            hwnd,
            id,
            BM_SETCHECK,
            if on {
                BST_CHECKED.0 as usize
            } else {
                BST_UNCHECKED.0 as usize
            },
            0,
        );
    }
}
unsafe fn set_number(hwnd: HWND, id: i32, n: u64) {
    unsafe {
        let _ = SetWindowTextW(control(hwnd, id), &HSTRING::from(n.to_string()));
    }
}
unsafe fn number(hwnd: HWND, id: i32) -> Option<u64> {
    unsafe {
        let mut buf = [0u16; 32];
        let n = GetWindowTextW(control(hwnd, id), &mut buf);
        String::from_utf16_lossy(&buf[..n as usize]).parse().ok()
    }
}
unsafe fn populate(ui: &Ui) {
    unsafe {
        let c = &ui.settings;
        send(
            ui.hwnd,
            THEME,
            CB_SETCURSEL,
            match c.theme {
                Theme::System => 0,
                Theme::Light => 1,
                Theme::Dark => 2,
            },
            0,
        );
        send(
            ui.hwnd,
            MODE,
            CB_SETCURSEL,
            match c.mode {
                Mode::Edge => 0,
                Mode::Floating => 1,
                Mode::Desktop => 2,
            },
            0,
        );
        send(
            ui.hwnd,
            SIDE,
            CB_SETCURSEL,
            if c.side == Side::Right { 0 } else { 1 },
            0,
        );
        send(ui.hwnd, OPACITY, TBM_SETPOS, 1, c.opacity as isize);
        set_number(ui.hwnd, HIDE, c.hide_ms);
        set_number(ui.hwnd, DWELL, c.dwell_ms);
        for (id, on) in [
            (AUTO, c.auto_hide),
            (PIN, c.pinned),
            (LOCK, c.locked),
            (TOP, c.topmost),
            (TITLE, c.titlebar),
            (FULL, c.fullscreen_pause),
            (START, c.autostart),
        ] {
            set_check(ui.hwnd, id, on)
        }
        let flags = (if c.hotkey_mod & 2 != 0 {
            HOTKEYF_CONTROL
        } else {
            0
        }) | (if c.hotkey_mod & 1 != 0 {
            HOTKEYF_ALT
        } else {
            0
        }) | (if c.hotkey_mod & 4 != 0 {
            HOTKEYF_SHIFT
        } else {
            0
        });
        send(
            ui.hwnd,
            KEY,
            HKM_SETHOTKEY,
            c.hotkey_vk as usize | ((flags as usize) << 8),
            0,
        );
    }
}
unsafe fn read(ui: &Ui) -> std::result::Result<Settings, String> {
    unsafe {
        let mut c = ui.settings.clone();
        c.theme = match send(ui.hwnd, THEME, CB_GETCURSEL, 0, 0).0 {
            1 => Theme::Light,
            2 => Theme::Dark,
            _ => Theme::System,
        };
        c.mode = match send(ui.hwnd, MODE, CB_GETCURSEL, 0, 0).0 {
            1 => Mode::Floating,
            2 => Mode::Desktop,
            _ => Mode::Edge,
        };
        c.side = if send(ui.hwnd, SIDE, CB_GETCURSEL, 0, 0).0 == 1 {
            Side::Left
        } else {
            Side::Right
        };
        c.opacity = send(ui.hwnd, OPACITY, WM_USER, 0, 0).0 as u32;
        c.hide_ms = number(ui.hwnd, HIDE).ok_or("请输入正确的收回延迟")?;
        c.dwell_ms = number(ui.hwnd, DWELL).ok_or("请输入正确的边缘停留时间")?;
        if !(100..=5000).contains(&c.hide_ms) || !(80..=3000).contains(&c.dwell_ms) {
            return Err("收回延迟范围 100–5000 ms；边缘停留范围 80–3000 ms。".into());
        }
        c.auto_hide = checked(ui.hwnd, AUTO);
        c.pinned = checked(ui.hwnd, PIN);
        c.locked = checked(ui.hwnd, LOCK);
        c.topmost = checked(ui.hwnd, TOP);
        c.titlebar = checked(ui.hwnd, TITLE);
        c.fullscreen_pause = checked(ui.hwnd, FULL);
        c.autostart = checked(ui.hwnd, START);
        let key = send(ui.hwnd, KEY, HKM_GETHOTKEY, 0, 0).0 as u32;
        c.hotkey_vk = key & 255;
        let flags = key >> 8;
        c.hotkey_mod = (if flags & HOTKEYF_ALT as u32 != 0 {
            1
        } else {
            0
        }) | (if flags & HOTKEYF_CONTROL as u32 != 0 {
            2
        } else {
            0
        }) | (if flags & HOTKEYF_SHIFT as u32 != 0 {
            4
        } else {
            0
        });
        if c.hotkey_vk == 0 || c.hotkey_mod == 0 {
            return Err("快捷键至少包含 Ctrl、Alt、Shift 之一和一个按键。".into());
        }
        Ok(c)
    }
}
unsafe extern "system" fn procedure(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_CLOSE => {
            close();
            LRESULT(0)
        }
        WM_COMMAND => {
            match w.0 & 65535 {
                1 => {
                    let result = UI.with(|u| u.borrow().as_ref().map(|ui| unsafe { read(ui) }));
                    match result {
                        Some(Ok(settings)) => {
                            UI.with(|u| {
                                if let Some(ui) = u.borrow_mut().as_mut() {
                                    ui.settings = settings.clone();
                                    PENDING.with(|p| *p.borrow_mut() = Some(settings));
                                    unsafe {
                                        let _ = PostMessageW(
                                            Some(ui.owner),
                                            CHANGED,
                                            WPARAM(0),
                                            LPARAM(0),
                                        );
                                    }
                                }
                            });
                        }
                        Some(Err(e)) => unsafe {
                            MessageBoxW(
                                Some(hwnd),
                                &HSTRING::from(e),
                                w!("设置未应用"),
                                MB_OK | MB_ICONWARNING,
                            );
                        },
                        _ => {}
                    }
                }
                2 => close(),
                3 => {
                    UI.with(|u| {
                        if let Some(ui) = u.borrow_mut().as_mut() {
                            let c = Settings::default();
                            ui.settings = Settings {
                                introduced: true,
                                import_decided: ui.settings.import_decided,
                                ..c
                            };
                            unsafe { populate(ui) }
                        }
                    });
                }
                _ => {}
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, w, l) },
    }
}
pub fn open(owner: HWND, settings: &Settings, diagnostic: &str) -> Result<()> {
    unsafe {
        if let Some(hwnd) = UI.with(|u| u.borrow().as_ref().map(|ui| ui.hwnd)) {
            let _ = SetForegroundWindow(hwnd);
            return Ok(());
        }
        InitCommonControls();
        let instance = GetModuleHandleW(None)?;
        RegisterClassW(&WNDCLASSW {
            lpfnWndProc: Some(procedure),
            hInstance: instance.into(),
            lpszClassName: w!("LightDo.Settings"),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as *mut _),
            ..Default::default()
        });
        let s = GetDpiForWindow(owner) as f32 / 96.0;
        let hwnd = CreateWindowExW(
            WS_EX_CONTROLPARENT,
            w!("LightDo.Settings"),
            w!("轻待 · 设置"),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
            (160.0 * s) as i32,
            (70.0 * s) as i32,
            (500.0 * s) as i32,
            (660.0 * s) as i32,
            Some(owner),
            None,
            Some(instance.into()),
            None,
        )?;
        let font = CreateFontW(
            -(14.0 * s) as i32,
            0,
            0,
            0,
            400,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            DEFAULT_PITCH.0 as u32,
            w!("Microsoft YaHei UI"),
        );
        let make = |class: &str,
                    label: &str,
                    id: i32,
                    x: i32,
                    y: i32,
                    width: i32,
                    height: i32,
                    extra: u32|
         -> Result<HWND> {
            let h = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                &HSTRING::from(class),
                &HSTRING::from(label),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(extra),
                (x as f32 * s) as i32,
                (y as f32 * s) as i32,
                (width as f32 * s) as i32,
                (height as f32 * s) as i32,
                Some(hwnd),
                Some(HMENU(id as usize as *mut _)),
                None,
                None,
            )?;
            SendMessageW(
                h,
                WM_SETFONT,
                Some(WPARAM(font.0 as usize)),
                Some(LPARAM(1)),
            );
            Ok(h)
        };
        let label = |t: &str, x, y, width| make("STATIC", t, 0, x, y, width, 22, 0);
        label("外观与窗口", 24, 20, 400)?;
        label("主题", 24, 58, 90)?;
        let theme = make(
            "COMBOBOX",
            "",
            THEME,
            140,
            54,
            290,
            160,
            CBS_DROPDOWNLIST as u32 | WS_TABSTOP.0,
        )?;
        for t in ["跟随系统", "浅色玻璃", "深色玻璃"] {
            SendMessageW(
                theme,
                CB_ADDSTRING,
                None,
                Some(LPARAM(HSTRING::from(t).as_ptr() as isize)),
            );
        }
        label("玻璃材质浓度", 24, 100, 110)?;
        let slider = make(
            "msctls_trackbar32",
            "",
            OPACITY,
            133,
            94,
            300,
            32,
            TBS_AUTOTICKS | WS_TABSTOP.0,
        )?;
        SendMessageW(
            slider,
            TBM_SETRANGE,
            Some(WPARAM(1)),
            Some(LPARAM(10 | (90 << 16))),
        );
        label("窗口形态", 24, 142, 90)?;
        let mode = make(
            "COMBOBOX",
            "",
            MODE,
            140,
            138,
            290,
            160,
            CBS_DROPDOWNLIST as u32 | WS_TABSTOP.0,
        )?;
        for t in ["边缘面板", "独立悬浮窗", "桌面组件"] {
            SendMessageW(
                mode,
                CB_ADDSTRING,
                None,
                Some(LPARAM(HSTRING::from(t).as_ptr() as isize)),
            );
        }
        label("主屏停靠侧", 24, 184, 100)?;
        let side = make(
            "COMBOBOX",
            "",
            SIDE,
            140,
            180,
            290,
            120,
            CBS_DROPDOWNLIST as u32 | WS_TABSTOP.0,
        )?;
        for t in ["右侧", "左侧"] {
            SendMessageW(
                side,
                CB_ADDSTRING,
                None,
                Some(LPARAM(HSTRING::from(t).as_ptr() as isize)),
            );
        }
        make(
            "BUTTON",
            "自动收回边缘面板",
            AUTO,
            24,
            224,
            220,
            25,
            BS_AUTOCHECKBOX as u32 | WS_TABSTOP.0,
        )?;
        label("收回延迟（ms）", 24, 263, 130)?;
        make(
            "EDIT",
            "",
            HIDE,
            160,
            258,
            72,
            26,
            WS_BORDER.0 | WS_TABSTOP.0 | ES_NUMBER as u32,
        )?;
        label("边缘停留（ms）", 248, 263, 130)?;
        make(
            "EDIT",
            "",
            DWELL,
            378,
            258,
            72,
            26,
            WS_BORDER.0 | WS_TABSTOP.0 | ES_NUMBER as u32,
        )?;
        for (id, t, x, y) in [
            (PIN, "固定展开（边缘）", 24, 307),
            (TOP, "始终置顶（悬浮）", 248, 307),
            (LOCK, "锁定位置与尺寸", 24, 342),
            (TITLE, "显示标题栏", 248, 342),
            (FULL, "全屏时暂停边缘呼出", 24, 377),
            (START, "登录 Windows 时启动", 248, 377),
        ] {
            make(
                "BUTTON",
                t,
                id,
                x,
                y,
                224,
                24,
                BS_AUTOCHECKBOX as u32 | WS_TABSTOP.0,
            )?;
        }
        label("呼出快捷键", 24, 425, 110)?;
        make(
            "msctls_hotkey32",
            "",
            KEY,
            140,
            420,
            290,
            28,
            WS_TABSTOP.0 | WS_BORDER.0,
        )?;
        label("诊断", 24, 472, 90)?;
        make("STATIC", diagnostic, 0, 24, 500, 430, 52, 0)?;
        make(
            "BUTTON",
            "恢复默认设置",
            3,
            24,
            568,
            130,
            32,
            BS_PUSHBUTTON as u32 | WS_TABSTOP.0,
        )?;
        make(
            "BUTTON",
            "应用",
            1,
            264,
            568,
            90,
            32,
            BS_DEFPUSHBUTTON as u32 | WS_TABSTOP.0,
        )?;
        make(
            "BUTTON",
            "关闭",
            2,
            364,
            568,
            90,
            32,
            BS_PUSHBUTTON as u32 | WS_TABSTOP.0,
        )?;
        let ui = Ui {
            hwnd,
            owner,
            font,
            settings: settings.clone(),
        };
        populate(&ui);
        UI.with(|u| *u.borrow_mut() = Some(ui));
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
        Ok(())
    }
}

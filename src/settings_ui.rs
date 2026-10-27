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
const PAGES: i32 = 315;
thread_local! {
    static UI:RefCell<Option<Ui>>=const{RefCell::new(None)};
    static PENDING:RefCell<Option<Settings>>=const{RefCell::new(None)};
    static LAST_FOCUS:std::cell::Cell<HWND>=const{std::cell::Cell::new(HWND(std::ptr::null_mut()))};
}
struct Ui {
    hwnd: HWND,
    owner: HWND,
    font: HFONT,
    settings: Settings,
    scroll: i32,
    pages: Vec<(usize, HWND)>,
    data_directory: std::path::PathBuf,
}
pub fn take() -> Option<Settings> {
    PENDING.with(|p| p.borrow_mut().take())
}
pub fn is_open() -> bool {
    UI.with(|u| u.borrow().is_some())
}
pub fn sync_layer() {
    let windows = UI.with(|u| u.borrow().as_ref().map(|ui| (ui.hwnd, ui.owner)));
    if let Some((hwnd, panel)) = windows {
        unsafe {
            // Settings remains independent of the desktop host, but must be
            // above a topmost edge/floating panel when their bounds overlap.
            let topmost = GetWindowLongPtrW(panel, GWL_EXSTYLE) as u32 & WS_EX_TOPMOST.0 != 0;
            let _ = SetWindowPos(
                hwnd,
                Some(if topmost {
                    HWND_TOPMOST
                } else {
                    HWND_NOTOPMOST
                }),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }
}
pub fn close() {
    let old = UI.with(|u| u.borrow_mut().take());
    if let Some(ui) = old {
        LAST_FOCUS.with(|f| f.set(HWND::default()));
        unsafe {
            let _ = DestroyWindow(ui.hwnd);
            let _ = DeleteObject(ui.font.into());
        }
    }
}
pub fn dialog_message(msg: &MSG) -> bool {
    let hwnd = UI.with(|ui| ui.borrow().as_ref().map(|ui| ui.hwnd));
    hwnd.is_some_and(|hwnd| unsafe { IsDialogMessageW(hwnd, msg).as_bool() })
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
        sync_mode_controls(ui.hwnd);
    }
}
unsafe fn sync_mode_controls(hwnd: HWND) {
    unsafe {
        let mode = send(hwnd, MODE, CB_GETCURSEL, 0, 0).0;
        for (id, enabled) in [
            (PIN, mode == 0),
            (TOP, mode == 1),
            (SIDE, mode == 0),
            (AUTO, mode == 0),
            (HIDE, mode == 0),
            (DWELL, mode == 0),
            (FULL, mode == 0),
        ] {
            let _ = windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow(
                control(hwnd, id),
                enabled,
            );
        }
    }
}
unsafe fn show_page(ui: &Ui, page: usize) {
    unsafe {
        for (index, hwnd) in &ui.pages {
            let _ = ShowWindow(*hwnd, if *index == page { SW_SHOWNA } else { SW_HIDE });
        }
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
        // Disabled controls retain their mode-specific preference. The window
        // layer applies it only in the corresponding mode.
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
        WM_ACTIVATE => unsafe {
            use windows::Win32::UI::Input::KeyboardAndMouse::{
                GetFocus, IsWindowEnabled, SetFocus,
            };
            if w.0 & 0xffff == WA_INACTIVE as usize {
                let focus = GetFocus();
                if IsChild(hwnd, focus).as_bool() {
                    LAST_FOCUS.with(|f| f.set(focus));
                }
            }
            let result = DefWindowProcW(hwnd, msg, w, l);
            if w.0 & 0xffff != WA_INACTIVE as usize {
                let focus = LAST_FOCUS.with(|f| f.get());
                if IsWindow(Some(focus)).as_bool()
                    && IsWindowVisible(focus).as_bool()
                    && IsWindowEnabled(focus).as_bool()
                {
                    let _ = SetFocus(Some(focus));
                }
            }
            result
        },
        WM_CTLCOLORSTATIC | WM_CTLCOLORBTN => unsafe {
            let dc = HDC(w.0 as *mut _);
            SetTextColor(dc, COLORREF(GetSysColor(COLOR_WINDOWTEXT)));
            SetBkColor(dc, COLORREF(GetSysColor(COLOR_WINDOW)));
            SetBkMode(dc, TRANSPARENT);
            LRESULT(GetSysColorBrush(COLOR_WINDOW).0 as isize)
        },
        WM_NOTIFY => {
            let header = unsafe { &*(l.0 as *const NMHDR) };
            if header.idFrom == PAGES as usize && header.code == TCN_SELCHANGE {
                let page = unsafe { send(hwnd, PAGES, TCM_GETCURSEL, 0, 0).0 } as usize;
                UI.with(|u| {
                    if let Some(ui) = u.borrow().as_ref() {
                        unsafe { show_page(ui, page) };
                    }
                });
            }
            LRESULT(0)
        }
        WM_SIZE => {
            unsafe {
                let mut r = RECT::default();
                let _ = GetClientRect(hwnd, &mut r);
                let scale = GetDpiForWindow(hwnd) as f32 / 96.0;
                let info = SCROLLINFO {
                    cbSize: size_of::<SCROLLINFO>() as u32,
                    fMask: SIF_RANGE | SIF_PAGE,
                    nMin: 0,
                    nMax: (640.0 * scale) as i32,
                    nPage: (r.bottom - r.top).max(1) as u32,
                    ..Default::default()
                };
                SetScrollInfo(hwnd, SB_VERT, &info, true);
            }
            LRESULT(0)
        }
        WM_VSCROLL | WM_MOUSEWHEEL => {
            unsafe {
                let mut info = SCROLLINFO {
                    cbSize: size_of::<SCROLLINFO>() as u32,
                    fMask: SIF_ALL,
                    ..Default::default()
                };
                let _ = GetScrollInfo(hwnd, SB_VERT, &mut info);
                let step = (28.0 * GetDpiForWindow(hwnd) as f32 / 96.0) as i32;
                let delta = if msg == WM_MOUSEWHEEL {
                    -((w.0 >> 16) as i16) as i32 / 120 * step
                } else {
                    match w.0 & 65535 {
                        0 => -step,
                        1 => step,
                        2 => -(info.nPage as i32),
                        3 => info.nPage as i32,
                        4 | 5 => info.nTrackPos - info.nPos,
                        _ => 0,
                    }
                };
                let next = (info.nPos + delta).clamp(0, (info.nMax - info.nPage as i32 + 1).max(0));
                let amount = info.nPos - next;
                info.fMask = SIF_POS;
                info.nPos = next;
                SetScrollInfo(hwnd, SB_VERT, &info, true);
                let _ = ScrollWindowEx(
                    hwnd,
                    0,
                    amount,
                    None,
                    None,
                    None,
                    None,
                    SW_SCROLLCHILDREN | SW_INVALIDATE | SW_ERASE,
                );
                UI.with(|u| {
                    if let Some(ui) = u.borrow_mut().as_mut() {
                        ui.scroll = next;
                    }
                });
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            close();
            LRESULT(0)
        }
        WM_COMMAND => {
            if w.0 & 65535 == MODE as usize && (w.0 >> 16) as u32 == CBN_SELCHANGE {
                unsafe { sync_mode_controls(hwnd) };
                return LRESULT(0);
            }
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
                4 => {
                    let directory =
                        UI.with(|u| u.borrow().as_ref().map(|ui| ui.data_directory.clone()));
                    if let Some(directory) = directory {
                        let result = std::fs::create_dir_all(&directory)
                            .map_err(|e| e.to_string())
                            .and_then(|()| unsafe {
                                let opened = windows::Win32::UI::Shell::ShellExecuteW(
                                    Some(hwnd),
                                    w!("open"),
                                    &HSTRING::from(directory.as_os_str()),
                                    None,
                                    None,
                                    SW_SHOWNORMAL,
                                );
                                if opened.0 as isize > 32 {
                                    Ok(())
                                } else {
                                    Err("无法打开本地数据文件夹".into())
                                }
                            });
                        if let Err(error) = result {
                            unsafe {
                                MessageBoxW(
                                    Some(hwnd),
                                    &HSTRING::from(error),
                                    w!("轻待"),
                                    MB_OK | MB_ICONWARNING,
                                );
                            }
                        }
                    }
                }
                3 => {
                    UI.with(|u| {
                        if let Some(ui) = u.borrow_mut().as_mut() {
                            let c = Settings::default();
                            ui.settings = Settings {
                                introduced: true,
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
pub fn open(
    owner: HWND,
    settings: &Settings,
    diagnostic: &str,
    data_directory: &std::path::Path,
) -> Result<()> {
    unsafe {
        if let Some(hwnd) = UI.with(|u| u.borrow().as_ref().map(|ui| ui.hwnd)) {
            sync_layer();
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
        let work = crate::platform::primary().rcWork;
        let hwnd = CreateWindowExW(
            WS_EX_CONTROLPARENT | WS_EX_APPWINDOW,
            w!("LightDo.Settings"),
            w!("轻待 · 设置"),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_VSCROLL,
            work.left + (24.0 * s) as i32,
            work.top + (24.0 * s) as i32,
            (500.0 * s) as i32,
            ((660.0 * s) as i32).min(work.bottom - work.top - 48),
            // A desktop panel is intentionally below ordinary applications.
            // Keep settings independent so sinking that panel cannot also
            // sink this dialog or pull the panel forward when it activates.
            None,
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
        let page = std::cell::Cell::new(usize::MAX);
        let pages = RefCell::new(Vec::new());
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
            if page.get() != usize::MAX {
                pages.borrow_mut().push((page.get(), h));
            }
            Ok(h)
        };
        let label = |t: &str, x, y, width| make("STATIC", t, 0, x, y, width, 22, 0);
        let tabs = make(
            "SysTabControl32",
            "",
            PAGES,
            24,
            20,
            430,
            38,
            WS_TABSTOP.0 | TCS_FIXEDWIDTH,
        )?;
        SendMessageW(
            tabs,
            TCM_SETITEMSIZE,
            None,
            Some(LPARAM((143.0 * s) as isize | (((32.0 * s) as isize) << 16))),
        );
        for (index, title) in ["外观", "窗口", "通用"].iter().enumerate() {
            let text = HSTRING::from(*title);
            let item = TCITEMW {
                mask: TCIF_TEXT,
                pszText: windows::core::PWSTR(text.as_ptr() as *mut u16),
                ..Default::default()
            };
            SendMessageW(
                tabs,
                TCM_INSERTITEMW,
                Some(WPARAM(index)),
                Some(LPARAM(&item as *const _ as isize)),
            );
        }
        page.set(0);
        label("让待办融入你的桌面", 24, 82, 410)?;
        label("主题", 24, 132, 90)?;
        let theme = make(
            "COMBOBOX",
            "",
            THEME,
            140,
            128,
            290,
            160,
            CBS_DROPDOWNLIST as u32 | WS_TABSTOP.0,
        )?;
        for text in ["跟随系统", "浅色玻璃", "深色玻璃"] {
            SendMessageW(
                theme,
                CB_ADDSTRING,
                None,
                Some(LPARAM(HSTRING::from(text).as_ptr() as isize)),
            );
        }
        label("玻璃材质浓度", 24, 188, 110)?;
        let slider = make(
            "msctls_trackbar32",
            "",
            OPACITY,
            133,
            180,
            300,
            36,
            WS_TABSTOP.0,
        )?;
        SendMessageW(
            slider,
            TBM_SETRANGE,
            Some(WPARAM(1)),
            Some(LPARAM(10 | (90 << 16))),
        );
        label("更通透", 140, 223, 90)?;
        label("更柔和", 363, 223, 90)?;
        make(
            "BUTTON",
            "显示标题栏",
            TITLE,
            24,
            280,
            300,
            28,
            BS_AUTOCHECKBOX as u32 | WS_TABSTOP.0,
        )?;
        make(
            "STATIC",
            "浓度仅改变玻璃材质，文字与图标保持清晰。",
            0,
            24,
            342,
            420,
            52,
            0,
        )?;

        page.set(1);
        label("窗口模式", 24, 86, 110)?;
        let mode = make(
            "COMBOBOX",
            "",
            MODE,
            140,
            82,
            290,
            160,
            CBS_DROPDOWNLIST as u32 | WS_TABSTOP.0,
        )?;
        for text in ["边缘面板", "独立悬浮窗", "桌面组件"] {
            SendMessageW(
                mode,
                CB_ADDSTRING,
                None,
                Some(LPARAM(HSTRING::from(text).as_ptr() as isize)),
            );
        }
        label("主屏停靠侧", 24, 134, 110)?;
        let side = make(
            "COMBOBOX",
            "",
            SIDE,
            140,
            130,
            290,
            120,
            CBS_DROPDOWNLIST as u32 | WS_TABSTOP.0,
        )?;
        for text in ["右侧", "左侧"] {
            SendMessageW(
                side,
                CB_ADDSTRING,
                None,
                Some(LPARAM(HSTRING::from(text).as_ptr() as isize)),
            );
        }
        for (id, text, x, y) in [
            (PIN, "固定展开", 24, 192),
            (TOP, "始终置顶", 248, 192),
            (LOCK, "锁定位置与尺寸", 24, 234),
            (AUTO, "自动收回边缘面板", 24, 306),
            (FULL, "全屏时暂停边缘呼出", 24, 438),
        ] {
            make(
                "BUTTON",
                text,
                id,
                x,
                y,
                224,
                28,
                BS_AUTOCHECKBOX as u32 | WS_TABSTOP.0,
            )?;
        }
        label("收回延迟（ms）", 24, 354, 130)?;
        make(
            "EDIT",
            "",
            HIDE,
            160,
            350,
            72,
            28,
            WS_BORDER.0 | WS_TABSTOP.0 | ES_NUMBER as u32,
        )?;
        label("边缘停留（ms）", 248, 354, 130)?;
        make(
            "EDIT",
            "",
            DWELL,
            378,
            350,
            72,
            28,
            WS_BORDER.0 | WS_TABSTOP.0 | ES_NUMBER as u32,
        )?;
        make(
            "STATIC",
            "停留越久，越不易误触。不适用于当前模式的选项会自动禁用。",
            0,
            24,
            392,
            420,
            38,
            0,
        )?;

        page.set(2);
        label("呼出快捷键", 24, 100, 110)?;
        make(
            "msctls_hotkey32",
            "",
            KEY,
            140,
            94,
            290,
            30,
            WS_TABSTOP.0 | WS_BORDER.0,
        )?;
        make(
            "STATIC",
            "呼出面板并进入输入；快捷键冲突时会提示。",
            0,
            24,
            146,
            420,
            44,
            0,
        )?;
        make(
            "BUTTON",
            "登录 Windows 时启动轻待",
            START,
            24,
            216,
            400,
            28,
            BS_AUTOCHECKBOX as u32 | WS_TABSTOP.0,
        )?;
        label("运行状态", 24, 314, 400)?;
        make("STATIC", diagnostic, 0, 24, 354, 420, 88, 0)?;
        make(
            "STATIC",
            "任务自动保存在本机，退出、重启和升级后继续保留。",
            0,
            24,
            466,
            420,
            28,
            0,
        )?;
        make(
            "BUTTON",
            "打开 LightDo 数据文件夹",
            4,
            24,
            500,
            264,
            30,
            BS_PUSHBUTTON as u32 | WS_TABSTOP.0,
        )?;

        page.set(usize::MAX);
        make("STATIC", "", 0, 24, 540, 430, 1, WS_BORDER.0)?;
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
            scroll: 0,
            pages: pages.into_inner(),
            data_directory: data_directory.to_path_buf(),
        };
        populate(&ui);
        show_page(&ui, 0);
        UI.with(|u| *u.borrow_mut() = Some(ui));
        sync_layer();
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
        Ok(())
    }
}

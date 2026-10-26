//! Compact native popup menus. Windows owns navigation, dismissal and focus;
//! owner drawing only supplies spacing, typography and semantic state colors.
use crate::model::{Mode, Settings, Side, Theme};
use std::cell::Cell;
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        UI::{Controls::*, WindowsAndMessaging::*},
    },
    core::{HSTRING, w},
};

thread_local! { static ACTIVE: Cell<*const Popup> = const { Cell::new(std::ptr::null()) }; }
pub fn is_open() -> bool {
    ACTIVE.with(|active| !active.get().is_null())
}
struct Row {
    label: String,
    hint: String,
    icon: char,
    checked: bool,
    heading: bool,
    separator: bool,
    danger: bool,
}
struct Popup {
    root: HMENU,
    rows: Vec<Box<Row>>,
    font: HFONT,
    icons: HFONT,
    brush: HBRUSH,
    scale: f32,
    dark: bool,
    contrast: bool,
}
impl Popup {
    fn px(&self, value: i32) -> i32 {
        (value as f32 * self.scale).round() as i32
    }
    fn color(&self, light: u32, dark: u32, system: SYS_COLOR_INDEX) -> COLORREF {
        if self.contrast {
            unsafe { COLORREF(GetSysColor(system)) }
        } else {
            let rgb = if self.dark { dark } else { light };
            COLORREF(((rgb & 255) << 16) | (rgb & 0xff00) | ((rgb >> 16) & 255))
        }
    }
    unsafe fn menu(&self) -> HMENU {
        unsafe {
            let m = CreatePopupMenu().unwrap_or_default();
            let info = MENUINFO {
                cbSize: size_of::<MENUINFO>() as u32,
                fMask: MIM_BACKGROUND | MIM_STYLE,
                hbrBack: self.brush,
                dwStyle: MNS_NOCHECK,
                ..Default::default()
            };
            let _ = SetMenuInfo(m, &info);
            m
        }
    }
    #[allow(clippy::too_many_arguments)]
    unsafe fn item(
        &mut self,
        menu: HMENU,
        id: usize,
        label: &str,
        hint: &str,
        icon: char,
        checked: bool,
        enabled: bool,
        child: Option<HMENU>,
    ) {
        unsafe {
            let row = Box::new(Row {
                label: label.into(),
                hint: hint.into(),
                icon,
                checked,
                heading: id == 0 && !label.is_empty(),
                separator: label.is_empty(),
                danger: id == 218,
            });
            let flags = MF_STRING
                | if checked {
                    MF_CHECKED
                } else {
                    MENU_ITEM_FLAGS(0)
                }
                | if child.is_some() {
                    MF_POPUP
                } else {
                    MENU_ITEM_FLAGS(0)
                }
                | if enabled {
                    MENU_ITEM_FLAGS(0)
                } else {
                    MF_GRAYED
                };
            let _ = AppendMenuW(
                menu,
                flags,
                child.map_or(id, |m| m.0 as usize),
                &HSTRING::from(label),
            );
            let position = GetMenuItemCount(Some(menu)) - 1;
            let info = MENUITEMINFOW {
                cbSize: size_of::<MENUITEMINFOW>() as u32,
                fMask: MIIM_FTYPE | MIIM_DATA,
                fType: MFT_OWNERDRAW
                    | if row.separator {
                        MFT_SEPARATOR
                    } else {
                        MENU_ITEM_TYPE(0)
                    },
                dwItemData: row.as_ref() as *const Row as usize,
                ..Default::default()
            };
            let _ = SetMenuItemInfoW(menu, position as u32, true, &info);
            self.rows.push(row);
        }
    }
    unsafe fn separator(&mut self, menu: HMENU) {
        unsafe {
            self.item(menu, 0, "", "", '\0', false, false, None);
        }
    }
}
impl Drop for Popup {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyMenu(self.root);
            let _ = DeleteObject(self.font.into());
            let _ = DeleteObject(self.icons.into());
            let _ = DeleteObject(self.brush.into());
        }
    }
}

/// Called synchronously by the owner's window procedure while TrackPopupMenu
/// runs its native modal loop. No business state or RefCell borrow is held.
pub fn handle(msg: u32, l: LPARAM) -> Option<LRESULT> {
    if msg != WM_MEASUREITEM && msg != WM_DRAWITEM {
        return None;
    }
    ACTIVE.with(|active| unsafe {
        let p = active.get().as_ref()?;
        if msg == WM_MEASUREITEM {
            let item = &mut *(l.0 as *mut MEASUREITEMSTRUCT);
            if item.CtlType != ODT_MENU {
                return None;
            }
            let row = (item.itemData as *const Row).as_ref()?;
            item.itemWidth = p.px(268) as u32;
            item.itemHeight = p.px(if row.separator { 13 } else { 38 }) as u32;
        } else {
            let item = &*(l.0 as *const DRAWITEMSTRUCT);
            if item.CtlType != ODT_MENU {
                return None;
            }
            let row = (item.itemData as *const Row).as_ref()?;
            let dc = item.hDC;
            let saved = SaveDC(dc);
            FillRect(dc, &item.rcItem, p.brush);
            let mut r = item.rcItem;
            if row.separator {
                r.left += p.px(16);
                r.right -= p.px(16);
                r.top += p.px(6);
                r.bottom = r.top + p.px(1).max(1);
                let b = CreateSolidBrush(p.color(0xe7eaf0, 0x363d49, COLOR_GRAYTEXT));
                FillRect(dc, &r, b);
                let _ = DeleteObject(b.into());
            } else {
                let disabled = item.itemState.0 & ODS_DISABLED.0 != 0 || row.heading;
                let selected = item.itemState.0 & ODS_SELECTED.0 != 0 && !disabled;
                if selected {
                    let b = CreateSolidBrush(p.color(0xe8effc, 0x303e57, COLOR_HIGHLIGHT));
                    let old_b = SelectObject(dc, b.into());
                    let old_p = SelectObject(dc, GetStockObject(NULL_PEN));
                    let _ = RoundRect(
                        dc,
                        r.left + p.px(5),
                        r.top + p.px(2),
                        r.right - p.px(5),
                        r.bottom - p.px(2),
                        p.px(10),
                        p.px(10),
                    );
                    SelectObject(dc, old_p);
                    SelectObject(dc, old_b);
                    let _ = DeleteObject(b.into());
                }
                SetBkMode(dc, TRANSPARENT);
                let ink = if p.contrast && selected {
                    COLORREF(GetSysColor(COLOR_HIGHLIGHTTEXT))
                } else if disabled {
                    p.color(0x87909f, 0x8590a1, COLOR_GRAYTEXT)
                } else if row.danger {
                    p.color(0xb64a51, 0xf39097, COLOR_MENUTEXT)
                } else {
                    p.color(0x263144, 0xe8edf5, COLOR_MENUTEXT)
                };
                SetTextColor(
                    dc,
                    if row.checked && !disabled {
                        p.color(0x326fd0, 0x83afff, COLOR_MENUTEXT)
                    } else {
                        ink
                    },
                );
                SelectObject(dc, p.icons.into());
                let mut icon_rect = RECT {
                    left: r.left + p.px(14),
                    right: r.left + p.px(39),
                    ..r
                };
                let glyph = if row.checked { '\u{e73e}' } else { row.icon };
                if glyph != '\0' {
                    let mut text: Vec<u16> = glyph.to_string().encode_utf16().collect();
                    DrawTextW(
                        dc,
                        &mut text,
                        &mut icon_rect,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                    );
                }
                SelectObject(dc, p.font.into());
                SetTextColor(dc, ink);
                let mut label_rect = RECT {
                    left: r.left + p.px(if row.heading { 17 } else { 48 }),
                    right: r.right - p.px(18),
                    ..r
                };
                let mut text: Vec<u16> = row.label.encode_utf16().collect();
                DrawTextW(
                    dc,
                    &mut text,
                    &mut label_rect,
                    DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                );
                if !row.hint.is_empty() {
                    SetTextColor(dc, p.color(0x87909f, 0x99a6b9, COLOR_GRAYTEXT));
                    let mut text: Vec<u16> = row.hint.encode_utf16().collect();
                    label_rect.right -= p.px(12);
                    DrawTextW(
                        dc,
                        &mut text,
                        &mut label_rect,
                        DT_RIGHT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                    );
                }
            }
            let _ = RestoreDC(dc, saved);
        }
        Some(LRESULT(1))
    })
}

pub fn show(owner: HWND, cfg: &Settings, visible: bool, scale: f32) -> usize {
    unsafe {
        let dark = match cfg.theme {
            Theme::Dark => true,
            Theme::Light => false,
            Theme::System => crate::platform::system_dark(),
        };
        let font = |face: &windows::core::PCWSTR| {
            CreateFontW(
                -(14.0 * scale) as i32,
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
                *face,
            )
        };
        let mut p = Popup {
            root: HMENU::default(),
            rows: Vec::new(),
            font: font(&w!("Microsoft YaHei UI")),
            icons: font(&w!("Segoe MDL2 Assets")),
            brush: HBRUSH::default(),
            scale,
            dark,
            contrast: crate::material::high_contrast(),
        };
        p.brush = CreateSolidBrush(p.color(0xfafbfd, 0x222832, COLOR_MENU));
        p.root = p.menu();
        let modes = p.menu();
        let look = p.menu();
        let behavior = p.menu();
        for (id, label, icon, mode) in [
            (201, "边缘面板", '\u{e90f}', Mode::Edge),
            (202, "独立悬浮窗", '\u{e8a7}', Mode::Floating),
            (203, "桌面组件", '\u{e977}', Mode::Desktop),
        ] {
            p.item(modes, id, label, "", icon, cfg.mode == mode, true, None);
        }
        p.separator(modes);
        for (id, label, side) in [
            (204, "停靠左侧", Side::Left),
            (205, "停靠右侧", Side::Right),
        ] {
            p.item(
                modes,
                id,
                label,
                "",
                '\u{e90f}',
                cfg.side == side,
                cfg.mode == Mode::Edge,
                None,
            );
        }
        for (id, label, theme, icon) in [
            (211, "跟随系统", Theme::System, '\u{e770}'),
            (209, "浅色玻璃", Theme::Light, '\u{e706}'),
            (210, "深色玻璃", Theme::Dark, '\u{e708}'),
        ] {
            p.item(look, id, label, "", icon, cfg.theme == theme, true, None);
        }
        p.separator(look);
        p.item(
            look,
            220,
            "显示标题栏",
            "",
            '\u{e8a4}',
            cfg.titlebar,
            true,
            None,
        );
        for (id, label, icon, on, enabled) in [
            (
                206,
                "固定展开",
                '\u{e718}',
                cfg.pinned,
                cfg.mode == Mode::Edge,
            ),
            (
                221,
                "自动收回",
                '\u{e8a0}',
                cfg.auto_hide,
                cfg.mode == Mode::Edge,
            ),
            (
                207,
                "始终置顶",
                '\u{e74a}',
                cfg.topmost,
                cfg.mode == Mode::Floating,
            ),
            (208, "锁定位置与尺寸", '\u{e72e}', cfg.locked, true),
        ] {
            p.item(behavior, id, label, "", icon, on, enabled, None);
        }
        p.separator(behavior);
        p.item(
            behavior,
            215,
            "返回主显示器",
            "",
            '\u{e80f}',
            false,
            true,
            None,
        );
        let root = p.root;
        p.item(root, 0, "轻待 · LightDo", "", '\0', false, false, None);
        p.item(
            root,
            230,
            "窗口模式",
            match cfg.mode {
                Mode::Edge => "边缘面板",
                Mode::Floating => "悬浮窗",
                Mode::Desktop => "桌面组件",
            },
            '\u{e737}',
            false,
            true,
            Some(modes),
        );
        p.item(
            root,
            231,
            "外观",
            match cfg.theme {
                Theme::System => "跟随系统",
                Theme::Light => "浅色",
                Theme::Dark => "深色",
            },
            '\u{e790}',
            false,
            true,
            Some(look),
        );
        p.item(
            root,
            232,
            "位置与行为",
            "",
            '\u{e713}',
            false,
            true,
            Some(behavior),
        );
        p.separator(root);
        p.item(root, 214, "设置…", "", '\u{e713}', false, true, None);
        p.separator(root);
        p.item(
            root,
            219,
            if visible {
                "隐藏到托盘"
            } else {
                "显示轻待"
            },
            "",
            '\u{e8f4}',
            false,
            true,
            None,
        );
        p.item(root, 218, "退出轻待", "", '\u{e8ac}', false, true, None);
        ACTIVE.with(|a| a.set(&p));
        let mut point = POINT::default();
        let _ = GetCursorPos(&mut point);
        let _ = SetForegroundWindow(owner);
        let result = TrackPopupMenu(
            root,
            TPM_RETURNCMD | TPM_RIGHTBUTTON,
            point.x,
            point.y,
            Some(0),
            owner,
            None,
        )
        .0 as usize;
        ACTIVE.with(|a| a.set(std::ptr::null()));
        // Required for reliable tray-menu dismissal on the next invocation.
        let _ = PostMessageW(Some(owner), WM_NULL, WPARAM(0), LPARAM(0));
        result
    }
}

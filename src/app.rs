//! One native panel, a system EDIT for each active editor, no web runtime.
use crate::{
    geometry,
    material::Material,
    model::*,
    platform, settings_ui,
    state::*,
    storage::Store,
    ui::{
        gfx::{self, Gfx, Lens, Surface},
        scene::Scene,
    },
};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::{
            Direct2D::{
                Common::{D2D_RECT_F, D2D1_GRADIENT_STOP},
                *,
            },
            DirectWrite::*,
            Gdi::*,
        },
        System::{Com::*, LibraryLoader::*, RemoteDesktop::*, Threading::*},
        UI::{
            Controls::*,
            HiDpi::*,
            Input::{Ime::*, KeyboardAndMouse::*},
            Shell::*,
            WindowsAndMessaging::*,
        },
    },
    core::{BOOL, HSTRING, Result, w},
};
use windows_numerics::Vector2;

const FRAME: u32 = WM_APP + 1;
const COMMIT: u32 = WM_APP + 2;
const CANCEL: u32 = WM_APP + 3;
const TRAY: u32 = WM_APP + 4;
const KEY_NAV: u32 = WM_APP + 15;
const SHOW_REQUEST: u32 = WM_APP + 16;
const INPUT: usize = 101;
const INLINE: usize = 102;
const INPUT_TOP: f32 = 72.0;
const INPUT_HEIGHT: f32 = 40.0;
const EDGE_TRIM_AFTER: Duration = Duration::from_secs(5);
const EDGE_WARMUP_LIMIT: Duration = Duration::from_millis(500);
fn edit_key(dark: bool) -> COLORREF {
    COLORREF(if dark { 0x030201 } else { 0xfcfdfe })
}
fn font_metrics(font: HFONT, scale: f32) -> (i32, i32) {
    unsafe {
        let dc = CreateCompatibleDC(None);
        if dc.is_invalid() {
            return ((18.0 * scale).ceil() as i32, (15.0 * scale).round() as i32);
        }
        let old = SelectObject(dc, font.into());
        let mut metrics = TEXTMETRICW::default();
        let ok = GetTextMetricsW(dc, &mut metrics).as_bool();
        SelectObject(dc, old);
        let _ = DeleteDC(dc);
        if ok {
            (metrics.tmHeight.max(1), metrics.tmAscent)
        } else {
            ((18.0 * scale).ceil() as i32, (15.0 * scale).round() as i32)
        }
    }
}
#[derive(Clone, Copy)]
struct Event {
    hwnd: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
}
thread_local! {
    static EVENTS:RefCell<VecDeque<Event>>=const{RefCell::new(VecDeque::new())};
    static MAIN:Cell<HWND>=const{Cell::new(HWND(std::ptr::null_mut()))};
    static PANEL:Cell<HWND>=const{Cell::new(HWND(std::ptr::null_mut()))};
    static SCALE:Cell<f32>=const{Cell::new(1.0)};
    static LOCKED:Cell<bool>=const{Cell::new(false)};
    static TITLE_VISIBLE:Cell<bool>=const{Cell::new(true)};
    static PAINT_DARK:Cell<bool>=const{Cell::new(false)};
    static COMPOSING:Cell<bool>=const{Cell::new(false)};
    static LAST_FOCUS:Cell<HWND>=const{Cell::new(HWND(std::ptr::null_mut()))};
    static IME_ENDED:Cell<Option<Instant>>=const{Cell::new(None)};
    static EDIT_BUFFERS:RefCell<std::collections::HashMap<usize,String>>=RefCell::new(std::collections::HashMap::new());
}
fn queue(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) {
    if msg == WM_COMMAND && (w.0 >> 16) as u32 == EN_CHANGE {
        let id = w.0 & 0xffff;
        if id == INPUT || id == INLINE {
            let value = edit_text(HWND(l.0 as *mut _));
            EDIT_BUFFERS.with(|b| {
                b.borrow_mut().insert(id, value);
            });
        }
    }
    EVENTS.with(|e| e.borrow_mut().push_back(Event { hwnd, msg, w, l }));
}
unsafe extern "system" fn procedure(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if let Some(result) = crate::menu_ui::handle(msg, l) {
        return result;
    }
    if crate::menu_ui::is_open() && matches!(msg, WM_KEYDOWN | WM_SYSKEYDOWN) {
        return unsafe { DefWindowProcW(hwnd, msg, w, l) };
    }
    // A normal shutdown request must leave the popup's modal loop before
    // the application can drain its queued commands and save/exit.
    if crate::menu_ui::is_open() && (msg == WM_CLOSE || (msg == WM_COMMAND && w.0 & 0xffff == 218))
    {
        unsafe {
            let _ = EndMenu();
        }
    }
    match msg {
        WM_SYSKEYDOWN if w.0 == VK_F10.0 as usize => {
            queue(hwnd, WM_KEYDOWN, w, l);
            LRESULT(0)
        }
        WM_SYSCOMMAND if w.0 & 0xfff0 == SC_KEYMENU as usize && l.0 == 0 => {
            queue(hwnd, WM_KEYDOWN, WPARAM(VK_F10.0 as usize), l);
            LRESULT(0)
        }
        WM_ACTIVATE => {
            queue(hwnd, msg, w, l);
            unsafe {
                if hwnd == PANEL.with(Cell::get) && w.0 & 0xffff == WA_INACTIVE as usize {
                    let focus = GetFocus();
                    if IsChild(hwnd, focus).as_bool() {
                        LAST_FOCUS.with(|f| f.set(focus));
                    }
                }
                let result = DefWindowProcW(hwnd, msg, w, l);
                if hwnd == PANEL.with(Cell::get)
                    && w.0 & 0xffff != WA_INACTIVE as usize
                    && !crate::menu_ui::is_open()
                    && !settings_ui::is_open()
                {
                    let focus = LAST_FOCUS.with(Cell::get);
                    if IsWindow(Some(focus)).as_bool() && IsChild(hwnd, focus).as_bool() {
                        let _ = SetFocus(Some(focus));
                    }
                }
                result
            }
        }
        WM_NCHITTEST => {
            // Dragging is handled in the regular message loop, so background
            // capture and IME keep running while a panel moves or resizes.
            LRESULT(HTCLIENT as isize)
        }
        WM_GETMINMAXINFO => {
            let scale = SCALE.with(Cell::get);
            unsafe {
                let info = &mut *(l.0 as *mut MINMAXINFO);
                info.ptMinTrackSize = POINT {
                    x: (320.0 * scale) as i32,
                    y: (360.0 * scale) as i32,
                };
                info.ptMaxTrackSize = POINT {
                    x: (1000.0 * scale) as i32,
                    y: (1600.0 * scale) as i32,
                };
            }
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_NCDESTROY => unsafe {
            let _ = RemovePropW(hwnd, w!("LightDo.Runtime"));
            DefWindowProcW(hwnd, msg, w, l)
        },
        WM_PAINT => {
            unsafe {
                let mut p = PAINTSTRUCT::default();
                let _ = BeginPaint(hwnd, &mut p);
                let _ = EndPaint(hwnd, &p);
            }
            queue(hwnd, FRAME, w, l);
            LRESULT(0)
        }
        WM_CLOSE
        | WM_DESTROY
        | WM_SIZE
        | WM_MOVE
        | WM_TIMER
        | WM_HOTKEY
        | WM_DISPLAYCHANGE
        | WM_SETTINGCHANGE
        | WM_DPICHANGED
        | WM_ENTERSIZEMOVE
        | WM_EXITSIZEMOVE
        | WM_LBUTTONUP
        | WM_LBUTTONDOWN
        | WM_CAPTURECHANGED
        | WM_LBUTTONDBLCLK
        | WM_MOUSEMOVE
        | WM_MOUSELEAVE
        | WM_MOUSEWHEEL
        | WM_RBUTTONUP
        | WM_COMMAND
        | WM_KEYDOWN
        | KEY_NAV
        | settings_ui::CHANGED
        | SHOW_REQUEST
        | WM_POWERBROADCAST
        | WM_WTSSESSION_CHANGE
        | FRAME
        | COMMIT
        | CANCEL
        | TRAY => {
            queue(hwnd, msg, w, l);
            LRESULT(0)
        }
        WM_CTLCOLOREDIT => {
            unsafe {
                let dc = HDC(w.0 as *mut _);
                let dark = PAINT_DARK.with(Cell::get);
                SetTextColor(dc, COLORREF(if dark { 0xf5f5f5 } else { 0x302420 }));
                let background = edit_key(dark);
                SetBkColor(dc, background);
                SetDCBrushColor(dc, background);
            }
            LRESULT(unsafe { GetStockObject(DC_BRUSH) }.0 as isize)
        }
        _ if msg >= 0xC000 => {
            queue(hwnd, msg, w, l);
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, w, l) },
    }
}
unsafe extern "system" fn edit_proc(
    hwnd: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
    _: usize,
    control_id: usize,
) -> LRESULT {
    match msg {
        WM_PAINT
            if control_id == INPUT
                && unsafe { GetWindowTextLengthW(hwnd) } == 0
                && !COMPOSING.with(Cell::get) =>
        {
            // Let EDIT update its internal paint/cue state before replacing
            // only the empty hint. Actual editing remains native.
            unsafe {
                let result = DefSubclassProc(hwnd, msg, w, l);
                let dc = GetDC(Some(hwnd));
                if dc.is_invalid() {
                    return result;
                }
                let dark = PAINT_DARK.with(Cell::get);
                let mut rect = RECT::default();
                let _ = GetClientRect(hwnd, &mut rect);
                SetDCBrushColor(dc, edit_key(dark));
                FillRect(dc, &rect, HBRUSH(GetStockObject(DC_BRUSH).0));
                SetBkMode(dc, TRANSPARENT);
                SetTextColor(dc, COLORREF(if dark { 0xd2bdac } else { 0x9c8471 }));
                let font = SendMessageW(hwnd, WM_GETFONT, None, None);
                let old = SelectObject(dc, HGDIOBJ(font.0 as *mut _));
                let mut hint: Vec<u16> = "写下一件事…".encode_utf16().collect();
                DrawTextW(
                    dc,
                    &mut hint,
                    &mut rect,
                    DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
                );
                SelectObject(dc, old);
                ReleaseDC(Some(hwnd), dc);
                return result;
            }
        }
        WM_KEYDOWN if w.0 == VK_TAB.0 as usize && !COMPOSING.with(Cell::get) => {
            MAIN.with(|m| unsafe {
                let _ = PostMessageW(Some(m.get()), KEY_NAV, WPARAM(0), LPARAM(0));
            });
            return LRESULT(0);
        }
        WM_IME_STARTCOMPOSITION => {
            COMPOSING.with(|c| c.set(true));
            if control_id == INPUT {
                // The IME may draw its composition in a separate window,
                // without making EDIT repaint the previous empty hint.
                unsafe {
                    SendMessageW(
                        hwnd,
                        EM_SETCUEBANNER,
                        Some(WPARAM(0)),
                        Some(LPARAM(w!("").0 as isize)),
                    );
                    let _ = InvalidateRect(Some(hwnd), None, true);
                    let _ = UpdateWindow(hwnd);
                }
            }
        }
        WM_IME_ENDCOMPOSITION => {
            COMPOSING.with(|c| c.set(false));
            IME_ENDED.with(|c| c.set(Some(Instant::now())));
            if control_id == INPUT {
                unsafe {
                    SendMessageW(
                        hwnd,
                        EM_SETCUEBANNER,
                        Some(WPARAM(1)),
                        Some(LPARAM(w!("写下一件事…").0 as isize)),
                    );
                    let _ = InvalidateRect(Some(hwnd), None, true);
                }
            }
        }
        WM_KEYDOWN if w.0 == VK_RETURN.0 as usize => {
            let recent = IME_ENDED.with(|c| c.get().is_some_and(|t| t.elapsed().as_millis() < 120));
            if !COMPOSING.with(Cell::get) && !recent {
                MAIN.with(|m| unsafe {
                    let _ = PostMessageW(Some(m.get()), COMMIT, WPARAM(hwnd.0 as usize), LPARAM(0));
                });
                return LRESULT(0);
            }
        }
        WM_CHAR if w.0 == 13 || w.0 == 10 => return LRESULT(0),
        WM_KEYDOWN if w.0 == VK_ESCAPE.0 as usize && !COMPOSING.with(Cell::get) => {
            MAIN.with(|m| unsafe {
                let _ = PostMessageW(Some(m.get()), CANCEL, WPARAM(hwnd.0 as usize), LPARAM(0));
            });
            return LRESULT(0);
        }
        WM_KILLFOCUS => {
            MAIN.with(|m| unsafe {
                let _ = PostMessageW(Some(m.get()), COMMIT, WPARAM(hwnd.0 as usize), LPARAM(1));
                let _ = PostMessageW(Some(m.get()), FRAME, WPARAM(0), LPARAM(0));
            });
        }
        WM_SETFOCUS => MAIN.with(|m| unsafe {
            let _ = PostMessageW(Some(m.get()), FRAME, WPARAM(0), LPARAM(0));
        }),
        _ => {}
    }
    unsafe { DefSubclassProc(hwnd, msg, w, l) }
}
fn edit_text(hwnd: HWND) -> String {
    unsafe {
        let length = GetWindowTextLengthW(hwnd);
        let mut text = vec![0u16; length as usize + 1];
        let n = GetWindowTextW(hwnd, &mut text);
        String::from_utf16_lossy(&text[..n as usize])
    }
}
fn set_text(hwnd: HWND, text: &str) {
    unsafe {
        let _ = SetWindowTextW(hwnd, &HSTRING::from(text));
    }
}

struct Row {
    id: String,
    top: f32,
    height: f32,
    layout: IDWriteTextLayout,
    text_height: f32,
    baseline: f32,
}
impl Row {
    fn center(&self) -> f32 {
        self.top + (self.height - 4.0) / 2.0
    }
    fn text_top(&self, scale: f32) -> f32 {
        ((self.center() - self.text_height / 2.0) * scale).round() / scale
    }
}
#[derive(Clone, Copy)]
struct Drag {
    origin: POINT,
    rect: geometry::Rect,
    kind: geometry::Gesture,
}
struct App {
    controller: HWND,
    hwnd: HWND,
    input: HWND,
    inline: HWND,
    editing: Option<String>,
    store: Store,
    model: Model,
    gfx: Rc<Gfx>,
    material: Material,
    surface: Option<Surface>,
    scene: Option<Scene>,
    lens: Option<Lens>,
    background: Option<(ID2D1Bitmap1, RECT)>,
    state: WindowState,
    start: Instant,
    last_tick: Instant,
    scale: f32,
    size: (u32, u32),
    rows: Vec<Row>,
    scroll: f32,
    hover: Option<String>,
    header_hover: Option<usize>,
    selected: Option<String>,
    dirty: bool,
    visible: bool,
    minimized: bool,
    in_menu: bool,
    moving: bool,
    drag: Option<Drag>,
    snap_preview: Option<Side>,
    desktop: Option<HWND>,
    error: Option<String>,
    quit: bool,
    font: HFONT,
    font_line_px: i32,
    font_ascent_px: i32,
    taskbar_created: u32,
    icon: HICON,
    graphic_retry: Option<Instant>,
    session_locked: bool,
    suspended: bool,
    edge_idle_since: Option<Instant>,
    runtime_snapshot: Cell<usize>,
    timer_interval: Cell<Option<u32>>,
    edge_open_since: Option<Instant>,
    pending_activation: bool,
    activation_grace_until: Option<Instant>,
}
fn header_rect(width: f32, index: usize) -> D2D_RECT_F {
    gfx::rect(width - 133.0 + 41.0 * index as f32, 17.0, 32.0, 32.0)
}
impl App {
    fn edge_warming(&self) -> bool {
        self.model.settings.mode == Mode::Edge
            && self.state.target == 1.0
            && self.state.progress == 0.0
            && matches!(self.state.phase, Phase::Opening | Phase::Pinned)
            && !self.visible
    }
    fn header_hit(&self, x: f32, y: f32) -> Option<usize> {
        let w = self.size.0 as f32 / self.scale;
        if !self.model.settings.titlebar {
            return None;
        }
        (0..3).find(|index| {
            let r = header_rect(w, *index);
            x >= r.left - 4.0 && x < r.right + 4.0 && y >= r.top - 4.0 && y < r.bottom + 4.0
        })
    }
    fn begin_drag(&mut self, x: f32, y: f32) -> bool {
        if self.model.settings.locked {
            return false;
        }
        let width = self.size.0 as f32 / self.scale;
        let height = self.size.1 as f32 / self.scale;
        let (left, right, top, bottom) = (x < 10.0, x > width - 10.0, y < 10.0, y > height - 10.0);
        let kind = if left || right || top || bottom {
            geometry::Gesture::Resize {
                left,
                right,
                top,
                bottom,
            }
        } else if y < if self.model.settings.titlebar {
            52.0
        } else {
            26.0
        } && x < width - 152.0
        {
            geometry::Gesture::Move
        } else {
            return false;
        };
        let mut r = RECT::default();
        // Mouse messages are queued for the regular application loop. The
        // cursor may already have moved when that loop consumes mouse-down;
        // derive its origin from the message instead of sampling it later.
        let mut p = POINT {
            x: (x * self.scale).round() as i32,
            y: (y * self.scale).round() as i32,
        };
        unsafe {
            let _ = GetWindowRect(self.hwnd, &mut r);
            let _ = ClientToScreen(self.hwnd, &mut p);
        }
        self.model.settings.x = r.left;
        self.model.settings.y = r.top;
        if self.model.settings.mode != Mode::Floating {
            self.mode(Mode::Floating);
        }
        self.drag = Some(Drag {
            origin: p,
            rect: geometry::Rect {
                x: r.left,
                y: r.top,
                w: r.right - r.left,
                h: r.bottom - r.top,
            },
            kind,
        });
        self.moving = true;
        self.state.phase = Phase::Moving;
        unsafe {
            SetCapture(self.hwnd);
        }
        true
    }
    fn update_drag(&mut self) {
        let Some(drag) = self.drag else { return };
        let mut p = POINT::default();
        unsafe {
            let _ = GetCursorPos(&mut p);
        }
        let r = geometry::drag(
            drag.rect,
            drag.kind,
            (p.x - drag.origin.x, p.y - drag.origin.y),
            self.scale,
        );
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                None,
                r.x,
                r.y,
                r.w,
                r.h,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
        self.model.settings.x = r.x;
        self.model.settings.y = r.y;
        self.model.settings.width = (r.w as f32 / self.scale).round() as i32;
        self.model.settings.height = (r.h as f32 / self.scale).round() as i32;
        let work = platform::primary().rcWork;
        self.snap_preview = if (r.x - work.left).abs() < (48.0 * self.scale) as i32 {
            Some(Side::Left)
        } else if (r.x + r.w - work.right).abs() < (48.0 * self.scale) as i32 {
            Some(Side::Right)
        } else {
            None
        };
        self.arrange_background();
        self.dirty = true;
    }
    fn end_drag(&mut self, snap: bool) {
        if self.drag.take().is_none() {
            return;
        }
        self.moving = false;
        unsafe {
            let _ = ReleaseCapture();
        }
        if snap {
            if let Some(side) = self.snap_preview.take() {
                self.model.settings.side = side;
                self.mode(Mode::Edge);
            }
        }
        self.snap_preview = None;
        self.state.mode(self.model.settings.mode);
        self.save();
    }
    fn release_surface(&mut self) {
        if let Some(surface) = &mut self.surface {
            surface.release(&self.gfx);
        }
    }
    fn rest_capture(&mut self) {
        if let Some(scene) = &mut self.scene {
            scene.rest();
        }
        self.background = None;
    }
    fn idle_graphics(&mut self) {
        // Reuse the WGC frame pool: repeatedly creating pools retains driver
        // allocations even after the application releases their COM objects.
        self.rest_capture();
        self.lens = None;
        self.release_surface();
        self.gfx.trim_idle();
    }
    fn recover_graphics(&mut self, error: windows::core::Error) {
        self.error = Some(format!("图形设备暂不可用，正在恢复：{}", error.code()));
        if self
            .graphic_retry
            .is_some_and(|at| at.elapsed().as_secs() < 1)
        {
            return;
        }
        self.graphic_retry = Some(Instant::now());
        self.scene = None;
        self.background = None;
        self.lens = None;
        self.surface = None;
        unsafe {
            let _ = self.gfx.dcomp.Commit();
        }
        if let Ok(gfx) = Gfx::new() {
            self.gfx = Rc::new(gfx);
            self.dirty = true;
        }
    }
    fn restore_panel(&mut self) -> Result<()> {
        let input = EDIT_BUFFERS
            .with(|b| b.borrow().get(&INPUT).cloned())
            .unwrap_or_default();
        let inline = EDIT_BUFFERS
            .with(|b| b.borrow().get(&INLINE).cloned())
            .unwrap_or_default();
        let editing = self.editing.take();
        self.surface = None;
        self.scene = None;
        self.background = None;
        self.lens = None;
        self.hwnd = unsafe { make_panel()? };
        self.input = self.editor(INPUT)?;
        set_text(self.input, &input);
        self.inline = HWND::default();
        self.desktop = None;
        self.model.settings.mode = Mode::Floating;
        self.state.mode(Mode::Floating);
        if let Some(id) = editing {
            self.inline = self.editor(INLINE)?;
            self.editing = Some(id);
            set_text(self.inline, &inline);
        }
        self.surface = Some(Surface::new(&self.gfx, self.hwnd)?);
        self.error = Some("桌面宿主已变化，已恢复悬浮窗；任务和正在编辑的文字已保留。".into());
        self.show(false);
        self.save();
        Ok(())
    }
    fn save(&mut self) {
        if let Err(e) = self.store.save(&self.model) {
            if self.error.as_deref() != Some(e.as_str()) {
                self.message(&e);
            }
            self.error = Some(e)
        }
        self.dirty = true;
    }
    fn open_settings(&mut self) {
        let diagnostic = format!(
            "背景：{} · {}\n数据仅保存在本机；安装包未签名。",
            if !self.material.glass() {
                self.material.reason()
            } else if self.background.is_some() {
                "实时背景合成"
            } else {
                "降级 / 背景不可用"
            },
            if self.gfx.software {
                "软件渲染"
            } else {
                "Direct3D 硬件渲染"
            }
        );
        self.show(false);
        if let Err(e) = settings_ui::open(self.hwnd, &self.model.settings, &diagnostic) {
            self.error = Some(e.to_string());
        }
    }
    fn apply_settings(&mut self, mut next: Settings) {
        if !self.store.writable {
            self.message("原设置未成功读取，已禁止覆盖。请先修复数据文件并重启。");
            return;
        }
        let old = self.model.settings.clone();
        if (next.hotkey_mod, next.hotkey_vk) != (old.hotkey_mod, old.hotkey_vk) {
            unsafe {
                let _ = UnregisterHotKey(Some(self.controller), 1);
                if RegisterHotKey(
                    Some(self.controller),
                    1,
                    HOT_KEY_MODIFIERS(next.hotkey_mod) | MOD_NOREPEAT,
                    next.hotkey_vk,
                )
                .is_err()
                {
                    let _ = RegisterHotKey(
                        Some(self.controller),
                        1,
                        HOT_KEY_MODIFIERS(old.hotkey_mod) | MOD_NOREPEAT,
                        old.hotkey_vk,
                    );
                    self.message("新快捷键已被占用，已保留原快捷键。请重新设置。");
                    next.hotkey_mod = old.hotkey_mod;
                    next.hotkey_vk = old.hotkey_vk;
                }
            }
        }
        if next.autostart != old.autostart {
            if let Err(e) = platform::autostart(next.autostart) {
                next.autostart = old.autostart;
                self.error = Some(e);
            }
        }
        let requested = next.mode;
        next.mode = old.mode;
        self.model.settings = next;
        LOCKED.with(|l| l.set(self.model.settings.locked));
        self.mode(requested);
        if requested == Mode::Floating
            && old.mode == requested
            && old.topmost != self.model.settings.topmost
        {
            unsafe {
                let _ = SetWindowPos(
                    self.hwnd,
                    Some(if self.model.settings.topmost {
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
        self.reposition();
        settings_ui::sync_layer();
        self.save();
    }
    fn message(&self, text: &str) {
        unsafe {
            MessageBoxW(
                Some(self.hwnd),
                &HSTRING::from(text),
                w!("轻待 LightDo"),
                MB_OK | MB_ICONINFORMATION,
            );
        }
    }
    fn activate_input(&mut self) {
        unsafe {
            let _ = SetForegroundWindow(self.hwnd);
            let _ = SetFocus(Some(self.input));
            self.activation_grace_until = if GetForegroundWindow() == self.hwnd {
                None
            } else {
                Some(Instant::now() + Duration::from_millis(1500))
            };
        }
    }
    fn update_font(&mut self) {
        unsafe {
            let font = CreateFontW(
                -(15.0 * self.scale) as i32,
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
                ANTIALIASED_QUALITY,
                DEFAULT_PITCH.0 as u32,
                w!("Microsoft YaHei UI"),
            );
            if !font.is_invalid() {
                for hwnd in [self.input, self.inline] {
                    if !hwnd.is_invalid() {
                        SendMessageW(
                            hwnd,
                            WM_SETFONT,
                            Some(WPARAM(font.0 as usize)),
                            Some(LPARAM(1)),
                        );
                    }
                }
                let old = std::mem::replace(&mut self.font, font);
                (self.font_line_px, self.font_ascent_px) = font_metrics(font, self.scale);
                let _ = DeleteObject(old.into());
            }
        }
    }
    fn show(&mut self, activate: bool) {
        self.edge_idle_since = None;
        if self.model.settings.mode == Mode::Edge && !self.visible {
            self.minimized = false;
            self.pending_activation |= activate;
            self.state.progress = 0.0;
            self.state.show();
            self.edge_open_since.get_or_insert_with(Instant::now);
            self.last_tick = Instant::now();
            self.reposition();
            self.schedule_tick();
            self.arrange_background();
            self.dirty = true;
            return;
        }
        self.visible = true;
        self.minimized = false;
        self.state.show();
        self.last_tick = Instant::now();
        self.schedule_tick();
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                Some(
                    if self.model.settings.mode == Mode::Edge
                        || (self.model.settings.mode == Mode::Floating
                            && self.model.settings.topmost)
                    {
                        HWND_TOPMOST
                    } else {
                        HWND_NOTOPMOST
                    },
                ),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
            let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
        }
        self.dirty = true;
        self.reposition();
        if activate {
            self.activate_input()
        }
    }
    fn hide_tray(&mut self) {
        settings_ui::close();
        self.state.tray_hide();
        self.edge_idle_since = None;
        self.edge_open_since = None;
        self.pending_activation = false;
        self.activation_grace_until = None;
        self.visible = false;
        self.idle_graphics();
        self.schedule_tick();
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
        if !self.model.settings.tray_tip_shown {
            self.model.settings.tray_tip_shown = true;
            self.save();
            self.tray_hint();
        }
    }
    fn editor(&self, id: usize) -> Result<HWND> {
        unsafe {
            let h = CreateWindowExW(
                WS_EX_LAYERED,
                w!("EDIT"),
                w!(""),
                WS_CHILD
                    | WS_VISIBLE
                    | WS_TABSTOP
                    | WINDOW_STYLE(if id == INLINE {
                        (ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN) as u32
                    } else {
                        ES_AUTOHSCROLL as u32
                    }),
                0,
                0,
                10,
                10,
                Some(self.hwnd),
                Some(HMENU(id as *mut _)),
                None,
                None,
            )?;
            let dark = match self.model.settings.theme {
                Theme::Dark => true,
                Theme::Light => false,
                Theme::System => platform::system_dark(),
            };
            PAINT_DARK.with(|d| d.set(dark));
            SetLayeredWindowAttributes(h, edit_key(dark), 255, LWA_ALPHA | LWA_COLORKEY)?;
            SendMessageW(
                h,
                WM_SETFONT,
                Some(WPARAM(self.font.0 as usize)),
                Some(LPARAM(1)),
            );
            SendMessageW(h, EM_SETLIMITTEXT, Some(WPARAM(32767)), Some(LPARAM(0)));
            SendMessageW(
                h,
                EM_SETMARGINS,
                Some(WPARAM((EC_LEFTMARGIN | EC_RIGHTMARGIN) as usize)),
                Some(LPARAM(0)),
            );
            let _ = SetWindowSubclass(h, Some(edit_proc), 1, id);
            Ok(h)
        }
    }
    fn layout_editors(&self) {
        let s = self.scale;
        let shift = if self.model.settings.titlebar {
            0.0
        } else {
            -44.0
        };
        self.place_editor(
            self.input,
            (32.0 * s) as i32,
            (((INPUT_TOP + shift) * s + (INPUT_HEIGHT * s - self.font_line_px as f32) / 2.0)
                .round()) as i32,
            (self.size.0 as f32 - 98.0 * s).max(40.0) as i32,
            self.font_line_px,
        );
        if self.editing.is_some() {
            if let Some(row) = self
                .rows
                .iter()
                .find(|r| Some(&r.id) == self.editing.as_ref())
            {
                let edit_y =
                    ((row.text_top(s) + row.baseline) * s).round() as i32 - self.font_ascent_px;
                self.place_editor(
                    self.inline,
                    (58.0 * s) as i32,
                    edit_y,
                    (self.size.0 as f32 - 124.0 * s) as i32,
                    (((row.top + row.height - 4.0) * s).floor() as i32 - edit_y)
                        .max(self.font_line_px),
                );
            }
        }
    }
    fn place_editor(&self, edit: HWND, x: i32, y: i32, width: i32, height: i32) {
        unsafe {
            let mut current = RECT::default();
            let mut at = POINT::default();
            if GetWindowRect(edit, &mut current).is_ok() {
                at.x = current.left;
                at.y = current.top;
                if ScreenToClient(self.hwnd, &mut at).as_bool()
                    && at.x == x
                    && at.y == y
                    && current.right - current.left == width
                    && current.bottom - current.top == height
                {
                    return;
                }
            }
            // Do not reset native EDIT layout/caret on every captured frame.
            let _ = SetWindowPos(
                edit,
                None,
                x,
                y,
                width,
                height,
                SWP_NOACTIVATE | SWP_NOZORDER,
            );
            if edit == self.inline {
                let format = RECT {
                    left: 0,
                    top: 0,
                    right: width,
                    bottom: height,
                };
                SendMessageW(
                    edit,
                    EM_SETRECTNP,
                    None,
                    Some(LPARAM(&format as *const RECT as isize)),
                );
            }
        }
    }
    fn commit(&mut self, hwnd: HWND, blur: bool) {
        if !self.store.writable {
            self.message("原数据未成功读取。为防止覆盖，当前禁止修改任务。请修复 state.json 后重启；原文件保持不变。");
            return;
        }
        if COMPOSING.with(Cell::get) {
            return;
        }
        if hwnd == self.inline && !self.inline.is_invalid() {
            if let Some(id) = self.editing.take() {
                self.model.edit(&id, &edit_text(self.inline));
                unsafe {
                    let _ = DestroyWindow(self.inline);
                }
                self.inline = HWND::default();
                self.save();
            }
        } else if hwnd == self.input && !blur {
            let value = edit_text(self.input);
            if self.model.add(&value).is_some() {
                self.save();
                set_text(self.input, "");
                self.activate_input();
                self.scroll = 0.0;
            }
        }
    }
    fn cancel(&mut self, hwnd: HWND) {
        if hwnd == self.inline && self.editing.take().is_some() {
            unsafe {
                let _ = DestroyWindow(self.inline);
            }
            self.inline = HWND::default();
            self.dirty = true;
        } else if self.model.settings.mode == Mode::Edge && !self.model.settings.pinned {
            unsafe {
                let _ = SetFocus(Some(self.hwnd));
            }
            self.state.blocked = false;
            self.state.dismiss();
        }
    }
    fn start_edit(&mut self, id: String) {
        if self.editing.is_some() {
            self.commit(self.inline, false);
            if self.editing.is_some() {
                return;
            }
        }
        if let Some(todo) = self.model.todos.iter().find(|t| t.id == id) {
            let value = todo.text.clone();
            match self.editor(INLINE) {
                Ok(hwnd) => {
                    self.inline = hwnd;
                    self.editing = Some(id);
                    set_text(hwnd, &value);
                    self.layout_editors();
                    self.dirty = true;
                    unsafe {
                        let _ = SetFocus(Some(hwnd));
                        SendMessageW(hwnd, EM_SETSEL, Some(WPARAM(0)), Some(LPARAM(-1)));
                    }
                }
                Err(e) => self.error = Some(e.to_string()),
            }
        }
    }
    fn mode(&mut self, mode: Mode) {
        let old = self.model.settings.mode;
        if mode == old {
            return;
        }
        if old == Mode::Desktop {
            let _ = platform::set_desktop(self.hwnd, None);
            self.desktop = None;
        }
        if mode == Mode::Desktop {
            if let Some(host) = platform::desktop_host() {
                match platform::set_desktop(self.hwnd, Some(host)) {
                    Ok(()) => self.desktop = Some(host),
                    Err(e) => {
                        self.error = Some(e);
                        self.model.settings.mode = Mode::Floating;
                        self.state.mode(Mode::Floating);
                        return;
                    }
                }
            } else {
                self.error = Some("桌面宿主不可用，已恢复普通悬浮窗".into());
                self.mode(Mode::Floating);
                return;
            }
        }
        self.model.settings.mode = mode;
        self.edge_idle_since = None;
        self.state.mode(mode);
        self.rest_capture();
        self.visible = true;
        self.last_tick = Instant::now();
        self.schedule_tick();
        unsafe {
            let top =
                if mode == Mode::Edge || (mode == Mode::Floating && self.model.settings.topmost) {
                    HWND_TOPMOST
                } else {
                    HWND_NOTOPMOST
                };
            let _ = SetWindowPos(
                self.hwnd,
                Some(top),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
            let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
        }
        self.reposition();
        settings_ui::sync_layer();
        self.save();
    }
    fn reposition(&mut self) {
        let cfg = &self.model.settings;
        let work = platform::primary().rcWork;
        let s = self.scale;
        let w = ((cfg.width as f32 * s) as i32).min(work.right - work.left - 32);
        let h = ((cfg.height as f32 * s) as i32).min(work.bottom - work.top - 32);
        let (x, y) = if cfg.mode == Mode::Edge {
            let shown = if cfg.side == Side::Right {
                work.right - w - 12
            } else {
                work.left + 12
            };
            let hidden = if cfg.side == Side::Right {
                work.right + 8
            } else {
                work.left - w - 8
            };
            let t = self.state.progress;
            let p = t * t * (3.0 - 2.0 * t);
            (
                (hidden as f32 + (shown - hidden) as f32 * p) as i32,
                work.top + ((work.bottom - work.top - h) / 2).max(12),
            )
        } else {
            platform::clamp_position(cfg.x, cfg.y, w, h)
        };
        let point = POINT { x, y };
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                None,
                point.x,
                point.y,
                w,
                h,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }
    fn schedule_tick(&self) {
        unsafe {
            let mode = match self.model.settings.mode {
                Mode::Edge => 1,
                Mode::Floating => 2,
                Mode::Desktop => 3,
            };
            let snapshot = 0x100
                | mode
                | ((self.state.phase as usize) << 2)
                | (usize::from(self.scene.as_ref().is_some_and(|s| !s.is_resting())) << 6)
                | (usize::from(self.background.is_some()) << 7)
                | (usize::from(self.scene.is_some()) << 9);
            if self.runtime_snapshot.get() != snapshot
                && SetPropW(
                    self.hwnd,
                    w!("LightDo.Runtime"),
                    Some(HANDLE(snapshot as *mut _)),
                )
                .is_ok()
            {
                self.runtime_snapshot.set(snapshot);
            }
            let interval = self.state.timer_interval(
                self.visible,
                self.session_locked || self.suspended,
                self.minimized,
            );
            if self.timer_interval.get() == interval {
                return;
            }
            match interval {
                Some(interval) => {
                    if SetTimer(Some(self.controller), 1, interval, None) == 0 {
                        return;
                    }
                }
                None => {
                    let _ = KillTimer(Some(self.controller), 1);
                }
            }
            self.timer_interval.set(interval);
        }
    }
    fn tick(&mut self) {
        if self
            .state
            .timer_interval(
                self.visible,
                self.session_locked || self.suspended,
                self.minimized,
            )
            .is_none()
        {
            self.schedule_tick();
            return;
        }
        let now = Instant::now();
        self.activation_grace_until = self.activation_grace_until.filter(|until| now < *until);
        let elapsed = now.duration_since(self.last_tick).as_secs_f32() * 1000.0;
        self.last_tick = now;
        let mut p = POINT::default();
        let previous_target = self.state.target;
        {
            unsafe {
                let _ = GetCursorPos(&mut p);
            }
            let mut r = RECT::default();
            unsafe {
                let _ = GetWindowRect(self.hwnd, &mut r);
            }
            let inside =
                p.x >= r.left - 8 && p.x < r.right + 8 && p.y >= r.top - 8 && p.y < r.bottom + 8;
            let cfg = &self.model.settings;
            self.state.pinned = cfg.pinned && cfg.mode == Mode::Edge;
            let active = unsafe { GetForegroundWindow() } == self.hwnd;
            self.state.blocked = self.in_menu
                || self.pending_activation
                || self.activation_grace_until.is_some()
                || settings_ui::is_open()
                || self.moving
                || COMPOSING.with(Cell::get)
                || (active && unsafe { GetFocus() } == self.input)
                || (active && !self.inline.is_invalid() && unsafe { GetFocus() } == self.inline);
            let primary = platform::primary().rcMonitor;
            let at = if cfg.side == Side::Right {
                (p.x - primary.right).abs() <= 2
            } else {
                (p.x - primary.left).abs() <= 2
            };
            let edge = at
                && p.y > primary.top + 48
                && p.y < primary.bottom - 48
                && (!cfg.fullscreen_pause || !platform::fullscreen_other(self.hwnd));
            if self.visible || self.state.phase == Phase::Hidden || self.edge_warming() {
                self.state.pointer(
                    self.start.elapsed().as_millis() as u64,
                    inside,
                    edge,
                    cfg.dwell_ms,
                    cfg.hide_ms,
                    cfg.auto_hide && (cfg.introduced || self.start.elapsed().as_secs() >= 12),
                );
            }
            if !self.model.settings.introduced && self.start.elapsed().as_secs() >= 12 {
                self.model.settings.introduced = true;
                self.save();
            }
        }
        let mut animation = BOOL(1);
        if (self.state.progress - self.state.target).abs() > 0.001 {
            unsafe {
                let _ = SystemParametersInfoW(
                    SPI_GETCLIENTAREAANIMATION,
                    0,
                    Some(&mut animation as *mut _ as *mut _),
                    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
                );
            }
        }
        if self.edge_warming() {
            self.arrange_background();
            let since = *self.edge_open_since.get_or_insert(now);
            if self.material.glass()
                && self.background.is_none()
                && now.duration_since(since) < EDGE_WARMUP_LIMIT
            {
                self.schedule_tick();
                return;
            }
            // Build the first complete surface while the panel is naturally
            // hidden. Showing it never exposes an unpainted/old surface.
            self.reposition();
            if let Err(error) = self.draw() {
                self.recover_graphics(error);
                self.schedule_tick();
                return;
            }
            unsafe {
                let _ = self.gfx.dcomp.WaitForCommitCompletion();
            }
            self.visible = true;
            self.edge_idle_since = None;
            self.edge_open_since = None;
            self.last_tick = Instant::now();
            unsafe {
                let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
                let _ = InvalidateRect(Some(self.input), None, true);
            }
            self.schedule_tick();
            return;
        }
        let frame_elapsed = if self.state.target != previous_target {
            0.0
        } else {
            // Preserve wall-clock duration under a missed frame; only a long
            // stall is clamped so reopening never jumps from hidden to done.
            elapsed.min(64.0)
        };
        if self.state.step(frame_elapsed, !animation.as_bool()) {
            if !self.visible && self.state.target == 1.0 {
                self.edge_idle_since = None;
                self.visible = true;
                unsafe {
                    let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
                }
            }
            self.reposition();
            self.dirty = true;
        }
        // A fully off-screen HWND cannot reliably take foreground focus.
        // Keep explicit reveal blocked against auto-hide until the input's
        // side of the panel has actually entered the work area.
        if self.visible && self.pending_activation && self.state.progress >= 0.5 {
            self.pending_activation = false;
            self.activate_input();
        }
        if self.state.phase == Phase::Hidden
            && !self.visible
            && self.edge_open_since.take().is_some()
        {
            self.pending_activation = false;
            self.idle_graphics();
        }
        if self.state.phase == Phase::Hidden && self.visible {
            self.visible = false;
            unsafe {
                let _ = ShowWindow(self.hwnd, SW_HIDE);
            }
            self.rest_capture();
            self.release_surface();
            self.edge_idle_since = Some(now);
        }
        // Keep the displacement map for a quick return. Reclaim the remaining
        // renderer/driver caches only after a genuine idle interval.
        if self.state.phase == Phase::Hidden
            && !self.visible
            && self
                .edge_idle_since
                .is_some_and(|since| now.duration_since(since) >= EDGE_TRIM_AFTER)
        {
            self.idle_graphics();
            self.edge_idle_since = None;
        }
        unsafe {
            if let Some(host) = self.desktop {
                if !IsWindow(Some(host)).as_bool() {
                    let _ = platform::set_desktop(self.hwnd, None);
                    self.desktop = None;
                    self.model.settings.mode = Mode::Floating;
                    self.state.mode(Mode::Floating);
                    self.error =
                        Some("资源管理器已重启；已恢复悬浮窗，可从菜单重新固定到桌面".into());
                    self.show(false);
                    self.save();
                } else if self.visible && !self.state.blocked {
                    platform::sink_to_desktop(self.hwnd, host);
                }
            }
        }
        if self.visible {
            self.arrange_background();
        }
        self.schedule_tick();
    }
    fn arrange_background(&mut self) {
        if (!self.visible && !self.edge_warming())
            || !self.material.glass()
            || self.session_locked
            || self.suspended
            || self.minimized
        {
            self.rest_capture();
            return;
        }
        let mut r = RECT::default();
        unsafe {
            let _ = GetWindowRect(self.hwnd, &mut r);
        }
        let mut area = RECT {
            left: r.left - 16,
            top: r.top - 16,
            right: r.right + 16,
            bottom: r.bottom + 16,
        };
        if self.model.settings.mode == Mode::Edge {
            let work = platform::primary().rcWork;
            let width = (r.right - r.left).max(1);
            let height = (r.bottom - r.top).max(1);
            let left = if self.model.settings.side == Side::Right {
                work.right - width - 12
            } else {
                work.left + 12
            };
            let top = work.top + ((work.bottom - work.top - height) / 2).max(12);
            area = RECT {
                left: left - 16,
                top: top - 16,
                right: left + width + 16,
                bottom: top + height + 16,
            };
        }
        if self.scene.is_none() {
            let hwnd = self.hwnd.0 as usize;
            self.scene = Scene::new(
                area,
                Arc::new(move || unsafe {
                    let _ = PostMessageW(Some(HWND(hwnd as *mut _)), FRAME, WPARAM(0), LPARAM(0));
                }),
                &self.gfx,
            );
        }
        if let Some(scene) = &mut self.scene {
            let over = self.hwnd;
            if scene.arrange(area, over) || scene.fresh() {
                scene.want();
            }
            if let Some(capture) = scene.ready() {
                match capture.bitmap_into(
                    &self.gfx.dc,
                    self.scale,
                    self.background.as_ref().map(|b| &b.0),
                ) {
                    Ok(bitmap) => {
                        self.background = Some((bitmap, capture.rect));
                        self.dirty = true;
                    }
                    Err(e) => self.error = Some(format!("背景纹理不可用：{e}")),
                }
            }
        }
    }
    fn draw(&mut self) -> Result<()> {
        if (!self.visible && !self.edge_warming())
            || self.session_locked
            || self.suspended
            || self.minimized
        {
            return Ok(());
        }
        let s = self.scale;
        let (w, h) = (self.size.0 as f32 / s, self.size.1 as f32 / s);
        let shift = if self.model.settings.titlebar {
            0.0
        } else {
            -44.0
        };
        if self.lens.as_ref().is_none_or(|l| !l.matches(self.size, s)) {
            self.lens = Some(Lens::new(&self.gfx, self.size, s)?);
        }
        let dark = match self.model.settings.theme {
            Theme::System => platform::system_dark(),
            Theme::Dark => true,
            Theme::Light => false,
        };
        let theme_changed = PAINT_DARK.with(|d| d.replace(dark) != dark);
        if theme_changed {
            unsafe {
                let _ = SetLayeredWindowAttributes(
                    self.input,
                    edit_key(dark),
                    255,
                    LWA_ALPHA | LWA_COLORKEY,
                );
                let _ = InvalidateRect(Some(self.input), None, true);
                let _ = UpdateWindow(self.input);
                if !self.inline.is_invalid() {
                    let _ = SetLayeredWindowAttributes(
                        self.inline,
                        edit_key(dark),
                        255,
                        LWA_ALPHA | LWA_COLORKEY,
                    );
                    let _ = InvalidateRect(Some(self.inline), None, true);
                    let _ = UpdateWindow(self.inline);
                }
            }
        }
        let ink = gfx::color(if dark { 0xf4f4f5 } else { 0x202630 }, 1.0);
        let muted = gfx::color(if dark { 0xaeb7c2 } else { 0x596474 }, 1.0);
        let accent = gfx::color(0x378cf5, 1.0);
        let mut bounds = RECT::default();
        unsafe {
            let _ = GetWindowRect(self.hwnd, &mut bounds);
        }
        let background = self.background.as_ref();
        let mut rows = Vec::new();
        let mut y = 140.0 + shift - self.scroll;
        for todo in &self.model.todos {
            let layout = self.gfx.layout(&todo.text, 15.0, w - 124.0)?;
            let mut metrics = DWRITE_TEXT_METRICS::default();
            let mut line_count = 0;
            let mut lines;
            unsafe {
                layout.GetMetrics(&mut metrics)?;
                lines = vec![DWRITE_LINE_METRICS::default(); metrics.lineCount.max(1) as usize];
                layout.GetLineMetrics(Some(&mut lines), &mut line_count)?;
                layout.SetStrikethrough(
                    todo.done,
                    DWRITE_TEXT_RANGE {
                        startPosition: 0,
                        length: todo.text.encode_utf16().count() as u32,
                    },
                )?;
            }
            let height = (metrics.height + 24.0).max(50.0);
            rows.push(Row {
                id: todo.id.clone(),
                top: y,
                height,
                layout,
                text_height: metrics.height,
                baseline: lines[0].baseline,
            });
            y += height;
        }
        let limit = (y + self.scroll - h + 55.0).max(0.0);
        if self.scroll > limit {
            self.scroll = limit;
            self.dirty = true;
        }
        self.rows = rows;
        if self.surface.is_none() {
            self.surface = Some(Surface::new(&self.gfx, self.hwnd)?);
        }
        self.surface
            .as_mut()
            .unwrap()
            .draw(&self.gfx, self.size, s, |dc| unsafe {
                for step in (0..8).rev() {
                    let pad = step as f32;
                    gfx::fill(
                        dc,
                        gfx::rect(
                            10.0 - pad,
                            14.0 - pad,
                            w - 20.0 + 2.0 * pad,
                            h - 25.0 + 2.0 * pad,
                        ),
                        24.0 + pad,
                        gfx::color(0, 0.018),
                    )?;
                }
                let rr = D2D1_ROUNDED_RECT {
                    rect: gfx::rect(10.0, 10.0, w - 20.0, h - 20.0),
                    radiusX: 24.0,
                    radiusY: 24.0,
                };
                let pane_clip = gfx::round_clip(&self.gfx.factory, dc, &rr)?;
                if let Some((bitmap, area)) = background {
                    self.lens.as_ref().unwrap().draw(
                        dc,
                        bitmap,
                        (
                            (area.left - bounds.left) as f32 / s,
                            (area.top - bounds.top) as f32 / s,
                        ),
                        6.0,
                    )?;
                }
                gfx::fill(
                    dc,
                    gfx::rect(0.0, 0.0, w, h),
                    24.0,
                    gfx::color(
                        if dark { 0x171e2a } else { 0xf1f5fa },
                        if background.is_some() {
                            self.model.settings.opacity as f32 / 100.0
                        } else {
                            0.98
                        },
                    ),
                )?;
                if self.model.settings.titlebar {
                    gfx::text(dc, &self.gfx.layout("轻待", 21.0, 100.0)?, 24.0, 18.0, ink)?;
                    gfx::text(
                        dc,
                        &self.gfx.layout("LightDo", 10.5, 100.0)?,
                        77.0,
                        28.0,
                        muted,
                    )?;
                    let pin_active = match self.model.settings.mode {
                        Mode::Edge => self.model.settings.pinned,
                        Mode::Floating => self.model.settings.topmost,
                        Mode::Desktop => false,
                    };
                    for (index, label) in ["⋯", if pin_active { "◆" } else { "◇" }, "×"]
                        .into_iter()
                        .enumerate()
                    {
                        let button = header_rect(w, index);
                        let enabled = index != 1 || self.model.settings.mode != Mode::Desktop;
                        if enabled
                            && (self.header_hover == Some(index) || (index == 1 && pin_active))
                        {
                            gfx::fill(
                                dc,
                                button,
                                9.0,
                                gfx::color(
                                    0x378cf5,
                                    if GetKeyState(VK_LBUTTON.0 as i32) < 0 {
                                        0.24
                                    } else {
                                        0.12
                                    },
                                ),
                            )?;
                        }
                        let color = if !enabled {
                            gfx::color(if dark { 0xaeb7c2 } else { 0x596474 }, 0.35)
                        } else if index == 1 && pin_active {
                            accent
                        } else {
                            muted
                        };
                        gfx::centered_text(dc, &self.gfx, label, 20.0, button, color)?;
                    }
                }
                gfx::fill(
                    dc,
                    gfx::rect(20.0, INPUT_TOP + shift, w - 40.0, INPUT_HEIGHT),
                    12.0,
                    gfx::color(0xffffff, if dark { 0.10 } else { 0.65 }),
                )?;
                let field = D2D1_ROUNDED_RECT {
                    rect: gfx::rect(20.0, INPUT_TOP + shift, w - 40.0, INPUT_HEIGHT),
                    radiusX: 12.0,
                    radiusY: 12.0,
                };
                let focused = GetFocus() == self.input && GetForegroundWindow() == self.hwnd;
                let border = dc.CreateSolidColorBrush(
                    &gfx::color(
                        if focused {
                            0x378cf5
                        } else if dark {
                            0xffffff
                        } else {
                            0x7186a0
                        },
                        if focused { 0.75 } else { 0.18 },
                    ),
                    None,
                )?;
                dc.DrawRoundedRectangle(&field, &border, if focused { 1.4 } else { 1.0 }, None);
                let add = self.gfx.layout("+", 23.0, 32.0)?;
                add.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER)?;
                let mut add_metrics = DWRITE_TEXT_METRICS::default();
                add.GetMetrics(&mut add_metrics)?;
                gfx::text(
                    dc,
                    &add,
                    w - 60.0,
                    INPUT_TOP + shift + (INPUT_HEIGHT - add_metrics.height) / 2.0,
                    accent,
                )?;
                let list_clip = gfx::axis_clip(
                    dc,
                    gfx::rect(16.0, 133.0 + shift, w - 32.0, h - 183.0 - shift),
                );
                if self.model.todos.is_empty() {
                    gfx::text(
                        dc,
                        &self.gfx.layout("在上方写下第一件事", 14.0, w - 64.0)?,
                        32.0,
                        158.0 + shift,
                        muted,
                    )?;
                }
                for (row, todo) in self.rows.iter().zip(&self.model.todos) {
                    if row.top + row.height < 133.0 + shift || row.top > h - 50.0 {
                        continue;
                    }
                    if self.hover.as_ref() == Some(&todo.id)
                        || self.selected.as_ref() == Some(&todo.id)
                    {
                        gfx::fill(
                            dc,
                            gfx::rect(18.0, row.top, w - 36.0, row.height - 4.0),
                            9.0,
                            gfx::color(if dark { 0xffffff } else { 0x254b75 }, 0.07),
                        )?;
                    }
                    let check = D2D1_ELLIPSE {
                        point: Vector2 {
                            X: 35.0,
                            Y: row.center(),
                        },
                        radiusX: 8.0,
                        radiusY: 8.0,
                    };
                    let brush =
                        dc.CreateSolidColorBrush(&if todo.done { accent } else { muted }, None)?;
                    if todo.done {
                        dc.FillEllipse(&check, &brush);
                        gfx::centered_text(
                            dc,
                            &self.gfx,
                            "✓",
                            12.0,
                            gfx::rect(25.0, row.center() - 10.0, 20.0, 20.0),
                            gfx::color(0xffffff, 1.0),
                        )?;
                    } else {
                        dc.DrawEllipse(&check, &brush, 1.2, None)
                    }
                    if self.editing.as_ref() != Some(&todo.id) {
                        gfx::text(
                            dc,
                            &row.layout,
                            58.0,
                            row.text_top(s),
                            if todo.done { muted } else { ink },
                        )?;
                    }
                    if self.hover.as_ref() == Some(&todo.id) {
                        gfx::centered_text(
                            dc,
                            &self.gfx,
                            "×",
                            18.0,
                            gfx::rect(w - 59.0, row.center() - 16.0, 32.0, 32.0),
                            muted,
                        )?;
                    }
                    gfx::line(
                        dc,
                        (58.0, row.top + row.height - 4.0),
                        (w - 28.0, row.top + row.height - 4.0),
                        gfx::color(if dark { 0xffffff } else { 0x000000 }, 0.06),
                        0.6,
                    )?;
                }
                drop(list_clip);
                let status = self
                    .snap_preview
                    .map(|side| {
                        if side == Side::Left {
                            "松手停靠主屏左侧"
                        } else {
                            "松手停靠主屏右侧"
                        }
                    })
                    .or(self.error.as_deref())
                    .unwrap_or(if !self.material.glass() {
                        self.material.reason()
                    } else if background.is_none() {
                        "玻璃降级 · 正在等待真实背景"
                    } else if !self.model.settings.introduced {
                        "推向主屏右边缘呼出 · Ctrl+Alt+L"
                    } else {
                        // The current mode is already visible in the menu;
                        // leave the lower-left status area quiet.
                        ""
                    });
                if !status.is_empty() {
                    gfx::text(
                        dc,
                        &self.gfx.layout(status, 10.5, w - 48.0)?,
                        24.0,
                        h - 35.0,
                        muted,
                    )?;
                }
                drop(pane_clip);
                let stops = [
                    D2D1_GRADIENT_STOP {
                        position: 0.0,
                        color: gfx::color(0xffffff, 0.8),
                    },
                    D2D1_GRADIENT_STOP {
                        position: 0.3,
                        color: gfx::color(0xffffff, 0.12),
                    },
                    D2D1_GRADIENT_STOP {
                        position: 0.7,
                        color: gfx::color(0xffffff, 0.04),
                    },
                    D2D1_GRADIENT_STOP {
                        position: 1.0,
                        color: gfx::color(0xffffff, 0.35),
                    },
                ];
                let collection = ID2D1RenderTarget::CreateGradientStopCollection(
                    dc,
                    &stops,
                    D2D1_GAMMA_2_2,
                    D2D1_EXTEND_MODE_CLAMP,
                )?;
                let rim = dc.CreateLinearGradientBrush(
                    &D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                        startPoint: Vector2 { X: 10.0, Y: 10.0 },
                        endPoint: Vector2 {
                            X: w - 10.0,
                            Y: h - 10.0,
                        },
                    },
                    None,
                    &collection,
                )?;
                dc.DrawRoundedRectangle(&rr, &rim, 1.0, None);
                Ok(())
            })?;
        self.layout_editors();
        self.dirty = false;
        Ok(())
    }
    fn menu(&mut self) {
        self.in_menu = true;
        let id = crate::menu_ui::show(
            if self.visible {
                self.hwnd
            } else {
                self.controller
            },
            &self.model.settings,
            self.visible,
            self.scale,
        );
        self.in_menu = false;
        if id != 0 {
            self.command(id);
        }
    }
    fn command(&mut self, id: usize) {
        match id {
            201 => self.mode(Mode::Edge),
            202 => self.mode(Mode::Floating),
            203 => self.mode(Mode::Desktop),
            204 | 205 if self.model.settings.mode == Mode::Edge => {
                self.model.settings.side = if id == 204 { Side::Left } else { Side::Right };
                self.reposition();
            }
            206 if self.model.settings.mode == Mode::Edge => {
                self.model.settings.pinned = !self.model.settings.pinned
            }
            207 if self.model.settings.mode == Mode::Floating => {
                self.model.settings.topmost = !self.model.settings.topmost;
                unsafe {
                    let _ = SetWindowPos(
                        self.hwnd,
                        Some(if self.model.settings.topmost {
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
            208 => {
                self.model.settings.locked = !self.model.settings.locked;
                LOCKED.with(|l| l.set(self.model.settings.locked));
            }
            209 => self.model.settings.theme = Theme::Light,
            210 => self.model.settings.theme = Theme::Dark,
            211 => self.model.settings.theme = Theme::System,
            212 => {
                self.model.settings.opacity = self.model.settings.opacity.saturating_sub(10).max(10)
            }
            213 => self.model.settings.opacity = (self.model.settings.opacity + 10).min(90),
            214 => self.open_settings(),
            215 => {
                self.model.settings.x = platform::primary().rcWork.left + 80;
                self.model.settings.y = platform::primary().rcWork.top + 80;
                self.show(false);
            }
            216 => {
                let wanted = !self.model.settings.autostart;
                match platform::autostart(wanted) {
                    Ok(()) => self.model.settings.autostart = wanted,
                    Err(e) => self.error = Some(e),
                }
            }
            218 => {
                if self.editing.is_some() {
                    if COMPOSING.with(Cell::get) {
                        unsafe {
                            let context = ImmGetContext(self.inline);
                            if !context.is_invalid() {
                                let _ = ImmNotifyIME(context, NI_COMPOSITIONSTR, CPS_COMPLETE, 0);
                                let _ = ImmReleaseContext(self.inline, context);
                            }
                        }
                    }
                    if COMPOSING.with(Cell::get) {
                        self.message(
                            "请先确认或取消输入法候选词，再退出轻待；正在编辑的文字保持不变。",
                        );
                        return;
                    }
                    self.commit(self.inline, false);
                }
                if !self.store.writable || self.store.save(&self.model).is_ok() {
                    self.quit = true
                } else {
                    self.message("退出前保存失败，请修复存储问题后重试。");
                }
            }
            219 => {
                if self.visible {
                    self.hide_tray();
                } else {
                    self.show(true);
                }
            }
            220 => {
                self.model.settings.titlebar = !self.model.settings.titlebar;
                self.layout_editors();
            }
            221 if self.model.settings.mode == Mode::Edge => {
                self.model.settings.auto_hide = !self.model.settings.auto_hide
            }
            _ => {}
        }
        settings_ui::sync_layer();
        self.save();
    }
    fn event(&mut self, e: Event) {
        match e.msg {
            WM_LBUTTONDOWN => {
                let x = (e.l.0 as i16) as f32 / self.scale;
                let y = ((e.l.0 >> 16) as i16) as f32 / self.scale;
                // Color-keyed edit backgrounds let the glass show through.
                // Route clicks in transparent gaps to the real system editor
                // so native caret positioning and mouse selection still work.
                for edit in [self.inline, self.input] {
                    unsafe {
                        if edit.is_invalid() || !IsWindowVisible(edit).as_bool() {
                            continue;
                        }
                        let mut point = POINT::default();
                        let mut rect = RECT::default();
                        let _ = GetCursorPos(&mut point);
                        let _ = GetWindowRect(edit, &mut rect);
                        if point.x >= rect.left
                            && point.x < rect.right
                            && point.y >= rect.top
                            && point.y < rect.bottom
                        {
                            let local = LPARAM(
                                ((point.x - rect.left) & 0xffff) as isize
                                    | ((((point.y - rect.top) & 0xffff) as isize) << 16),
                            );
                            let _ = SetFocus(Some(edit));
                            SendMessageW(edit, WM_LBUTTONDOWN, Some(e.w), Some(local));
                            return;
                        }
                    }
                }
                if self.header_hit(x, y).is_some() {
                    self.dirty = true;
                }
                self.begin_drag(x, y);
            }
            WM_CAPTURECHANGED => self.end_drag(false),
            SHOW_REQUEST => self.show(true),
            WM_POWERBROADCAST => {
                if e.w.0 == 4 {
                    self.suspended = true;
                } else if e.w.0 == 7 || e.w.0 == 18 {
                    self.suspended = false;
                }
                self.scene = None;
                self.background = None;
                self.lens = None;
                if self.suspended {
                    self.idle_graphics();
                }
                self.reposition();
                self.dirty = true;
                self.last_tick = Instant::now();
                self.schedule_tick();
            }
            WM_WTSSESSION_CHANGE => {
                if e.w.0 == WTS_SESSION_LOCK as usize {
                    self.session_locked = true;
                    self.idle_graphics();
                } else if e.w.0 == WTS_SESSION_UNLOCK as usize {
                    self.session_locked = false;
                    self.scene = None;
                    self.background = None;
                    self.reposition();
                    self.dirty = true;
                }
                self.last_tick = Instant::now();
                self.schedule_tick();
            }
            settings_ui::CHANGED => {
                if let Some(settings) = settings_ui::take() {
                    self.apply_settings(settings);
                }
            }
            KEY_NAV => {
                if let Some(first) = self.model.todos.first() {
                    self.selected = Some(first.id.clone());
                    unsafe {
                        let _ = SetFocus(Some(self.hwnd));
                    }
                    self.dirty = true;
                }
            }
            WM_KEYDOWN => {
                let key = e.w.0 as u16;
                if key == VK_F10.0 {
                    self.menu();
                } else if key == VK_ESCAPE.0 {
                    self.cancel(self.hwnd);
                } else if key == VK_TAB.0 {
                    self.activate_input();
                } else if key == VK_UP.0 || key == VK_DOWN.0 {
                    if !self.model.todos.is_empty() {
                        let current = self
                            .selected
                            .as_ref()
                            .and_then(|id| self.model.todos.iter().position(|t| &t.id == id))
                            .unwrap_or(0);
                        let next = if key == VK_UP.0 {
                            current.saturating_sub(1)
                        } else {
                            (current + 1).min(self.model.todos.len() - 1)
                        };
                        self.selected = Some(self.model.todos[next].id.clone());
                        if let Some(row) = self.rows.get(next) {
                            let h = self.size.1 as f32 / self.scale;
                            if row.top < 140.0 {
                                self.scroll = (self.scroll + row.top - 140.0).max(0.0);
                            } else if row.top + row.height > h - 50.0 {
                                self.scroll += row.top + row.height - h + 50.0;
                            }
                        }
                        self.dirty = true;
                    }
                } else if let Some(id) = self.selected.clone() {
                    if key == VK_RETURN.0 {
                        self.start_edit(id);
                    } else if key == VK_SPACE.0 {
                        self.model.toggle(&id);
                        self.save();
                    } else if key == VK_DELETE.0 {
                        self.model.delete(&id);
                        self.selected = None;
                        self.save();
                    }
                }
            }
            WM_CLOSE => self.hide_tray(),
            WM_DESTROY => {
                if e.hwnd == self.hwnd && self.desktop.is_some() {
                    if let Err(error) = self.restore_panel() {
                        self.message(&format!(
                            "桌面窗口恢复失败：{error}。任务文件已保留，请重新启动轻待。"
                        ));
                        self.quit = true;
                    }
                } else if e.hwnd == self.hwnd {
                    self.quit = true;
                }
            }
            WM_TIMER => self.tick(),
            FRAME => {
                self.arrange_background();
                self.dirty = true;
            }
            COMMIT => self.commit(HWND(e.w.0 as *mut _), e.l.0 != 0),
            CANCEL => self.cancel(HWND(e.w.0 as *mut _)),
            WM_SIZE => {
                if e.hwnd != self.hwnd {
                    return;
                }
                if e.w.0 == SIZE_MINIMIZED as usize {
                    self.minimized = true;
                    self.idle_graphics();
                    self.schedule_tick();
                    return;
                }
                self.minimized = false;
                let mut r = RECT::default();
                unsafe {
                    let _ = GetClientRect(self.hwnd, &mut r);
                }
                self.size = (
                    (r.right - r.left).max(1) as u32,
                    (r.bottom - r.top).max(1) as u32,
                );
                if self.moving {
                    self.model.settings.width = (self.size.0 as f32 / self.scale) as i32;
                    self.model.settings.height = (self.size.1 as f32 / self.scale) as i32;
                }
                self.dirty = true;
                self.layout_editors();
                self.schedule_tick();
            }
            WM_MOVE => {
                self.dirty = true;
            }
            WM_ENTERSIZEMOVE => {
                self.moving = true;
                if self.model.settings.mode == Mode::Edge {
                    self.mode(Mode::Floating);
                }
            }
            WM_EXITSIZEMOVE => {
                self.moving = false;
                let mut r = RECT::default();
                unsafe {
                    let _ = GetWindowRect(self.hwnd, &mut r);
                }
                self.model.settings.x = r.left;
                self.model.settings.y = r.top;
                let work = platform::primary().rcWork;
                if (r.left - work.left).abs() < 28 {
                    self.model.settings.side = Side::Left;
                    self.mode(Mode::Edge)
                } else if (r.right - work.right).abs() < 28 {
                    self.model.settings.side = Side::Right;
                    self.mode(Mode::Edge)
                }
                self.save();
            }
            WM_DPICHANGED => {
                self.scale = (e.w.0 & 0xffff) as f32 / 96.0;
                SCALE.with(|s| s.set(self.scale));
                self.update_font();
                self.lens = None;
                self.rest_capture();
                self.reposition();
                self.dirty = true;
            }
            WM_DISPLAYCHANGE => {
                self.rest_capture();
                self.reposition();
            }
            WM_SETTINGCHANGE => {
                self.material = Material::detect(self.gfx.software);
                self.rest_capture();
                self.dirty = true;
            }
            WM_HOTKEY => {
                if self.visible && !self.state.blocked && self.model.settings.mode == Mode::Edge {
                    self.state.dismiss()
                } else {
                    self.show(true)
                }
            }
            WM_RBUTTONUP => self.menu(),
            TRAY => match e.l.0 as u32 {
                WM_LBUTTONUP => self.show(true),
                WM_RBUTTONUP => self.menu(),
                _ => {}
            },
            WM_LBUTTONUP | WM_LBUTTONDBLCLK => {
                if self.drag.is_some() {
                    self.end_drag(true);
                    return;
                }
                let x = (e.l.0 as i16) as f32 / self.scale;
                let y = ((e.l.0 >> 16) as i16) as f32 / self.scale;
                let w = self.size.0 as f32 / self.scale;
                let shift = if self.model.settings.titlebar {
                    0.0
                } else {
                    -44.0
                };
                if self.model.settings.titlebar && y < 60.0 {
                    match self.header_hit(x, y) {
                        Some(2) => self.hide_tray(),
                        Some(1) => match self.model.settings.mode {
                            Mode::Edge => self.command(206),
                            Mode::Floating => self.command(207),
                            Mode::Desktop => {}
                        },
                        Some(0) => self.menu(),
                        _ => {}
                    }
                } else if (INPUT_TOP + shift..INPUT_TOP + INPUT_HEIGHT + shift).contains(&y) {
                    if x > w - 66.0 {
                        self.commit(self.input, false)
                    } else {
                        self.activate_input();
                        self.dirty = true;
                    }
                } else if let Some(id) = self
                    .rows
                    .iter()
                    .find(|r| {
                        y >= r.top
                            && y < r.top + r.height
                            && y < self.size.1 as f32 / self.scale - 50.0
                    })
                    .map(|r| r.id.clone())
                {
                    if x < 52.0 {
                        self.model.toggle(&id);
                        self.save()
                    } else if x > w - 58.0 {
                        self.model.delete(&id);
                        self.save()
                    } else if e.msg == WM_LBUTTONDBLCLK {
                        self.start_edit(id)
                    } else {
                        self.selected = Some(id);
                        unsafe {
                            let _ = SetFocus(Some(self.hwnd));
                        }
                        self.dirty = true;
                    }
                }
            }
            WM_MOUSEMOVE => {
                if self.drag.is_some() {
                    self.update_drag();
                    return;
                }
                let x = (e.l.0 as i16) as f32 / self.scale;
                let y = ((e.l.0 >> 16) as i16) as f32 / self.scale;
                let header = self.header_hit(x, y);
                if header != self.header_hover {
                    self.header_hover = header;
                    self.dirty = true;
                    let mut tracking = TRACKMOUSEEVENT {
                        cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                        dwFlags: TME_LEAVE,
                        hwndTrack: self.hwnd,
                        ..Default::default()
                    };
                    unsafe {
                        let _ = TrackMouseEvent(&mut tracking);
                    }
                }
                let hovered = self
                    .rows
                    .iter()
                    .find(|r| y >= r.top && y < r.top + r.height)
                    .map(|r| r.id.clone());
                if hovered != self.hover {
                    self.hover = hovered;
                    self.dirty = true;
                }
            }
            WM_MOUSELEAVE => {
                self.header_hover = None;
                self.hover = None;
                self.dirty = true;
            }
            WM_MOUSEWHEEL => {
                if self.editing.is_none() {
                    self.scroll =
                        (self.scroll - ((e.w.0 >> 16) as i16) as f32 / 120.0 * 56.0).max(0.0);
                    self.dirty = true;
                }
            }
            WM_COMMAND if (201..=221).contains(&(e.w.0 & 0xffff)) => self.command(e.w.0 & 0xffff),
            _ => {
                if e.msg == self.taskbar_created {
                    self.tray(true);
                }
            }
        }
    }
    fn tray(&self, add: bool) {
        unsafe {
            let mut info = NOTIFYICONDATAW {
                cbSize: size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.controller,
                uID: 1,
                uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
                uCallbackMessage: TRAY,
                hIcon: self.icon,
                ..Default::default()
            };
            let tip: Vec<u16> = "轻待 LightDo".encode_utf16().collect();
            info.szTip[..tip.len()].copy_from_slice(&tip);
            let _ = Shell_NotifyIconW(if add { NIM_ADD } else { NIM_DELETE }, &info);
        }
    }
    fn tray_hint(&self) {
        unsafe {
            let mut info = NOTIFYICONDATAW {
                cbSize: size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.controller,
                uID: 1,
                uFlags: NIF_INFO,
                dwInfoFlags: NIIF_INFO,
                ..Default::default()
            };
            let title: Vec<u16> = "轻待已隐藏到托盘".encode_utf16().collect();
            let text: Vec<u16> = "点击托盘图标或使用呼出快捷键找回；右键托盘可退出。"
                .encode_utf16()
                .collect();
            info.szInfoTitle[..title.len()].copy_from_slice(&title);
            info.szInfo[..text.len()].copy_from_slice(&text);
            let _ = Shell_NotifyIconW(NIM_MODIFY, &info);
        }
    }
}
pub fn run() -> std::result::Result<(), String> {
    unsafe { run_inner().map_err(|e| e.to_string()) }
}
unsafe fn make_panel() -> Result<HWND> {
    unsafe {
        let hwnd = CreateWindowExW(
            WS_EX_APPWINDOW | WS_EX_NOREDIRECTIONBITMAP,
            w!("LightDo.Panel"),
            w!("轻待 LightDo"),
            WS_POPUP | WS_CLIPCHILDREN,
            100,
            100,
            400,
            600,
            None,
            None,
            Some(GetModuleHandleW(None)?.into()),
            None,
        )?;
        PANEL.with(|p| p.set(hwnd));
        LAST_FOCUS.with(|f| f.set(HWND::default()));
        Ok(hwnd)
    }
}
unsafe fn run_inner() -> Result<()> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let mutex = CreateMutexW(None, false, w!("Local\\LightDo.SingleInstance"))?;
        if GetLastError() == ERROR_ALREADY_EXISTS {
            if let Ok(hwnd) = FindWindowW(w!("LightDo.Controller"), None) {
                let shutdown = std::env::args().any(|arg| arg == "--shutdown");
                if !shutdown {
                    let mut process_id = 0;
                    GetWindowThreadProcessId(hwnd, Some(&mut process_id));
                    let _ = AllowSetForegroundWindow(process_id);
                }
                let _ = PostMessageW(
                    Some(hwnd),
                    if shutdown { WM_COMMAND } else { SHOW_REQUEST },
                    WPARAM(if shutdown { 218 } else { 0 }),
                    LPARAM(0),
                );
            } else if let Ok(hwnd) = FindWindowW(w!("LightDo.Panel"), None) {
                let _ = PostMessageW(Some(hwnd), WM_HOTKEY, WPARAM(1), LPARAM(0));
            }
            let _ = CloseHandle(mutex);
            return Ok(());
        }
        if std::env::args().any(|arg| arg == "--shutdown") {
            let _ = CloseHandle(mutex);
            return Ok(());
        }
        let instance = GetModuleHandleW(None)?;
        let icon = LoadIconW(
            Some(instance.into()),
            windows::core::PCWSTR(1usize as *const u16),
        )
        .unwrap_or_else(|_| LoadIconW(None, IDI_APPLICATION).unwrap_or_default());
        let class = WNDCLASSW {
            style: CS_DBLCLKS,
            lpfnWndProc: Some(procedure),
            hInstance: instance.into(),
            lpszClassName: w!("LightDo.Panel"),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hIcon: icon,
            ..Default::default()
        };
        RegisterClassW(&class);
        RegisterClassW(&WNDCLASSW {
            lpfnWndProc: Some(procedure),
            hInstance: instance.into(),
            lpszClassName: w!("LightDo.Controller"),
            ..Default::default()
        });
        let controller = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("LightDo.Controller"),
            w!(""),
            WS_POPUP,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(instance.into()),
            None,
        )?;
        let hwnd = make_panel()?;
        MAIN.with(|m| m.set(controller));
        let path =
            std::path::PathBuf::from(std::env::var_os("LIGHTDO_DATA_DIR").unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("APPDATA").unwrap_or_default())
                    .join("LightDo")
                    .into_os_string()
            }))
            .join("state.json");
        let (store, loaded) = Store::open(path);
        let (mut model, error) = match loaded {
            Ok(model) => (model, None),
            Err(error) => (Model::default(), Some(error)),
        };
        model.settings.autostart = platform::autostart_enabled();
        let scale = GetDpiForWindow(hwnd) as f32 / 96.0;
        SCALE.with(|s| s.set(scale));
        LOCKED.with(|l| l.set(model.settings.locked));
        let font = CreateFontW(
            -(15.0 * scale) as i32,
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
            ANTIALIASED_QUALITY,
            DEFAULT_PITCH.0 as u32,
            w!("Microsoft YaHei UI"),
        );
        let gfx = Rc::new(Gfx::new()?);
        let (font_line_px, font_ascent_px) = font_metrics(font, scale);
        let surface = Surface::new(&gfx, hwnd)?;
        let material = Material::detect(gfx.software);
        let mut state = WindowState::default();
        state.mode(model.settings.mode);
        let initial_mode = model.settings.mode;
        let mut app = App {
            controller,
            hwnd,
            input: HWND::default(),
            inline: HWND::default(),
            editing: None,
            store,
            model,
            gfx,
            material,
            surface: Some(surface),
            scene: None,
            lens: None,
            background: None,
            state,
            start: Instant::now(),
            last_tick: Instant::now(),
            scale,
            size: ((400.0 * scale) as u32, (600.0 * scale) as u32),
            rows: Vec::new(),
            scroll: 0.0,
            hover: None,
            header_hover: None,
            selected: None,
            dirty: true,
            visible: true,
            minimized: false,
            in_menu: false,
            moving: false,
            drag: None,
            snap_preview: None,
            desktop: None,
            error,
            quit: false,
            font,
            font_line_px,
            font_ascent_px,
            taskbar_created: RegisterWindowMessageW(w!("TaskbarCreated")),
            icon,
            graphic_retry: None,
            session_locked: crate::diagnostics::inspect()["session"]["locked"]
                .as_bool()
                .unwrap_or(false),
            suspended: false,
            edge_idle_since: None,
            runtime_snapshot: Cell::new(0),
            timer_interval: Cell::new(None),
            edge_open_since: None,
            pending_activation: false,
            activation_grace_until: None,
        };
        app.input = app.editor(INPUT)?;
        SendMessageW(
            app.input,
            EM_SETCUEBANNER,
            Some(WPARAM(1)),
            Some(LPARAM(w!("写下一件事…").0 as isize)),
        );
        app.reposition();
        app.tray(true);
        if RegisterHotKey(
            Some(controller),
            1,
            HOT_KEY_MODIFIERS(app.model.settings.hotkey_mod) | MOD_NOREPEAT,
            app.model.settings.hotkey_vk,
        )
        .is_err()
        {
            app.error = Some("快捷键 Ctrl+Alt+L 已被其他程序占用，请从托盘找回".into());
        }
        SetTimer(Some(controller), 1, 80, None);
        let _ = WTSRegisterSessionNotification(controller, NOTIFY_FOR_THIS_SESSION);
        if initial_mode == Mode::Edge
            && app.model.settings.introduced
            && app.model.settings.auto_hide
            && !app.model.settings.pinned
        {
            app.state.hide();
            app.state.step(220.0, true);
            app.visible = false;
            app.edge_idle_since = Some(Instant::now());
            let _ = SetWindowPos(
                app.hwnd,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
            app.reposition();
            app.schedule_tick();
        } else {
            app.show(false);
        }
        if !app.store.writable {
            let _ = EnableWindow(app.input, false);
            app.message("无法读取轻待的数据文件。原文件保持不变，当前已禁止修改和覆盖保存。请检查 %APPDATA%\\LightDo\\state.json 后重启。可以从托盘正常退出。");
        }
        if initial_mode == Mode::Desktop {
            app.model.settings.mode = Mode::Floating;
            app.mode(Mode::Desktop);
        }
        while !app.quit {
            let mut msg = MSG::default();
            if GetMessageW(&mut msg, None, 0, 0).0 <= 0 {
                break;
            }
            let settings_shortcut = matches!(msg.message, WM_KEYDOWN | WM_SYSKEYDOWN)
                && msg.wParam.0 == VK_F10.0 as usize
                && GetKeyState(VK_CONTROL.0 as i32) < 0
                && GetKeyState(VK_MENU.0 as i32) >= 0
                && !COMPOSING.with(Cell::get)
                && (msg.hwnd == app.hwnd || GetParent(msg.hwnd).ok() == Some(app.hwnd));
            if settings_shortcut {
                app.open_settings();
            } else if !settings_ui::dialog_message(&msg) {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            loop {
                let event = EVENTS.with(|e| e.borrow_mut().pop_front());
                if let Some(event) = event {
                    app.event(event)
                } else {
                    break;
                }
            }
            if app.dirty {
                if let Err(e) = app.draw() {
                    app.recover_graphics(e);
                }
            }
        }
        settings_ui::close();
        app.tray(false);
        let _ = UnregisterHotKey(Some(controller), 1);
        let _ = KillTimer(Some(controller), 1);
        let _ = WTSUnRegisterSessionNotification(controller);
        app.scene = None;
        app.release_surface();
        let _ = DestroyWindow(app.hwnd);
        let _ = DestroyWindow(controller);
        let _ = DeleteObject(app.font.into());
        let _ = CloseHandle(mutex);
        Ok(())
    }
}

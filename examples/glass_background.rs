//! Controlled, non-private background for real LightDo glass verification.
//! Run with `cargo run --release --example glass_background` (not installed).
#![windows_subsystem = "windows"]
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::{LibraryLoader::GetModuleHandleW, SystemInformation::GetTickCount64},
        UI::{HiDpi::*, WindowsAndMessaging::*},
    },
    core::w,
};
unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let target = BeginPaint(hwnd, &mut ps);
                let mut r = RECT::default();
                let _ = GetClientRect(hwnd, &mut r);
                // Present a complete test frame. Directly painting the moving
                // grid exposes intermediate solid/partial frames to WGC.
                let dc = CreateCompatibleDC(Some(target));
                let bitmap = CreateCompatibleBitmap(target, r.right.max(1), r.bottom.max(1));
                let previous = SelectObject(dc, bitmap.into());
                let base = CreateSolidBrush(COLORREF(0xf5f0e8));
                FillRect(dc, &r, base);
                let _ = DeleteObject(base.into());
                let phase = (GetTickCount64() / 20 % 128) as i32;
                for col in -1..r.right / 64 + 1 {
                    for row in 0..r.bottom / 64 + 1 {
                        let tone = if (col + row) % 2 == 0 {
                            COLORREF(0xe5b85d)
                        } else {
                            COLORREF(0xdbecf8)
                        };
                        let brush = CreateSolidBrush(tone);
                        let tile = RECT {
                            left: col * 64 + phase / 2,
                            top: row * 64,
                            right: col * 64 + 64 + phase / 2,
                            bottom: row * 64 + 64,
                        };
                        FillRect(dc, &tile, brush);
                        let _ = DeleteObject(brush.into());
                    }
                }
                SetBkMode(dc, TRANSPARENT);
                SetTextColor(dc, COLORREF(0x60402b));
                let text: Vec<u16> = "LIGHTDO · LIVE BACKGROUND · 实时移动网格 · 无隐私测试"
                    .encode_utf16()
                    .collect();
                let _ = TextOutW(dc, 36, 36, &text);
                let _ = BitBlt(target, 0, 0, r.right, r.bottom, Some(dc), 0, 0, SRCCOPY);
                SelectObject(dc, previous);
                let _ = DeleteObject(bitmap.into());
                let _ = DeleteDC(dc);
                let _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_TIMER => {
                let _ = InvalidateRect(Some(hwnd), None, false);
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wp, lp),
        }
    }
}
fn main() {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let instance = GetModuleHandleW(None).unwrap();
        RegisterClassW(&WNDCLASSW {
            lpfnWndProc: Some(proc),
            hInstance: instance.into(),
            lpszClassName: w!("LightDo.TestBackground"),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
            ..Default::default()
        });
        let fullscreen = std::env::args().any(|arg| arg == "--fullscreen");
        let area = if fullscreen {
            lightdo::platform::primary().rcMonitor
        } else {
            RECT {
                left: 60,
                top: 60,
                right: 1160,
                bottom: 980,
            }
        };
        let hwnd = CreateWindowExW(
            if std::env::args().any(|arg| arg == "--topmost") {
                WS_EX_TOPMOST
            } else {
                WINDOW_EX_STYLE(0)
            },
            w!("LightDo.TestBackground"),
            w!("LightDo 无隐私玻璃测试背景"),
            (if fullscreen {
                WS_POPUP
            } else {
                WS_OVERLAPPEDWINDOW
            }) | WS_VISIBLE,
            area.left,
            area.top,
            area.right - area.left,
            area.bottom - area.top,
            None,
            None,
            Some(instance.into()),
            None,
        )
        .unwrap();
        if !fullscreen && std::env::args().any(|arg| arg == "--maximized") {
            let _ = ShowWindow(hwnd, SW_SHOWMAXIMIZED);
        }
        if !std::env::args().any(|arg| arg == "--static") {
            SetTimer(Some(hwnd), 1, 33, None);
        }
        let mut m = MSG::default();
        while GetMessageW(&mut m, None, 0, 0).0 > 0 {
            let _ = TranslateMessage(&m);
            DispatchMessageW(&m);
        }
    }
}

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
                let dc = BeginPaint(hwnd, &mut ps);
                let mut r = RECT::default();
                let _ = GetClientRect(hwnd, &mut r);
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
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("LightDo.TestBackground"),
            w!("LightDo 无隐私玻璃测试背景"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            60,
            60,
            1100,
            920,
            None,
            None,
            Some(instance.into()),
            None,
        )
        .unwrap();
        SetTimer(Some(hwnd), 1, 33, None);
        let mut m = MSG::default();
        while GetMessageW(&mut m, None, 0, 0).0 > 0 {
            let _ = TranslateMessage(&m);
            DispatchMessageW(&m);
        }
    }
}

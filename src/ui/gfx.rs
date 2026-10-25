//! Minimal LightDo Direct2D/DirectComposition device. Scene acquisition is
//! adapted from Glance; business UI and resource ownership are independent.
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::Graphics::{
    Direct2D::Common::*, Direct2D::*, Direct3D::*, Direct3D11::*, DirectComposition::*,
    DirectWrite::*, Dxgi::Common::*, Dxgi::*,
};
use windows::core::{Interface, Result, w};
use windows_numerics::{Matrix3x2, Vector2};

pub struct Gfx {
    pub guarded: bool,
    pub software: bool,
    d3d: ID3D11Device,
    pub factory: ID2D1Factory1,
    pub device: ID2D1Device,
    pub dcomp: IDCompositionDesktopDevice,
    pub dc: ID2D1DeviceContext,
    pub write: IDWriteFactory,
}
impl Gfx {
    pub fn new() -> Result<Self> {
        unsafe {
            let create = |kind| {
                let mut d = None;
                D3D11CreateDevice(
                    None,
                    kind,
                    Default::default(),
                    D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                    None,
                    D3D11_SDK_VERSION,
                    Some(&mut d),
                    None,
                    None,
                )?;
                Ok::<ID3D11Device, windows::core::Error>(d.unwrap())
            };
            let (d3d, software) = match create(D3D_DRIVER_TYPE_HARDWARE) {
                Ok(d) => (d, false),
                Err(_) => (create(D3D_DRIVER_TYPE_WARP)?, true),
            };
            let guard: ID3D11Multithread = d3d.GetImmediateContext()?.cast()?;
            let _ = guard.SetMultithreadProtected(true);
            let dxgi: IDXGIDevice = d3d.cast()?;
            let factory: ID2D1Factory1 = D2D1CreateFactory(D2D1_FACTORY_TYPE_MULTI_THREADED, None)?;
            let device = factory.CreateDevice(&dxgi)?;
            device.SetMaximumTextureMemory(32 << 20);
            let dc = device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;
            let dcomp = DCompositionCreateDevice2(&device)?;
            let write = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
            Ok(Self {
                guarded: guard.GetMultithreadProtected().as_bool(),
                software,
                d3d,
                factory,
                device,
                dcomp,
                dc,
                write,
            })
        }
    }
    pub fn d3d(&self) -> Result<ID3D11Device> {
        Ok(self.d3d.clone())
    }
    pub fn layout(&self, text: &str, size: f32, width: f32) -> Result<IDWriteTextLayout> {
        unsafe {
            let format = self.write.CreateTextFormat(
                w!("Microsoft YaHei UI"),
                None,
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                size,
                w!("zh-CN"),
            )?;
            format.SetWordWrapping(DWRITE_WORD_WRAPPING_WRAP)?;
            self.write.CreateTextLayout(
                &text.encode_utf16().collect::<Vec<_>>(),
                &format,
                width.max(1.0),
                100000.0,
            )
        }
    }
}
pub fn room(have: Option<(u32, u32)>, want: (u32, u32)) -> Option<(u32, u32)> {
    if have == Some(want) {
        None
    } else {
        Some((want.0.max(1), want.1.max(1)))
    }
}
#[cfg(test)]
pub fn held() -> Result<Gfx> {
    Gfx::new()
}

pub fn color(hex: u32, a: f32) -> D2D1_COLOR_F {
    D2D1_COLOR_F {
        r: ((hex >> 16) & 255) as f32 / 255.0,
        g: ((hex >> 8) & 255) as f32 / 255.0,
        b: (hex & 255) as f32 / 255.0,
        a,
    }
}
pub struct RoundClip<'a> {
    dc: &'a ID2D1DeviceContext,
    _mask: ID2D1Geometry,
}
impl Drop for RoundClip<'_> {
    fn drop(&mut self) {
        unsafe {
            self.dc.PopLayer();
        }
    }
}
pub fn round_clip<'a>(
    factory: &ID2D1Factory1,
    dc: &'a ID2D1DeviceContext,
    r: &D2D1_ROUNDED_RECT,
) -> Result<RoundClip<'a>> {
    unsafe {
        let mask: ID2D1Geometry = factory.CreateRoundedRectangleGeometry(r)?.cast()?;
        let mut params = D2D1_LAYER_PARAMETERS1 {
            contentBounds: r.rect,
            geometricMask: std::mem::ManuallyDrop::new(Some(mask.clone())),
            maskAntialiasMode: D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
            maskTransform: Matrix3x2::identity(),
            opacity: 1.0,
            ..Default::default()
        };
        dc.PushLayer(&params, None);
        std::mem::ManuallyDrop::drop(&mut params.geometricMask);
        Ok(RoundClip { dc, _mask: mask })
    }
}
pub struct AxisClip<'a>(&'a ID2D1DeviceContext);
impl Drop for AxisClip<'_> {
    fn drop(&mut self) {
        unsafe {
            self.0.PopAxisAlignedClip();
        }
    }
}
pub fn axis_clip(dc: &ID2D1DeviceContext, r: D2D_RECT_F) -> AxisClip<'_> {
    unsafe {
        dc.PushAxisAlignedClip(&r, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
    }
    AxisClip(dc)
}
pub fn rect(x: f32, y: f32, w: f32, h: f32) -> D2D_RECT_F {
    D2D_RECT_F {
        left: x,
        top: y,
        right: x + w,
        bottom: y + h,
    }
}
/// # Safety
/// `value` must have the representation and byte size required by the
/// selected Direct2D effect property.
pub unsafe fn property<T>(
    effect: &ID2D1Effect,
    index: u32,
    kind: D2D1_PROPERTY_TYPE,
    value: &T,
) -> Result<()> {
    unsafe {
        effect.SetValue(
            index,
            kind,
            std::slice::from_raw_parts(value as *const T as *const u8, size_of::<T>()),
        )
    }
}

pub struct Surface {
    _target: IDCompositionTarget,
    visual: IDCompositionVisual2,
    surface: Option<IDCompositionSurface>,
    size: (u32, u32),
}
impl Surface {
    pub fn new(gfx: &Gfx, hwnd: HWND) -> Result<Self> {
        unsafe {
            let target = gfx.dcomp.CreateTargetForHwnd(hwnd, false)?;
            let visual = gfx.dcomp.CreateVisual()?;
            target.SetRoot(&visual)?;
            Ok(Self {
                _target: target,
                visual,
                surface: None,
                size: (0, 0),
            })
        }
    }
    pub fn release(&mut self, gfx: &Gfx) {
        self.surface = None;
        unsafe {
            let _ = self.visual.SetContent(None);
            let _ = gfx.dcomp.Commit();
        }
    }
    pub fn draw(
        &mut self,
        gfx: &Gfx,
        size: (u32, u32),
        scale: f32,
        paint: impl FnOnce(&ID2D1DeviceContext) -> Result<()>,
    ) -> Result<()> {
        unsafe {
            if self.size != size || self.surface.is_none() {
                self.surface = Some(gfx.dcomp.CreateSurface(
                    size.0.max(1),
                    size.1.max(1),
                    DXGI_FORMAT_B8G8R8A8_UNORM,
                    DXGI_ALPHA_MODE_PREMULTIPLIED,
                )?);
                self.size = size;
            }
            let surface = self.surface.as_ref().unwrap();
            let mut offset = POINT::default();
            let dc: ID2D1DeviceContext = surface.BeginDraw(None, &mut offset)?;
            dc.SetDpi(96.0 * scale, 96.0 * scale);
            dc.SetTransform(&Matrix3x2::translation(
                offset.x as f32 / scale,
                offset.y as f32 / scale,
            ));
            dc.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
            dc.Clear(Some(&color(0, 0.0)));
            let result = paint(&dc);
            let end = surface.EndDraw();
            result?;
            end?;
            self.visual.SetContent(surface)?;
            gfx.dcomp.Commit()
        }
    }
}

pub fn fill(dc: &ID2D1DeviceContext, r: D2D_RECT_F, radius: f32, c: D2D1_COLOR_F) -> Result<()> {
    unsafe {
        let brush = dc.CreateSolidColorBrush(&c, None)?;
        dc.FillRoundedRectangle(
            &D2D1_ROUNDED_RECT {
                rect: r,
                radiusX: radius,
                radiusY: radius,
            },
            &brush,
        );
        Ok(())
    }
}
pub fn text(
    dc: &ID2D1DeviceContext,
    layout: &IDWriteTextLayout,
    x: f32,
    y: f32,
    c: D2D1_COLOR_F,
) -> Result<()> {
    unsafe {
        let brush = dc.CreateSolidColorBrush(&c, None)?;
        dc.DrawTextLayout(
            Vector2 { X: x, Y: y },
            layout,
            &brush,
            D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
        );
        Ok(())
    }
}
pub fn line(
    dc: &ID2D1DeviceContext,
    a: (f32, f32),
    b: (f32, f32),
    c: D2D1_COLOR_F,
    width: f32,
) -> Result<()> {
    unsafe {
        let brush = dc.CreateSolidColorBrush(&c, None)?;
        dc.DrawLine(
            Vector2 { X: a.0, Y: a.1 },
            Vector2 { X: b.0, Y: b.1 },
            &brush,
            width,
            None,
        );
        Ok(())
    }
}

/// Rounded distance-field displacement, derived from Glance skins.rs
/// (MIT, lulu-loopp, commit ec0fb40). R/G encode the true background shift.
pub fn displaced(w: f32, h: f32, r: f32, x: f32, y: f32) -> [u8; 4] {
    let (hx, hy) = (w / 2.0, h / 2.0);
    let r = r.min(hx).min(hy);
    let (x, y) = (x - hx, y - hy);
    let (qx, qy) = (x.abs() - (hx - r), y.abs() - (hy - r));
    let (depth, nx, ny) = if qx > 0.0 && qy > 0.0 {
        let len = qx.hypot(qy);
        (r - len, qx / len, qy / len)
    } else if qx > qy {
        (hx - x.abs(), 1.0, 0.0)
    } else {
        (hy - y.abs(), 0.0, 1.0)
    };
    let strength = (1.0 - (depth / 16.0).clamp(0.0, 1.0)).powi(2);
    [
        128,
        (128.0 + ny * y.signum() * strength * 127.0).round() as u8,
        (128.0 + nx * x.signum() * strength * 127.0).round() as u8,
        255,
    ]
}
#[cfg(test)]
mod lens_tests {
    use super::*;
    #[test]
    fn the_center_is_neutral_and_the_rim_bends_outward() {
        assert_eq!(
            displaced(400.0, 600.0, 24.0, 200.0, 300.0),
            [128, 128, 128, 255]
        );
        assert!(displaced(400.0, 600.0, 24.0, 0.5, 300.0)[2] < 128);
        assert!(displaced(400.0, 600.0, 24.0, 399.5, 300.0)[2] > 128);
        assert!(displaced(400.0, 600.0, 24.0, 200.0, 0.5)[1] < 128);
        assert!(displaced(400.0, 600.0, 24.0, 200.0, 599.5)[1] > 128);
    }
}
pub struct Lens {
    map: ID2D1Bitmap1,
    size: (u32, u32),
    scale: f32,
}
impl Lens {
    pub fn new(gfx: &Gfx, size: (u32, u32), scale: f32) -> Result<Self> {
        unsafe {
            let mut pixels = Vec::with_capacity((size.0 * size.1 * 4) as usize);
            for y in 0..size.1 {
                for x in 0..size.0 {
                    pixels.extend_from_slice(&displaced(
                        size.0 as f32 / scale - 20.0,
                        size.1 as f32 / scale - 20.0,
                        24.0,
                        (x as f32 + 0.5) / scale - 10.0,
                        (y as f32 + 0.5) / scale - 10.0,
                    ));
                }
            }
            let props = D2D1_BITMAP_PROPERTIES1 {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 96.0 * scale,
                dpiY: 96.0 * scale,
                ..Default::default()
            };
            let map = gfx.dc.CreateBitmap(
                D2D_SIZE_U {
                    width: size.0,
                    height: size.1,
                },
                Some(pixels.as_ptr().cast()),
                size.0 * 4,
                &props,
            )?;
            Ok(Self { map, size, scale })
        }
    }
    pub fn matches(&self, size: (u32, u32), scale: f32) -> bool {
        self.size == size && self.scale == scale
    }
    pub fn draw(
        &self,
        dc: &ID2D1DeviceContext,
        bitmap: &ID2D1Bitmap1,
        origin: (f32, f32),
        blur: f32,
    ) -> Result<()> {
        unsafe {
            let transform = dc.CreateEffect(&CLSID_D2D12DAffineTransform)?;
            transform.SetInput(0, bitmap, true);
            property(
                &transform,
                D2D1_2DAFFINETRANSFORM_PROP_TRANSFORM_MATRIX.0 as u32,
                D2D1_PROPERTY_TYPE_MATRIX_3X2,
                &Matrix3x2::translation(origin.0, origin.1),
            )?;
            let frost = dc.CreateEffect(&CLSID_D2D1GaussianBlur)?;
            frost.SetInput(0, &transform.GetOutput()?, true);
            property(&frost, 0, D2D1_PROPERTY_TYPE_FLOAT, &blur)?;
            property(&frost, 2, D2D1_PROPERTY_TYPE_ENUM, &1u32)?;
            let bend = dc.CreateEffect(&CLSID_D2D1DisplacementMap)?;
            bend.SetInput(0, &frost.GetOutput()?, true);
            bend.SetInput(1, &self.map, true);
            property(&bend, 0, D2D1_PROPERTY_TYPE_FLOAT, &16.0f32)?;
            property(&bend, 1, D2D1_PROPERTY_TYPE_ENUM, &0u32)?;
            property(&bend, 2, D2D1_PROPERTY_TYPE_ENUM, &1u32)?;
            dc.DrawImage(
                &bend.GetOutput()?,
                None,
                None,
                D2D1_INTERPOLATION_MODE_LINEAR,
                D2D1_COMPOSITE_MODE_SOURCE_OVER,
            );
            Ok(())
        }
    }
}

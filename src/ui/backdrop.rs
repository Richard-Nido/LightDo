//! What is on screen behind the panel, for skins that draw the desktop
//! through glass or acrylic.

use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct2D::Common::{
    D2D_SIZE_U, D2D1_ALPHA_MODE_IGNORE, D2D1_PIXEL_FORMAT,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1_BITMAP_OPTIONS_NONE, D2D1_BITMAP_OPTIONS_TARGET, D2D1_BITMAP_PROPERTIES1, ID2D1Bitmap1,
    ID2D1DeviceContext,
};
use windows::Win32::Graphics::Direct3D11::ID3D11Texture2D;
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Dxgi::IDXGISurface;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleDC, CreateDIBSection,
    DIB_RGB_COLORS, DeleteDC, DeleteObject, GdiFlush, GetDC, ReleaseDC, SRCCOPY, SelectObject,
};
use windows::core::{Interface, Result};

/// The screen inside a rectangle, as it was when captured.
pub struct Capture {
    /// Physical screen coordinates.
    pub rect: RECT,
    /// Rows top to bottom, BGRA, opaque; none once let go of (see
    /// `forget_pixels`).
    pixels: Option<Vec<u8>>,
    /// How light it is, a value (0–255) for each `LUMA_STEP` square of
    /// pixels: what is read of it once its pixels are let go of (or when it
    /// had none in memory: see `from_texture`).
    luma: Vec<u8>,
    /// Tells one capture's content from another's.
    pub digest: u64,
    /// Where it is on the graphics card, when it was never in memory: a
    /// frame of what is behind (see `scene`), drawn from there as it is.
    texture: Option<Texture>,
}

/// A texture of the device Glance draws with, made on another thread (see `scene`) and drawn on the one that draws:
/// copied from, never drawn from where it is (see `Capture::bitmap`), so that once let go it is no one's, and goes
/// back to `spare` to be written again.
pub struct Texture {
    pub texture: ID3D11Texture2D,
    pub spare: std::sync::Arc<Spare>,
}

/// Textures let go of, to be written again, the most recent last: a few at most (see `SPARE_MOST`).
#[derive(Default)]
pub struct Spare(pub std::sync::Mutex<Vec<ID3D11Texture2D>>);

/// How many textures are kept spare: the one written next (one more is the capture's, till it is drawn).
const SPARE_MOST: usize = 1;

impl Drop for Texture {
    fn drop(&mut self) {
        let mut spare = self.spare.0.lock().unwrap();
        if spare.len() < SPARE_MOST {
            spare.push(self.texture.clone());
        }
    }
}

// (A Direct3D 11 resource may be handed between threads; it is drawn on the device's.)
unsafe impl Send for Texture {}
unsafe impl Sync for Texture {}
unsafe impl Send for Spare {}
unsafe impl Sync for Spare {}

/// The side of the squares of pixels the luminance is kept for.
pub const LUMA_STEP: i32 = 2;

/// What tells one screenful from another: every pixel, eight bytes at a
/// time, mixed as FxHash mixes (a screen's worth in a few milliseconds,
/// where a general hasher takes tens). Alpha, which the copy leaves as it
/// is, is left out.
fn digest(pixels: &[u8]) -> u64 {
    const K: u64 = 0x517C_C1B7_2722_0A95;
    const COLOUR: u64 = 0x00FF_FFFF_00FF_FFFF;
    let mut hash = pixels.len() as u64;
    let mut words = pixels.chunks_exact(8);
    for word in &mut words {
        let v = u64::from_le_bytes(word.try_into().unwrap()) & COLOUR;
        hash = (hash.rotate_left(5) ^ v).wrapping_mul(K);
    }
    for &byte in words.remainder() {
        hash = (hash.rotate_left(5) ^ byte as u64).wrapping_mul(K);
    }
    hash
}

/// How many `LUMA_STEP` squares span `pixels`.
fn squares(pixels: i32) -> i32 {
    (pixels + LUMA_STEP - 1) / LUMA_STEP
}

/// The relative luminance (0–1) of a BGRA pixel at `i`.
fn pixel_luminance(pixels: &[u8], i: usize) -> f64 {
    channels_luminance(pixels[i + 2] as f64, pixels[i + 1] as f64, pixels[i] as f64)
}

/// The relative luminance (0–1) of red, green and blue (0–255 each), or of sums of them (0–255 times as many as
/// summed: the luminance is a weighted sum, and that of their mean is the mean of theirs).
fn channels_luminance(red: f64, green: f64, blue: f64) -> f64 {
    (0.2126 * red + 0.7152 * green + 0.0722 * blue) / 255.0
}

/// A BGRA pixel's luminance as kept for its square (see `Capture::luma`): out of 255, the nearest whole one (the
/// weights in ten-thousandths, whole numbers: no float of a sum of them rounded the wrong way).
fn pixel_luma(pixel: &[u8]) -> u8 {
    ((2126 * pixel[2] as u32 + 7152 * pixel[1] as u32 + 722 * pixel[0] as u32 + 5000) / 10000) as u8
}

impl Capture {
    /// Copies the screen inside `rect`; `None` while it cannot be read (the
    /// lock screen or a UAC prompt has the display), or for an empty `rect`.
    pub fn take(rect: RECT) -> Option<Self> {
        let (width, height) = (rect.right - rect.left, rect.bottom - rect.top);
        if width <= 0 || height <= 0 {
            return None;
        }
        let header = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            // Negative: rows run top to bottom.
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        };
        let info = BITMAPINFO {
            bmiHeader: header,
            ..Default::default()
        };
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        unsafe {
            let screen = GetDC(None);
            let memory = CreateCompatibleDC(Some(screen));
            let mut bits = std::ptr::null_mut();
            let copied = CreateDIBSection(Some(memory), &info, DIB_RGB_COLORS, &mut bits, None, 0)
                .ok()
                .map(|bitmap| {
                    let previous = SelectObject(memory, bitmap.into());
                    // Read once GDI has done the copy, not while it may still be
                    // queued.
                    let copied = BitBlt(
                        memory,
                        0,
                        0,
                        width,
                        height,
                        Some(screen),
                        rect.left,
                        rect.top,
                        SRCCOPY,
                    )
                    .is_ok()
                        && GdiFlush().as_bool();
                    if copied {
                        let length = pixels.len();
                        pixels
                            .copy_from_slice(std::slice::from_raw_parts(bits as *const u8, length));
                    }
                    SelectObject(memory, previous);
                    let _ = DeleteObject(bitmap.into());
                    copied
                });
            let _ = DeleteDC(memory);
            ReleaseDC(None, screen);
            if copied != Some(true) {
                return None;
            }
        }
        // The copy leaves alpha at zero: its bitmap ignores it (the screen
        // is opaque), rather than every pixel being set.
        Some(Capture::from_pixels(rect, pixels))
    }

    /// A picture already in memory (BGRA, opaque, rows top to bottom),
    /// standing for the screen inside `rect`.
    pub fn from_pixels(rect: RECT, pixels: Vec<u8>) -> Self {
        let (width, height) = (rect.right - rect.left, rect.bottom - rect.top);
        let (columns, rows) = (squares(width), squares(height));
        let mut luma = Vec::with_capacity((columns * rows) as usize);
        // (Each square's top left pixel's, row by row.)
        for line in pixels
            .chunks_exact((width * 4) as usize)
            .step_by(LUMA_STEP as usize)
        {
            luma.extend(
                line.chunks_exact(4)
                    .step_by(LUMA_STEP as usize)
                    .map(pixel_luma),
            );
        }
        let digest = digest(&pixels);
        Capture {
            rect,
            pixels: Some(pixels),
            luma,
            digest,
            texture: None,
        }
    }

    /// A picture on the graphics card (`texture`, of the device Glance draws with), standing for the screen inside
    /// `rect`: how light it is (`luma`, a value for each `LUMA_STEP` square, as `from_pixels` makes it), and what tells
    /// it from another (`frame`, a count of the frames it is one of).
    pub fn from_texture(rect: RECT, texture: Texture, luma: Vec<u8>, frame: u64) -> Self {
        Capture {
            rect,
            pixels: None,
            luma,
            digest: frame,
            texture: Some(texture),
        }
    }

    /// Lets go of the texture it was taken in, once it is a bitmap (see `bitmap_into`), for the next capture to be
    /// taken in; what was taken from memory (see `take`) is kept.
    pub fn forget_texture(&mut self) {
        self.texture = None;
    }

    /// A copy of a capture taken into memory (see `take`, `from_pixels`): none of one on the graphics card.
    pub fn copied(&self) -> Option<Self> {
        let pixels = self.pixels.clone()?;
        Some(Capture {
            rect: self.rect,
            pixels: Some(pixels),
            luma: self.luma.clone(),
            digest: self.digest,
            texture: None,
        })
    }

    /// The same capture without its pixels (what is read of it is from its
    /// luminance): to be lent along with a bitmap already made of it.
    pub fn without_pixels(&self) -> Self {
        Capture {
            rect: self.rect,
            pixels: None,
            luma: self.luma.clone(),
            digest: self.digest,
            texture: None,
        }
    }

    fn width(&self) -> i32 {
        self.rect.right - self.rect.left
    }

    /// Lets go of its pixels once they are a bitmap: a screen's worth of
    /// memory. What is read of it after is from its luminance (see `luma`);
    /// a bitmap is not made of it again.
    pub fn forget_pixels(&mut self) {
        self.pixels = None;
        self.texture = None;
    }

    /// The relative luminance (0–1) at (`x`, `y`) from its corner: of the
    /// pixel while it has its pixels, else of the square it is in.
    fn luminance_at(&self, x: i32, y: i32) -> f64 {
        match &self.pixels {
            Some(pixels) => pixel_luminance(pixels, ((y * self.width() + x) * 4) as usize),
            None => {
                self.luma[((y / LUMA_STEP) * squares(self.width()) + x / LUMA_STEP) as usize] as f64
                    / 255.0
            }
        }
    }

    /// The mean relative luminance (0–1) of the pixels in `columns` × `rows` from its corner, as `luminance_at` has
    /// each: the channels (or the squares' values) summed as whole numbers, and weighted once.
    fn mean_luminance(&self, columns: std::ops::Range<i32>, rows: std::ops::Range<i32>) -> f64 {
        let count = (columns.len() * rows.len()) as f64;
        let width = self.width();
        match &self.pixels {
            Some(pixels) => {
                let (mut red, mut green, mut blue) = (0u32, 0u32, 0u32);
                for y in rows {
                    let line = &pixels[((y * width + columns.start) * 4) as usize
                        ..((y * width + columns.end) * 4) as usize];
                    for pixel in line.chunks_exact(4) {
                        blue += pixel[0] as u32;
                        green += pixel[1] as u32;
                        red += pixel[2] as u32;
                    }
                }
                channels_luminance(red as f64, green as f64, blue as f64) / count
            }
            None => {
                let across = squares(width);
                let mut total = 0u32;
                for y in rows {
                    let line = &self.luma[((y / LUMA_STEP) * across) as usize..];
                    total += columns
                        .clone()
                        .map(|x| line[(x / LUMA_STEP) as usize] as u32)
                        .sum::<u32>();
                }
                total as f64 / 255.0 / count
            }
        }
    }

    /// The capture as a bitmap at `px` physical pixels per DIP, so that it
    /// covers its rectangle's size in DIPs.
    pub fn bitmap(&self, dc: &ID2D1DeviceContext, px: f32) -> Result<ID2D1Bitmap1> {
        self.bitmap_into(dc, px, None)
    }

    /// `made`, a bitmap made of this capture (see `bitmap_into`), at `px` physical pixels per DIP: the same pixels,
    /// nothing copied (a panel sized is drawn at another zoom at each step). Made again from the capture if its
    /// pixels cannot be taken at another scale (made from memory, not on the card).
    pub fn at_scale(
        &self,
        dc: &ID2D1DeviceContext,
        made: &ID2D1Bitmap1,
        px: f32,
    ) -> Result<ID2D1Bitmap1> {
        let mut dpi = (0.0, 0.0);
        unsafe { made.GetDpi(&mut dpi.0, &mut dpi.1) };
        if dpi.0 == 96.0 * px {
            return Ok(made.clone());
        }
        let properties = D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_IGNORE,
            },
            dpiX: 96.0 * px,
            dpiY: 96.0 * px,
            bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET,
            ..Default::default()
        };
        unsafe {
            made.GetSurface()
                .and_then(|surface| dc.CreateBitmapFromDxgiSurface(&surface, Some(&properties)))
        }
        .or_else(|_| self.bitmap(dc, px))
    }

    /// As `bitmap`; a frame on the graphics card copied into `into` if it is
    /// of the same size (one made before for the same place, at this scale
    /// or another: its memory written again, not another taken for each
    /// frame, or for each scale a panel is sized through).
    pub fn bitmap_into(
        &self,
        dc: &ID2D1DeviceContext,
        px: f32,
        into: Option<&ID2D1Bitmap1>,
    ) -> Result<ID2D1Bitmap1> {
        let properties = D2D1_BITMAP_PROPERTIES1 {
            // Opaque whatever its alpha bytes say (see `take`).
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_IGNORE,
            },
            dpiX: 96.0 * px,
            dpiY: 96.0 * px,
            bitmapOptions: D2D1_BITMAP_OPTIONS_NONE,
            ..Default::default()
        };
        // (A target's memory is a surface of the card's, which may be taken again at another scale.)
        let copies = D2D1_BITMAP_PROPERTIES1 {
            bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET,
            ..properties.clone()
        };
        match (&self.pixels, &self.texture) {
            (Some(pixels), _) => {
                let size = D2D_SIZE_U {
                    width: self.width() as u32,
                    height: (self.rect.bottom - self.rect.top) as u32,
                };
                unsafe {
                    dc.CreateBitmap(
                        size,
                        Some(pixels.as_ptr().cast()),
                        (self.width() * 4) as u32,
                        &properties,
                    )
                }
            }
            // Copied on the card (nothing sent to or from it): the texture is then no one's but the capture's, and
            // goes back to be written again as the capture goes (see `Texture`).
            (None, Some(texture)) => unsafe {
                let frame = dc.CreateBitmapFromDxgiSurface(
                    &texture.texture.cast::<IDXGISurface>()?,
                    Some(&properties),
                )?;
                let size = frame.GetPixelSize();
                let bitmap = match into.filter(|bitmap| {
                    let have = bitmap.GetPixelSize();
                    have.width == size.width && have.height == size.height
                }) {
                    Some(bitmap) => {
                        let mut dpi = (0.0, 0.0);
                        bitmap.GetDpi(&mut dpi.0, &mut dpi.1);
                        if dpi.0 == 96.0 * px {
                            bitmap.clone()
                        } else {
                            dc.CreateBitmapFromDxgiSurface(&bitmap.GetSurface()?, Some(&copies))?
                        }
                    }
                    None => dc.CreateBitmap(size, None, 0, &copies)?,
                };
                bitmap.CopyFromBitmap(None, &frame, None)?;
                Ok(bitmap)
            },
            (None, None) => Err(windows::Win32::Foundation::E_UNEXPECTED.into()),
        }
    }

    /// The relative luminance, as WCAG measures it (its channels made
    /// linear first), at (`x`, `y`) from its corner, while it has its
    /// pixels; else its luminance as kept.
    fn wcag_at(&self, x: i32, y: i32) -> f32 {
        let channel = |v: u8| v as f32 / 255.0;
        match &self.pixels {
            Some(pixels) => {
                let i = ((y * self.width() + x) * 4) as usize;
                crate::ui::overlay::luminance(
                    channel(pixels[i + 2]),
                    channel(pixels[i + 1]),
                    channel(pixels[i]),
                )
            }
            None => self.luminance_at(x, y) as f32,
        }
    }

    /// The relative luminance (0–1, as WCAG measures it) of every `step`th
    /// pixel each way inside `area` but outside `hole` (both in the
    /// capture's coordinates: physical screen ones for the screen's).
    pub fn luminances(&self, area: RECT, hole: RECT, step: i32) -> Vec<f32> {
        let r = self.rect;
        let (left, right) = (area.left.max(r.left), area.right.min(r.right));
        let (top, bottom) = (area.top.max(r.top), area.bottom.min(r.bottom));
        let mut luminances = Vec::new();
        for y in (top..bottom).step_by(step.max(1) as usize) {
            for x in (left..right).step_by(step.max(1) as usize) {
                if x >= hole.left && x < hole.right && y >= hole.top && y < hole.bottom {
                    continue;
                }
                luminances.push(self.wcag_at(x - r.left, y - r.top));
            }
        }
        luminances
    }

    /// The mean and standard deviation (0–1) of the luminance behind `area`
    /// (physical screen coordinates), sampled once per DIP: fine enough to
    /// see text strokes, and independent of the display's scale.
    pub fn luminance(&self, area: RECT, px: f32) -> (f32, f32) {
        let clip = |v: i32, low: i32, high: i32| v.clamp(low, high);
        let left = clip(area.left, self.rect.left, self.rect.right) - self.rect.left;
        let right = clip(area.right, self.rect.left, self.rect.right) - self.rect.left;
        let top = clip(area.top, self.rect.top, self.rect.bottom) - self.rect.top;
        let bottom = clip(area.bottom, self.rect.top, self.rect.bottom) - self.rect.top;
        let (columns, rows) = (
            ((right - left) as f32 / px) as i32,
            ((bottom - top) as f32 / px) as i32,
        );
        if columns <= 0 || rows <= 0 {
            return (0.5, 0.0);
        }
        // The pixels each DIP covers across (at least one), the same in every row.
        let spans: Vec<(i32, i32)> = (0..columns)
            .map(|column| {
                let (x0, x1) = (
                    left + (column as f32 * px) as i32,
                    left + ((column + 1) as f32 * px) as i32,
                );
                (x0, x1.max(x0 + 1))
            })
            .collect();
        let (mut sum, mut squares) = (0.0f64, 0.0f64);
        for row in 0..rows {
            let (y0, y1) = (
                top + (row as f32 * px) as i32,
                top + ((row + 1) as f32 * px) as i32,
            );
            let y1 = y1.max(y0 + 1);
            for &(x0, x1) in &spans {
                // The average of the pixels in this DIP.
                let value = self.mean_luminance(x0..x1, y0..y1);
                sum += value;
                squares += value * value;
            }
        }
        let count = (columns * rows) as f64;
        let mean = sum / count;
        (
            mean as f32,
            (squares / count - mean * mean).max(0.0).sqrt() as f32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A screenful of pixels varied enough that no two neighbours are alike: (x, y) mixed as a hash mixes.
    fn varied(width: i32, height: i32) -> Vec<u8> {
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                let mixed = ((x as u32).wrapping_mul(0x9E37_79B9)
                    ^ (y as u32).wrapping_mul(0x85EB_CA6B))
                .wrapping_mul(0xC2B2_AE35);
                pixels.extend([
                    (mixed >> 8) as u8,
                    (mixed >> 16) as u8,
                    (mixed >> 24) as u8,
                    0,
                ]);
            }
        }
        pixels
    }

    /// The mean and spread of the luminance behind `area`, worked out pixel by pixel as `luminance_at` has each.
    fn luminance_pixel_by_pixel(capture: &Capture, area: RECT, px: f32) -> (f32, f32) {
        let r = capture.rect;
        let (left, right) = (
            area.left.clamp(r.left, r.right) - r.left,
            area.right.clamp(r.left, r.right) - r.left,
        );
        let (top, bottom) = (
            area.top.clamp(r.top, r.bottom) - r.top,
            area.bottom.clamp(r.top, r.bottom) - r.top,
        );
        let (columns, rows) = (
            ((right - left) as f32 / px) as i32,
            ((bottom - top) as f32 / px) as i32,
        );
        let (mut sum, mut squares) = (0.0f64, 0.0f64);
        for row in 0..rows {
            let (y0, y1) = (
                top + (row as f32 * px) as i32,
                top + ((row + 1) as f32 * px) as i32,
            );
            for column in 0..columns {
                let (x0, x1) = (
                    left + (column as f32 * px) as i32,
                    left + ((column + 1) as f32 * px) as i32,
                );
                let (mut cell, mut count) = (0.0f64, 0);
                for y in y0..y1.max(y0 + 1) {
                    for x in x0..x1.max(x0 + 1) {
                        cell += capture.luminance_at(x, y);
                        count += 1;
                    }
                }
                let value = cell / count as f64;
                sum += value;
                squares += value * value;
            }
        }
        let count = (columns * rows) as f64;
        let mean = sum / count;
        (
            mean as f32,
            (squares / count - mean * mean).max(0.0).sqrt() as f32,
        )
    }

    /// How light the desktop behind is, summed a DIP at a time, is what it is pixel by pixel: at every scale, over
    /// the whole capture and a part of it, from its pixels and from its squares' values once they are let go.
    #[test]
    fn the_luminance_behind_is_as_it_is_pixel_by_pixel() {
        let rect = RECT {
            left: 100,
            top: 40,
            right: 100 + 181,
            bottom: 40 + 123,
        };
        let mut capture = Capture::from_pixels(rect, varied(181, 123));
        let areas = [
            rect,
            RECT {
                left: 117,
                top: 51,
                right: 260,
                bottom: 140,
            },
            RECT {
                left: 0,
                top: 0,
                right: 150,
                bottom: 90,
            },
        ];
        for kept in [true, false] {
            if !kept {
                capture.forget_pixels();
            }
            for area in areas {
                for px in [1.0, 1.25, 1.5, 2.0, 2.5] {
                    let (mean, spread) = capture.luminance(area, px);
                    let (expected_mean, expected_spread) =
                        luminance_pixel_by_pixel(&capture, area, px);
                    assert!(
                        (mean - expected_mean).abs() < 1e-5
                            && (spread - expected_spread).abs() < 1e-5,
                        "{area:?} at {px} (pixels kept: {kept}): {mean}, {spread}; pixel by pixel {expected_mean}, {expected_spread}"
                    );
                }
            }
        }
    }

    /// Each square's value is its top left pixel's luminance out of 255, the nearest whole one.
    #[test]
    fn a_square_is_as_light_as_its_first_pixel() {
        let (width, height) = (37, 21);
        let pixels = varied(width, height);
        let capture = Capture::from_pixels(
            RECT {
                left: 0,
                top: 0,
                right: width,
                bottom: height,
            },
            pixels.clone(),
        );
        assert_eq!(
            capture.luma.len(),
            (squares(width) * squares(height)) as usize
        );
        for row in 0..squares(height) {
            for column in 0..squares(width) {
                let i = (((row * LUMA_STEP) * width + column * LUMA_STEP) * 4) as usize;
                let exact = 0.2126 * pixels[i + 2] as f64
                    + 0.7152 * pixels[i + 1] as f64
                    + 0.0722 * pixels[i] as f64;
                assert_eq!(
                    capture.luma[(row * squares(width) + column) as usize],
                    exact.round() as u8,
                    "({column}, {row})"
                );
            }
        }
    }
}

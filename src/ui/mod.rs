pub mod backdrop;
pub mod gfx;
pub mod scene;
pub mod overlay {
    pub fn luminance(r: f32, g: f32, b: f32) -> f32 {
        fn linear(x: f32) -> f32 {
            if x <= 0.04045 {
                x / 12.92
            } else {
                ((x + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
    }
}

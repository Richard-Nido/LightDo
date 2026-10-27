//! Physical-pixel drag/resize math, independent of HWNDs and rendering.
use crate::model::Side;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}
#[derive(Clone, Copy, Debug)]
pub enum Gesture {
    Move,
    Resize {
        left: bool,
        right: bool,
        top: bool,
        bottom: bool,
    },
}
pub fn drag(original: Rect, kind: Gesture, delta: (i32, i32), scale: f32) -> Rect {
    let mut r = original;
    let (dx, dy) = delta;
    match kind {
        Gesture::Move => {
            r.x += dx;
            r.y += dy;
        }
        Gesture::Resize {
            left,
            right,
            top,
            bottom,
        } => {
            let clamp_w = |v: i32| v.clamp((320.0 * scale) as i32, (1000.0 * scale) as i32);
            let clamp_h = |v: i32| v.clamp((360.0 * scale) as i32, (1600.0 * scale) as i32);
            if left {
                r.w = clamp_w(original.w - dx);
                r.x = original.x + original.w - r.w;
            } else if right {
                r.w = clamp_w(original.w + dx);
            }
            if top {
                r.h = clamp_h(original.h - dy);
                r.y = original.y + original.h - r.h;
            } else if bottom {
                r.h = clamp_h(original.h + dy);
            }
        }
    }
    r
}
/// Preview and release use the same logical distance at every display scale.
pub fn snap_side(panel: Rect, work: Rect, distance: u32, scale: f32) -> Option<Side> {
    let threshold = (distance as f32 * scale).round() as i64;
    let left = (i64::from(panel.x) - i64::from(work.x)).abs();
    let right =
        (i64::from(panel.x) + i64::from(panel.w) - i64::from(work.x) - i64::from(work.w)).abs();
    if left <= threshold && left <= right {
        Some(Side::Left)
    } else if right <= threshold {
        Some(Side::Right)
    } else {
        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resizing_at_all_target_scales_keeps_opposite_corner() {
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let original = Rect {
                x: -400,
                y: 200,
                w: (400.0 * scale) as i32,
                h: (600.0 * scale) as i32,
            };
            let r = drag(
                original,
                Gesture::Resize {
                    left: true,
                    right: false,
                    top: true,
                    bottom: false,
                },
                (10000, 10000),
                scale,
            );
            assert_eq!(r.x + r.w, original.x + original.w);
            assert_eq!(r.y + r.h, original.y + original.h);
            assert_eq!(r.w, (320.0 * scale) as i32);
            assert_eq!(r.h, (360.0 * scale) as i32);
        }
    }
    #[test]
    fn moving_preserves_size_in_negative_monitor_coordinates() {
        let r = Rect {
            x: 20,
            y: 30,
            w: 400,
            h: 600,
        };
        assert_eq!(
            drag(r, Gesture::Move, (-1000, -300), 1.0),
            Rect {
                x: -980,
                y: -270,
                w: 400,
                h: 600
            }
        );
    }
    #[test]
    fn snap_distance_scales_at_both_edges_and_custom_values_change_the_boundary() {
        let work = Rect {
            x: -1920,
            y: 0,
            w: 1920,
            h: 1080,
        };
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let threshold = (16.0_f32 * scale).round() as i32;
            let mut panel = Rect {
                x: work.x + threshold,
                y: 100,
                w: 400,
                h: 600,
            };
            assert_eq!(snap_side(panel, work, 16, scale), Some(Side::Left));
            panel.x += 1;
            assert_eq!(snap_side(panel, work, 16, scale), None);
            panel.x = work.x + work.w - panel.w - threshold;
            assert_eq!(snap_side(panel, work, 16, scale), Some(Side::Right));
            panel.x -= 1;
            assert_eq!(snap_side(panel, work, 16, scale), None);
            assert_eq!(snap_side(panel, work, 32, scale), Some(Side::Right));
        }
    }
    #[test]
    fn wide_snap_ranges_choose_the_nearest_edge_instead_of_always_the_left() {
        let work = Rect {
            x: 0,
            y: 0,
            w: 500,
            h: 800,
        };
        let panel = Rect {
            x: 70,
            y: 50,
            w: 400,
            h: 600,
        };
        assert_eq!(snap_side(panel, work, 96, 1.0), Some(Side::Right));
    }
}

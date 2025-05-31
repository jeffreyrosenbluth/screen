use image::*;
use std::cmp::{max, min};

pub(crate) type SortFn = fn(Rgba<u8>) -> i16;

pub(crate) fn luma(c: Rgba<u8>) -> i16 {
    c.to_luma()[0] as i16
}

pub(crate) fn hue(c: Rgba<u8>) -> i16 {
    let hsl = hsl(c[0], c[1], c[2]);
    let hue = hsl.0 / 360.0 * 255.0;
    hue as i16
}

pub(crate) fn sat(c: Rgba<u8>) -> i16 {
    let hsl = hsl(c[0], c[1], c[2]);
    let sat = hsl.1 * 255.0;
    sat as i16
}

pub(crate) fn max_rgb(c: Rgba<u8>) -> i16 {
    let max = max(max(c[0], c[1]), c[2]);
    max as i16
}

pub(crate) fn min_rgb(c: Rgba<u8>) -> i16 {
    let min = min(min(c[0], c[1]), c[2]);
    min as i16
}

pub(crate) fn r_g(c: Rgba<u8>) -> i16 {
    (c[0] - c[1]) as i16
}

pub(crate) fn g_b(c: Rgba<u8>) -> i16 {
    (c[1] - c[2]) as i16
}

pub(crate) fn b_r(c: Rgba<u8>) -> i16 {
    (c[2] - c[0]) as i16
}

pub(crate) fn wrapped_hue(c: Rgba<u8>) -> i16 {
    let hsl = hsl(c[0], c[1], c[2]);
    let h = f32::min(hsl.0, 360.0 - hsl.0);
    (h / 360.0 * 255.0) as i16
}

pub(crate) fn hue_sat(c: Rgba<u8>) -> i16 {
    hue(c) * sat(c)
}

pub(crate) fn luma_sat(c: Rgba<u8>) -> i16 {
    luma(c) * sat(c)
}

pub(crate) fn chroma(c: Rgba<u8>) -> i16 {
    let mx = max(max(c[0], c[1]), c[2]);
    let mn = min(min(c[0], c[1]), c[2]);
    (mx - mn) as i16
}

fn hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let mut h: f32;
    let max = max(max(r, g), b);
    let min = min(min(r, g), b);
    let (r, g, b) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    let (min, max) = (min as f32 / 255.0, max as f32 / 255.0);

    let l = (max + min) / 2.0;

    let delta: f32 = max - min;
    if delta == 0.0 {
        // it's gray
        return (0.0, 0.0, l);
    }
    let s = if l < 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };
    let r2 = (((max - r) / 6.0) + (delta / 2.0)) / delta;
    let g2 = (((max - g) / 6.0) + (delta / 2.0)) / delta;
    let b2 = (((max - b) / 6.0) + (delta / 2.0)) / delta;

    h = match max {
        x if x == r => b2 - g2,
        x if x == g => (1.0 / 3.0) + r2 - b2,
        _ => (2.0 / 3.0) + g2 - r2,
    };
    if h < 0 as f32 {
        h += 1.0;
    } else if h > 1.0 {
        h -= 1.0;
    }
    let h_degrees = (h * 360.0 * 100.0).round() / 100.0;

    (h_degrees, s, l)
}

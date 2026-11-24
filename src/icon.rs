//! The application icon, drawn procedurally (used for the window, the tray, and the Windows executable).
//! This file is also included by `build.rs` to generate the `.ico` resource.
//!
//! The drawing is the bellows of an accordion (the meaning of "mantice") seen from the side: its folds widen
//! from left to right, and the text cursor stands at the narrow end, where a short keyword opens up into a
//! long text.

type Point = (f32, f32);
type Segment = (Point, Point);

/// The drawing, in normalized coordinates (0 to 1 on both axes).
struct Glyph {
    /// Lines of the text cursor.
    cursor: Vec<Segment>,
    /// The silhouette of the bellows: a polygon with zigzag top and bottom edges.
    bellows: Vec<Point>,
    /// The creases between the folds, drawn over the silhouette in the color of the background.
    creases: Vec<Segment>,
}

/// Render the icon as RGBA pixels. `active == false` renders the grayscale "paused" variant.
pub fn render_rgba(size: u32, active: bool) -> Vec<u8> {
    const SS: u32 = 4; // supersampling factor
    let n = size as f32;
    let mut out = vec![0u8; (size * size * 4) as usize];
    let radius = 0.23;
    // fewer folds, a thicker cursor and no creases when the icon is small
    let small = size < 32;
    let glyph = glyph(if small { 2 } else { 5 }, !small);
    let (cursor_stroke, crease_stroke) = if small { (0.055, 0.0) } else { (0.034, 0.017) };
    let (top, bottom, fg) = if active {
        ([0x2B, 0x4F, 0xC0], [0x0B, 0x1E, 0x5A], [0xFF, 0xF3, 0xD4])
    } else {
        ([0xA8, 0xAE, 0xB5], [0x5B, 0x63, 0x6B], [0xFF, 0xFF, 0xFF])
    };
    for py in 0..size {
        for px in 0..size {
            let (mut bg_cov, mut fg_cov) = (0.0f32, 0.0f32);
            let mut top_mix = 0.0f32;
            for sy in 0..SS {
                for sx in 0..SS {
                    let x = (px as f32 + (sx as f32 + 0.5) / SS as f32) / n;
                    let y = (py as f32 + (sy as f32 + 0.5) / SS as f32) / n;
                    if in_rounded_rect(x, y, 0.04, 0.96, radius) {
                        bg_cov += 1.0;
                        top_mix += 1.0 - y;
                        let on_cursor = glyph.cursor.iter().any(|(a, b)| dist_to_segment(x, y, *a, *b) < cursor_stroke);
                        let on_bellows = in_polygon(x, y, &glyph.bellows)
                            && !glyph.creases.iter().any(|(a, b)| dist_to_segment(x, y, *a, *b) < crease_stroke);
                        if on_cursor || on_bellows {
                            fg_cov += 1.0;
                        }
                    }
                }
            }
            let samples = (SS * SS) as f32;
            if bg_cov == 0.0 {
                continue;
            }
            let t = top_mix / bg_cov; // 1 at the top, 0 at the bottom
            let fg_ratio = fg_cov / bg_cov;
            let i = ((py * size + px) * 4) as usize;
            for c in 0..3 {
                let bg = bottom[c] as f32 + (top[c] as f32 - bottom[c] as f32) * t;
                out[i + c] = (bg * (1.0 - fg_ratio) + fg[c] as f32 * fg_ratio).round() as u8;
            }
            out[i + 3] = (255.0 * bg_cov / samples).round() as u8;
        }
    }
    out
}

/// The text cursor at the narrow end and the bellows, which widens to the right and has `folds` outer folds on
/// each edge. `serifs` adds the little bars at the ends of the cursor.
fn glyph(folds: usize, serifs: bool) -> Glyph {
    const CENTER: f32 = 0.5;
    const X_NARROW: f32 = 0.29;
    const X_WIDE: f32 = 0.85;
    const H_NARROW: f32 = 0.06; // half height of the bellows at the narrow end
    const H_WIDE: f32 = 0.29; // and at the wide end
    const INNER: f32 = 0.62; // depth of the valleys of the zigzag, relative to the outer folds
    let half_height = |x: f32| H_NARROW + (x - X_NARROW) / (X_WIDE - X_NARROW) * (H_WIDE - H_NARROW);

    let cursor_x = 0.155;
    let cursor_h = 0.17;
    let mut cursor = vec![((cursor_x, CENTER - cursor_h), (cursor_x, CENTER + cursor_h))];
    if serifs {
        for y in [CENTER - cursor_h, CENTER + cursor_h] {
            cursor.push(((cursor_x - 0.05, y), (cursor_x + 0.05, y)));
        }
    }

    // points of the top edge: the valleys at the even indexes (the ends included), the outer folds at the odd ones
    let points = 2 * folds;
    let x_at = |i: usize| X_NARROW + (X_WIDE - X_NARROW) * i as f32 / points as f32;
    let depth = |i: usize| if i % 2 == 1 { 1.0 } else { INNER };
    let top_edge = (0..=points).map(|i| (x_at(i), CENTER - depth(i) * half_height(x_at(i))));
    let bottom_edge = (0..=points).rev().map(|i| (x_at(i), CENTER + depth(i) * half_height(x_at(i))));
    let bellows = top_edge.chain(bottom_edge).collect();
    // a crease at every inner valley; it stops short of the edges, so the silhouette stays in one piece
    let creases = (1..points)
        .filter(|i| i % 2 == 0)
        .map(|i| {
            let (x, h) = (x_at(i), 0.7 * INNER * half_height(x_at(i)));
            ((x, CENTER - h), (x, CENTER + h))
        })
        .collect();
    Glyph { cursor, bellows, creases }
}

/// Even-odd test of a point against a polygon.
fn in_polygon(x: f32, y: f32, polygon: &[Point]) -> bool {
    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let ((xi, yi), (xj, yj)) = (polygon[i], polygon[j]);
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn in_rounded_rect(x: f32, y: f32, lo: f32, hi: f32, r: f32) -> bool {
    if x < lo || x > hi || y < lo || y > hi {
        return false;
    }
    let cx = x.clamp(lo + r, hi - r);
    let cy = y.clamp(lo + r, hi - r);
    (x - cx).powi(2) + (y - cy).powi(2) <= r * r
}

fn dist_to_segment(x: f32, y: f32, a: Point, b: Point) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 == 0.0 { 0.0 } else { (((x - a.0) * dx + (y - a.1) * dy) / len2).clamp(0.0, 1.0) };
    let (px, py) = (a.0 + t * dx, a.1 + t * dy);
    ((x - px).powi(2) + (y - py).powi(2)).sqrt()
}

/// Build a Windows `.ico` file containing the given sizes (32-bit BMP entries).
#[allow(dead_code)]
pub fn build_ico(sizes: &[u32]) -> Vec<u8> {
    let images: Vec<Vec<u8>> = sizes
        .iter()
        .map(|&s| {
            let rgba = render_rgba(s, true);
            let mut bmp = Vec::new();
            // BITMAPINFOHEADER (height is doubled: XOR mask + AND mask)
            bmp.extend_from_slice(&40u32.to_le_bytes());
            bmp.extend_from_slice(&(s as i32).to_le_bytes());
            bmp.extend_from_slice(&((s * 2) as i32).to_le_bytes());
            bmp.extend_from_slice(&1u16.to_le_bytes());
            bmp.extend_from_slice(&32u16.to_le_bytes());
            bmp.extend_from_slice(&0u32.to_le_bytes());
            bmp.extend_from_slice(&(s * s * 4).to_le_bytes());
            bmp.extend_from_slice(&[0u8; 16]);
            for y in (0..s).rev() {
                for x in 0..s {
                    let i = ((y * s + x) * 4) as usize;
                    bmp.extend_from_slice(&[rgba[i + 2], rgba[i + 1], rgba[i], rgba[i + 3]]);
                }
            }
            let mask_row = s.div_ceil(32) * 4;
            bmp.extend(std::iter::repeat_n(0u8, (mask_row * s) as usize));
            bmp
        })
        .collect();
    let mut ico = Vec::new();
    ico.extend_from_slice(&0u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&(sizes.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * sizes.len() as u32;
    for (s, img) in sizes.iter().zip(&images) {
        let dim = if *s >= 256 { 0u8 } else { *s as u8 };
        ico.extend_from_slice(&[dim, dim, 0, 0]);
        ico.extend_from_slice(&1u16.to_le_bytes());
        ico.extend_from_slice(&32u16.to_le_bytes());
        ico.extend_from_slice(&(img.len() as u32).to_le_bytes());
        ico.extend_from_slice(&offset.to_le_bytes());
        offset += img.len() as u32;
    }
    for img in images {
        ico.extend_from_slice(&img);
    }
    ico
}

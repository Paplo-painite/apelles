//! P2-B local mask and proxy helpers.
//!
//! The document model continues to use `apelles_doc::LayerMask` and the existing
//! `SmartObject::filter_mask`, so old `.pcraft` manifests remain readable.  This module
//! contains the UI-friendly editing representation and the bounded proxy entry point.

use serde::{Deserialize, Serialize};

/// A grayscale editing mask. Values are 0 (hide) through 255 (reveal).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mask {
    pub width: usize,
    pub height: usize,
    #[serde(default)]
    pub pixels: Vec<u8>,
}

impl Mask {
    pub fn reveal(width: usize, height: usize) -> Self {
        Self { width, height, pixels: vec![255; width.saturating_mul(height)] }
    }

    pub fn hide(width: usize, height: usize) -> Self {
        Self { width, height, pixels: vec![0; width.saturating_mul(height)] }
    }

    pub fn value(&self, x: usize, y: usize) -> u8 {
        if x >= self.width || y >= self.height { return 0; }
        self.pixels[y * self.width + x]
    }

    /// Paints a soft circular brush. `amount` is signed: positive paints white, negative erases.
    pub fn brush(&mut self, cx: f32, cy: f32, radius: f32, amount: f32) {
        if self.width == 0 || self.height == 0 || radius <= 0.0 { return; }
        let x0 = (cx - radius - 1.0).floor().max(0.0) as usize;
        let y0 = (cy - radius - 1.0).floor().max(0.0) as usize;
        let x1 = (cx + radius + 1.0).ceil().min(self.width as f32) as usize;
        let y1 = (cy + radius + 1.0).ceil().min(self.height as f32) as usize;
        let delta = amount.clamp(-1.0, 1.0);
        for y in y0..y1 {
            for x in x0..x1 {
                let d = ((x as f32 + 0.5 - cx).hypot(y as f32 + 0.5 - cy) / radius).clamp(0.0, 1.0);
                let coverage = 1.0 - d;
                let old = f32::from(self.pixels[y * self.width + x]) / 255.0;
                self.pixels[y * self.width + x] = ((old + delta * coverage).clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
    }

    /// Compact run-length encoding used for previews, clipboard state and tests.
    pub fn encode_rle(&self) -> Vec<(u32, u8)> {
        let mut out = Vec::new();
        for &v in &self.pixels {
            if let Some((n, last)) = out.last_mut() {
                if *last == v && *n < u32::MAX { *n += 1; continue; }
            }
            out.push((1, v));
        }
        out
    }

    pub fn decode_rle(width: usize, height: usize, runs: &[(u32, u8)]) -> Option<Self> {
        let want = width.checked_mul(height)?;
        let mut pixels = Vec::with_capacity(want);
        for &(n, v) in runs {
            let n = usize::try_from(n).ok()?;
            if pixels.len().checked_add(n)? > want { return None; }
            pixels.extend(std::iter::repeat(v).take(n));
        }
        (pixels.len() == want).then_some(Self { width, height, pixels })
    }
}

/// Parameters for a bounded proxy operation. The caller owns the full-resolution apply step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProxyPlan {
    pub width: usize,
    pub height: usize,
    pub scale: usize,
}

impl ProxyPlan {
    pub fn for_size(width: usize, height: usize, max_side: usize) -> Self {
        let scale = width.max(height).div_ceil(max_side.max(1)).max(1);
        Self { width: width.div_ceil(scale), height: height.div_ceil(scale), scale }
    }
}

/// Runs the existing local multi-scale PatchMatch completion on a bounded proxy.
/// `image` and `hole` must already be proxy-sized; cancellation discards the result.
#[allow(clippy::too_many_arguments)]
pub fn remove_proxy(
    width: usize,
    height: usize,
    channels: usize,
    image: &[f32],
    hole: &[bool],
    source: &[bool],
    ctl: &apelles_raster::Interrupt,
) -> Result<Vec<f32>, apelles_raster::Cancelled> {
    let opts = crate::content_aware::FillOptions { color_adaptation: crate::inpaint::Adapt::NONE, ..Default::default() };
    crate::content_aware::fill_with(width, height, channels, image, hole, source, &opts, ctl)
}

/// A small boundary blend that suppresses a hard patch edge after proxy upsampling.
pub fn seam_heal(width: usize, height: usize, channels: usize, image: &mut [f32], hole: &[bool], radius: usize) {
    if width == 0 || height == 0 || channels == 0 || radius == 0 { return; }
    let before = image.to_vec();
    for y in 0..height {
        for x in 0..width {
            let i = y * width + x;
            if !hole.get(i).copied().unwrap_or(false) { continue; }
            let edge = (x > 0 && !hole[i - 1]) || (x + 1 < width && !hole[i + 1]) || (y > 0 && !hole[i - width]) || (y + 1 < height && !hole[i + width]);
            if !edge { continue; }
            let mut sum = vec![0.0f32; channels];
            let mut n = 0.0;
            let r = radius as isize;
            for dy in -r..=r {
                for dx in -r..=r {
                    let xx = x as isize + dx;
                    let yy = y as isize + dy;
                    if xx < 0 || yy < 0 || xx >= width as isize || yy >= height as isize { continue; }
                    let j = yy as usize * width + xx as usize;
                    if !hole[j] {
                        for c in 0..channels { sum[c] += before[j * channels + c]; }
                        n += 1.0;
                    }
                }
            }
            if n > 0.0 {
                for c in 0..channels { image[i * channels + c] = (image[i * channels + c] * 0.7 + sum[c] / n * 0.3).clamp(0.0, 1.0); }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_create_save_load() {
        let mut m = Mask::hide(8, 4);
        m.brush(3.5, 1.5, 2.0, 1.0);
        let back = Mask::decode_rle(m.width, m.height, &m.encode_rle()).unwrap();
        assert_eq!(m, back);
        assert!(m.pixels.iter().any(|&v| v > 0));
    }

    #[test]
    fn mask_backward_compat_missing_pixels_defaults_to_empty() {
        let old = serde_json::json!({"width": 2, "height": 2});
        let m: Mask = serde_json::from_value(old).unwrap();
        assert!(m.pixels.is_empty());
    }

    #[test]
    fn proxy_plan_caps_long_side() {
        let p = ProxyPlan::for_size(4000, 2000, 1600);
        assert_eq!((p.width, p.height, p.scale), (1334, 667, 3));
    }

    #[test]
    fn seam_heal_changes_only_boundary_hole_pixels() {
        let mut img = vec![0.0f32; 5 * 5];
        let mut hole = vec![false; 25];
        for y in 1..4 { for x in 1..4 { hole[y * 5 + x] = true; img[y * 5 + x] = 1.0; } }
        seam_heal(5, 5, 1, &mut img, &hole, 1);
        assert!(img[2 * 5 + 2] > 0.99);
        assert_eq!(img[0], 0.0);
        assert!(img[1 * 5 + 1] < 1.0);
    }
}

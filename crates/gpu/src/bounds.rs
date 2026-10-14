//! Layer bounds exactly as `apelles-compose` computes them (`layer_bounds`), with the per-tile
//! content scan cached by copy-on-write tile identity, so an effect layer's region costs a hash
//! lookup per tile instead of a pixel scan per frame.

use apelles_doc::Layer;
use apelles_geom::Rect;
#[cfg(test)]
use apelles_raster::Surface;

pub use apelles_compose::bounds::content_bounds;

/// Bounds of a layer's own pixels: `apelles_compose::layer_bounds`.
pub fn layer_bounds(layer: &Layer, canvas: Rect) -> Rect {
    apelles_compose::layer_bounds(layer, canvas)
}

/// The region a layer's effect maps cover: its bounds grown by the effect reach, within the
/// canvas grown likewise (`apelles_compose::effect_maps`) when the layer reaches far beyond
/// it. A layer about the canvas's size keeps its whole region even where it overhangs the edge:
/// then the region keeps its size while the layer moves (a Move tool drag), so its maps move with
/// it instead of being rebuilt every frame (#761). The maps are exact either way.
pub fn effect_region(layer: &Layer, canvas: Rect) -> Rect {
    let m = apelles_compose::effects::margin(layer);
    let (whole, window) = (layer_bounds(layer, canvas).inflate(m), canvas.inflate(m));
    let area = |r: Rect| u64::from(r.width()) * u64::from(r.height());
    if area(whole) <= area(window).saturating_add(area(window) / 4) { whole } else { whole.intersect(&window) }
}

/// `apelles_compose::transparent_outside`.
pub fn transparent_outside(layer: &Layer) -> bool {
    apelles_compose::transparent_outside(layer)
}

/// `apelles_compose::composite_bounds`: where compositing the layer can change anything.
pub fn composite_bounds(layer: &Layer, canvas: Rect) -> Option<Rect> {
    apelles_compose::composite_bounds(layer, canvas)
}

#[cfg(test)]
mod tests {
    use super::*;
    use apelles_color::PixelFormat;

    #[test]
    fn matches_surface_content_bounds() {
        let mut s = Surface::new(PixelFormat::RGBA8);
        assert_eq!(content_bounds(&s), s.content_bounds());
        s.fill_rect(Rect::new(10, 300, 40, 700), &[1.0, 0.0, 0.0, 1.0]);
        s.fill_rect(Rect::new(-50, 5, -3, 9), &[0.0, 0.0, 1.0, 0.5]);
        assert_eq!(content_bounds(&s), s.content_bounds());
        // Cached a second time, and after a change.
        assert_eq!(content_bounds(&s), s.content_bounds());
        s.fill_rect(Rect::new(600, 600, 601, 601), &[0.0, 1.0, 0.0, 1.0]);
        assert_eq!(content_bounds(&s), s.content_bounds());
        let mut g = Surface::with_default(PixelFormat::GRAY8, &[1.0]);
        g.fill_rect(Rect::new(3, 3, 9, 9), &[0.5]);
        assert_eq!(content_bounds(&g), g.content_bounds());
    }
}

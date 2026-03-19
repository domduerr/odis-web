#![allow(dead_code)]

use bit_set::BitSet;
use odis::{
    algorithms::{fast_dimdraw::FastDimDraw, sugiyama::Sugiyama},
    Drawing, DrawingAlgorithm, Lattice,
};

pub struct Dimensions {
    pub width: f64,
    pub height: f64,
    pub margin: f64,
}

pub fn dimdraw_drawing(lattice: &Lattice<(BitSet, BitSet)>) -> Option<Drawing> {
    FastDimDraw { timeout_ms: 1000 }.draw(lattice)
}

pub fn sugiyama_drawing(lattice: &Lattice<(BitSet, BitSet)>) -> Option<Drawing> {
    Sugiyama { vertex_spacing: 1 }.draw(lattice)
}

pub fn scale_coordinates_to_viewport(
    coords: &[(f64, f64)],
    dimensions: Dimensions,
) -> Vec<(usize, f64, f64)> {
    if coords.is_empty() {
        return Vec::new();
    }

    let min_x = coords.iter().map(|c| c.0).fold(f64::INFINITY, f64::min);
    let max_x = coords
        .iter()
        .map(|c| c.0)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y = coords.iter().map(|c| c.1).fold(f64::INFINITY, f64::min);
    let max_y = coords
        .iter()
        .map(|c| c.1)
        .fold(f64::NEG_INFINITY, f64::max);

    let graph_width = max_x - min_x;
    let graph_height = max_y - min_y;

    let available_width = (dimensions.width - 2.0 * dimensions.margin).max(1.0);
    let available_height = (dimensions.height - 2.0 * dimensions.margin).max(1.0);

    let x_coef = if graph_width > 0.0 {
        available_width / graph_width
    } else {
        1.0
    };
    let y_coef = if graph_height > 0.0 {
        available_height / graph_height
    } else {
        1.0
    };

    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;

    let canvas_center_x = dimensions.width / 2.0;
    let canvas_center_y = dimensions.height / 2.0;

    let mut position_counts: std::collections::HashMap<(i32, i32), usize> =
        std::collections::HashMap::new();

    coords
        .iter()
        .enumerate()
        .map(|(idx, &(x, y))| {
            let scaled_x = (x - center_x) * x_coef + canvas_center_x;
            let scaled_y = (y - center_y) * y_coef + canvas_center_y;
            let pos_key = (scaled_x as i32, scaled_y as i32);
            let offset = *position_counts.entry(pos_key).or_insert(0) as f64 * 20.0;
            *position_counts.get_mut(&pos_key).unwrap() += 1;
            (idx, scaled_x + offset, scaled_y + offset)
        })
        .collect()
}

/// Scales a raw drawing into the current SVG viewport while preserving relative layout.
pub fn scale_drawing_to_viewport(
    drawing: &Drawing,
    dimensions: Dimensions,
) -> Vec<(usize, f64, f64)> {
    scale_coordinates_to_viewport(&drawing.coordinates, dimensions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_drawing_preserves_y_order() {
        let drawing = Drawing::new(vec![(0.0, 0.0), (0.0, 10.0)]);
        let dims = Dimensions {
            width: 200.0,
            height: 200.0,
            margin: 0.0,
        };

        let scaled = scale_drawing_to_viewport(&drawing, dims);
        assert_eq!(scaled.len(), 2);

        // Larger source y should stay larger after scaling.
        assert!(scaled[0].2 < scaled[1].2, "scaled coords: {:?}", scaled);
    }
}

#![allow(dead_code)]

use bit_set::BitSet;
use odis::{
    algorithms::{dimdraw::DimDraw, dimflux::DimFlux, sugiyama::Sugiyama},
    ConceptDrawingAlgorithm, Drawing, DrawingAlgorithm, Lattice,
};

pub fn dimdraw_drawing(lattice: &Lattice<(BitSet, BitSet)>) -> Option<Drawing> {
    DimDraw::default().draw(lattice)
}

/// DimFlux refines a DimDraw layout into an additive one, so it needs the
/// dimensions of the context the lattice came from.
pub fn dimflux_drawing(
    lattice: &Lattice<(BitSet, BitSet)>,
    object_count: usize,
    attribute_count: usize,
) -> Option<Drawing> {
    DimFlux::default().draw_lattice(lattice, object_count, attribute_count)
}

pub fn sugiyama_drawing(lattice: &Lattice<(BitSet, BitSet)>) -> Option<Drawing> {
    Sugiyama { vertex_spacing: 1 }.draw(lattice)
}

/// Thin helper: scale a raw `Drawing` to a viewport and return
/// the scaled coordinates as a parallel `Vec<(f64, f64)>`.
pub fn scale_drawing_to_viewport(
    drawing: &Drawing,
    width: f64,
    height: f64,
    margin: f64,
) -> Vec<(f64, f64)> {
    drawing.scale_to_viewport(width, height, margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_drawing_preserves_y_order() {
        let drawing = Drawing::new(vec![(0.0, 0.0), (0.0, 10.0)]);
        let scaled = scale_drawing_to_viewport(&drawing, 200.0, 200.0, 0.0);
        assert_eq!(scaled.len(), 2);
        // Larger source y should stay larger after scaling.
        assert!(scaled[0].1 < scaled[1].1, "scaled coords: {:?}", scaled);
    }
}

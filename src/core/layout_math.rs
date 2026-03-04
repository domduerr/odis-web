#![allow(dead_code)]

use core::f64;
use odis::algorithms::fast_dimdraw::fastdimdraw;

pub struct Dimensions {
    pub width: f64,
    pub height: f64,
    pub margin: f64,
}

pub fn compute_dimdraw_layout(
    edges: &[(u32, u32)],
    dimensions: Dimensions,
) -> Vec<(usize, f64, f64)> {
    if let Some(result) = fastdimdraw(edges, -1.0).first() {
        let coords = &result.drawing.coordinates;

        if coords.is_empty() {
            return Vec::new();
        }

        let min_x = coords.iter().map(|c| c.0).min().unwrap_or(0);
        let max_x = coords.iter().map(|c| c.0).max().unwrap_or(0);
        let min_y = coords.iter().map(|c| c.1).min().unwrap_or(0);
        let max_y = coords.iter().map(|c| c.1).max().unwrap_or(0);

        let graph_width = (max_x - min_x) as f64;
        let graph_height = (max_y - min_y) as f64;

        let available_width = dimensions.width - 2.0 * dimensions.margin;
        let available_height = dimensions.height - 2.0 * dimensions.margin;

        let x_coef = if graph_width > 0.0 {
            available_width / graph_width
        } else {
            0.0
        };
        let y_coef = if graph_height > 0.0 {
            available_height / graph_height
        } else {
            0.0
        };

        let center_x = (min_x + max_x) as f64 / 2.0;
        let center_y = (min_y + max_y) as f64 / 2.0;

        let canvas_center_x = dimensions.width / 2.0;
        let canvas_center_y = dimensions.height / 2.0;

        coords
            .iter()
            .enumerate()
            .map(|(idx, &(x, y))| {
                let scaled_x = if x_coef > 0.0 {
                    (x as f64 - center_x) * x_coef + canvas_center_x
                } else {
                    canvas_center_x
                };
                let scaled_y = if y_coef > 0.0 {
                    (y as f64 - center_y) * y_coef + canvas_center_y
                } else {
                    canvas_center_y
                };
                (idx, scaled_x, scaled_y)
            })
            .collect()
    } else {
        Vec::new()
    }
}

pub fn compute_sugiyama_layout(
    edges: &[(u32, u32)],
    dimensions: Dimensions,
    node_count: usize,
) -> Vec<(usize, f64, f64)> {
    let layouts = rust_sugiyama::from_edges(edges).vertex_spacing(1).build();

    if let Some((layout_points, _width, _height)) = layouts.first() {
        if layout_points.is_empty() {
            return Vec::new();
        }

        let xs: Vec<f64> = layout_points
            .iter()
            .map(|(_, (x, _))| x.abs() as f64)
            .collect();
        let ys: Vec<f64> = layout_points
            .iter()
            .map(|(_, (_, y))| y.abs() as f64)
            .collect();

        let min_x = xs.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_x = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_y = ys.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_y = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let graph_width = max_x - min_x;
        let graph_height = max_y - min_y;

        let available_width = dimensions.width - 2.0 * dimensions.margin;
        let available_height = dimensions.height - 2.0 * dimensions.margin;

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

        let mut positions = vec![(0, 0.0, 0.0); node_count];
        let mut position_counts: std::collections::HashMap<(i32, i32), usize> =
            std::collections::HashMap::new();

        for (node_id, (x, y)) in layout_points {
            let idx = *node_id as usize;
            if idx < positions.len() {
                let abs_x = x.abs() as f64;
                let abs_y = y.abs() as f64;
                let scaled_x = if x_coef > 0.0 {
                    (abs_x - center_x) * x_coef + canvas_center_x
                } else {
                    canvas_center_x
                };
                let scaled_y = if y_coef > 0.0 {
                    (abs_y - center_y) * y_coef + canvas_center_y
                } else {
                    canvas_center_y
                };
                let pos_key = (scaled_x as i32, scaled_y as i32);
                let offset = *position_counts.entry(pos_key).or_insert(0) as f64 * 20.0;
                *position_counts.get_mut(&pos_key).unwrap() += 1;
                positions[idx] = (idx, scaled_x + offset, scaled_y + offset);
            }
        }
        positions
    } else {
        Vec::new()
    }
}

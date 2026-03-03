use core::f64;

use bit_set::BitSet;
use leptos::{either::Either, prelude::*};
use odis::algorithms::fast_dimdraw::fastdimdraw;
use odis::{FormalContext, Lattice};

use crate::components::{
    svg::{edge::EdgeComp, node::NodeComp},
    svg_download::SvgDownloadComp,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayoutAlgorithm {
    Dimdraw,
    Sugiyama,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub id: usize,
    pub label: (Option<String>, Option<String>),
    pub x: f64,
    pub y: f64,
    pub x_signal: RwSignal<f64>,
    pub y_signal: RwSignal<f64>,
}

#[derive(Clone)]
pub struct Dimensions {
    pub width: f64,
    pub height: f64,
    pub margin: f64,
    pub radius: f64,
    pub font_size: u8,
}

impl Node {
    pub fn new(id: usize, label: (Option<String>, Option<String>), x: f64, y: f64) -> Self {
        Node {
            id,
            label,
            x,
            y,
            x_signal: RwSignal::new(x),
            y_signal: RwSignal::new(y),
        }
    }
}

#[component]
pub fn GraphComp(
    concepts: Vec<(BitSet, BitSet)>,
    context: FormalContext<String>,
    algorithm: LayoutAlgorithm,
) -> impl IntoView {
    let lattice_option = Lattice::from_index_concepts(&concepts, &context);

    let mut lattice = Lattice::new(odis::Order::new(odis::Graph::new(Vec::new(), Vec::new())));
    let mut error = " ";

    if let Some(n) = lattice_option {
        lattice = n;
    } else {
        error = "Cannot draw concept lattice from singular concept.";
    }

    let width_node_ref: NodeRef<leptos::html::Input> = NodeRef::new();
    let height_node_ref: NodeRef<leptos::html::Input> = NodeRef::new();

    let dimensions = RwSignal::new(Dimensions {
        width: 600.0,
        height: 600.0,
        margin: 70.0,
        radius: 8.0,
        font_size: 16,
    });

    let edges = lattice.order.graph.edges.clone();
    let graph_nodes = lattice.order.graph.nodes.clone();
    let edges_for_view = edges.clone();

    let graph_node: NodeRef<leptos::svg::Svg> = NodeRef::new();
    let offset = RwSignal::new(None);

    Effect::new(move || {
        let element: web_sys::SvgElement = graph_node.get().unwrap();
        let rect: web_sys::DomRect = element.get_bounding_client_rect();

        let scroll_x = window().scroll_x().unwrap();
        let scroll_y = window().scroll_y().unwrap();

        offset.set(Some((rect.x() + scroll_x, rect.y() + scroll_y)));
    });

    let nodes = RwSignal::new(Vec::new());

    Effect::new(move || {
        let width_input: web_sys::HtmlInputElement = width_node_ref.get().unwrap();
        width_input.set_value(&dimensions.get_untracked().width.to_string());

        let height_input: web_sys::HtmlInputElement = height_node_ref.get().unwrap();
        height_input.set_value(&dimensions.get_untracked().height.to_string());
    });

    let dimensions_initial = dimensions.get();
    let node_positions: Vec<(usize, f64, f64)> = match algorithm {
        LayoutAlgorithm::Dimdraw => {
            if let Some(result) = fastdimdraw(&edges, -1.0).first() {
                let coords = &result.drawing.coordinates;

                let min_x = coords.iter().map(|c| c.0).min().unwrap_or(0);
                let max_x = coords.iter().map(|c| c.0).max().unwrap_or(0);
                let min_y = coords.iter().map(|c| c.1).min().unwrap_or(0);
                let max_y = coords.iter().map(|c| c.1).max().unwrap_or(0);

                let graph_width = (max_x - min_x) as f64;
                let graph_height = (max_y - min_y) as f64;

                let available_width = dimensions_initial.width - 2.0 * dimensions_initial.margin;
                let available_height = dimensions_initial.height - 2.0 * dimensions_initial.margin;

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

                let canvas_center_x = dimensions_initial.width / 2.0;
                let canvas_center_y = dimensions_initial.height / 2.0;

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
        LayoutAlgorithm::Sugiyama => {
            let layouts = rust_sugiyama::from_edges(&edges).vertex_spacing(1).build();
            if let Some((layout_points, _width, _height)) = layouts.first() {
                if layout_points.is_empty() {
                    Vec::new()
                } else {
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

                    let available_width =
                        dimensions_initial.width - 2.0 * dimensions_initial.margin;
                    let available_height =
                        dimensions_initial.height - 2.0 * dimensions_initial.margin;

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

                    let canvas_center_x = dimensions_initial.width / 2.0;
                    let canvas_center_y = dimensions_initial.height / 2.0;

                    let mut positions = vec![(0, 0.0, 0.0); graph_nodes.len()];
                    let mut position_counts: std::collections::HashMap<(i32, i32), usize> =
                        std::collections::HashMap::new();
                    for (node_id, (x, y)) in layout_points {
                        let idx = *node_id as usize;
                        if idx < positions.len() {
                            let abs_x = x.abs() as f64;
                            let abs_y = y.abs() as f64;
                            let scaled_x = (abs_x - center_x) * x_coef + canvas_center_x;
                            let scaled_y = (abs_y - center_y) * y_coef + canvas_center_y;
                            let pos_key = (scaled_x as i32, scaled_y as i32);
                            let offset = *position_counts.entry(pos_key).or_insert(0) as f64 * 20.0;
                            *position_counts.get_mut(&pos_key).unwrap() += 1;
                            positions[idx] = (idx, scaled_x + offset, scaled_y + offset);
                        }
                    }
                    positions
                }
            } else {
                Vec::new()
            }
        }
    };

    nodes.set(
        graph_nodes
            .iter()
            .map(|node| {
                let pos = node_positions.iter().find(|(id, _, _)| *id == node.id);
                if let Some((_, x, y)) = pos {
                    Node::new(node.id, node.label.clone(), *x, *y)
                } else {
                    Node::new(
                        node.id,
                        node.label.clone(),
                        dimensions_initial.width / 2.0,
                        dimensions_initial.margin,
                    )
                }
            })
            .collect(),
    );

    Effect::new(move || {
        let dimensions = dimensions.get();

        let node_positions_scaled: Vec<(usize, f64, f64)> = match algorithm {
            LayoutAlgorithm::Dimdraw => {
                if let Some(result) = fastdimdraw(&edges, -1.0).first() {
                    let coords = &result.drawing.coordinates;

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
            LayoutAlgorithm::Sugiyama => {
                let layouts = rust_sugiyama::from_edges(&edges).vertex_spacing(1).build();
                if let Some((layout_points, _width, _height)) = layouts.first() {
                    if layout_points.is_empty() {
                        Vec::new()
                    } else {
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
                            0.0
                        };
                        let y_coef = if graph_height > 0.0 {
                            available_height / graph_height
                        } else {
                            0.0
                        };

                        let center_x = (min_x + max_x) / 2.0;
                        let center_y = (min_y + max_y) / 2.0;

                        let canvas_center_x = dimensions.width / 2.0;
                        let canvas_center_y = dimensions.height / 2.0;

                        let mut positions = vec![(0, 0.0, 0.0); graph_nodes.len()];
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
                                let offset =
                                    *position_counts.entry(pos_key).or_insert(0) as f64 * 20.0;
                                *position_counts.get_mut(&pos_key).unwrap() += 1;
                                positions[idx] = (idx, scaled_x + offset, scaled_y + offset);
                            }
                        }
                        positions
                    }
                } else {
                    Vec::new()
                }
            }
        };

        nodes.set(
            graph_nodes
                .iter()
                .map(|node| {
                    let pos = node_positions_scaled
                        .iter()
                        .find(|(id, _, _)| *id == node.id);
                    if let Some((_, x, y)) = pos {
                        Node::new(node.id, node.label.clone(), *x, *y)
                    } else {
                        Node::new(
                            node.id,
                            node.label.clone(),
                            dimensions.width / 2.0,
                            dimensions.margin,
                        )
                    }
                })
                .collect(),
        );
    });

    view! {
        <div class="bg-white border border-dhbw-gray-25 rounded p-4">
            <div class="mb-4">
                <div class="flex items-center gap-4 mb-2">
                    <label class="text-dhbw-gray font-medium text-sm">Width of concept lattice:</label>
                    <input
                        id="width"
                        node_ref=width_node_ref
                        type="range"
                        value="600"
                        min={2.0 * dimensions.get().margin}
                        max="1000"
                        on:input=move |_| {
                            dimensions.update(|dimen| {
                                let width_input: web_sys::HtmlInputElement = width_node_ref.get().unwrap();
                                dimen.width = width_input.value().parse().unwrap();
                            });
                        }
                        class="flex-1 max-w-xs h-2 bg-dhbw-gray-5 rounded-lg appearance-none cursor-pointer"
                    />
                    <span class="text-sm text-dhbw-gray-50 w-12">{move || dimensions.get().width.to_string()}</span>
                </div>
                <div class="flex items-center gap-4">
                    <label class="text-dhbw-gray font-medium text-sm">Height of concept lattice:</label>
                    <input
                        id="height"
                        node_ref=height_node_ref
                        type="range"
                        value="600"
                        min={2.0 * dimensions.get().margin}
                        max="1000"
                        on:input=move |_| {
                            dimensions.update(|dimen| {
                                let height_input: web_sys::HtmlInputElement = height_node_ref.get().unwrap();
                                dimen.height = height_input.value().parse().unwrap();
                            });
                        }
                        class="flex-1 max-w-xs h-2 bg-dhbw-gray-5 rounded-lg appearance-none cursor-pointer"
                    />
                    <span class="text-sm text-dhbw-gray-50 w-12">{move || dimensions.get().height.to_string()}</span>
                </div>
            </div>

            <SvgDownloadComp node_ref=graph_node/>

            <div
                class="overflow-auto border border-dhbw-gray-25 rounded"
                style:width=move || {format!("{}px", dimensions.get().width)}
                style:max-width="100%"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    style:width=move || {format!("{}px", dimensions.get().width)}
                    style:height=move || {format!("{}px", dimensions.get().height)}
                    node_ref=graph_node
                    class="bg-white"
                >
                    <rect
                        width="100%"
                        height="100%"
                        x="0"
                        y="0"
                        fill="transparent"
                        stroke-width="2"
                        stroke="#5C697133"
                    />
                    {move || {
                        let nodes = nodes.get();
                        if let Some(off) = offset.get() {
                            Either::Left(view! {
                                {
                                    edges_for_view.iter().map(|edge| {
                                        let start = nodes.iter().position(|x| x.id == edge.0 as usize).unwrap();
                                        let end = nodes.iter().position(|x| x.id == edge.1 as usize).unwrap();
                                        view! {
                                            <EdgeComp
                                                start=(nodes[start].x_signal, nodes[start].y_signal)
                                                end=(nodes[end].x_signal, nodes[end].y_signal)
                                            />
                                        }
                                    }).collect_view()
                                }
                                {
                                    nodes.iter().map(|node| {
                                        view! {
                                            <NodeComp
                                                node=node.clone()
                                                offset=off
                                                dimensions=dimensions.get()
                                            />
                                        }
                                    }).collect_view()
                                }
                            })
                        } else {
                            Either::Right(view! {})
                        }
                    }}
                    {move || {
                        if let " " = error {
                            Either::Left(())
                        } else {
                            Either::Right(
                                view! {
                                    <rect width="100%" height="100%" x="0" y="0" fill="white" stroke-width="3" stroke="#E2001A"/>
                                    <text
                                        font-size=dimensions.get().font_size as f64 * 1.6
                                        dy=".35em"
                                        text-anchor="middle"
                                        stroke-width="0.3em"
                                        font-family="monospace"
                                        x=dimensions.get().height / 2.0
                                        y=dimensions.get().width / 2.0
                                    >{error}</text>
                                }
                            )
                        }
                    }}
                </svg>
            </div>
        </div>
    }
}

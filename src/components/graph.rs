use core::f64;

use bit_set::BitSet;
use leptos::{either::Either, prelude::*};
use odis::{FormalContext, Lattice};

use crate::components::{
    svg::{edge::EdgeComp, node::NodeComp},
    svg_download::SvgDownloadComp,
};
use crate::core::layout_math::{
    compute_dimdraw_layout, compute_sugiyama_layout, Dimensions as LayoutDimensions,
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
        if let Some(element) = graph_node.get() {
            let rect: web_sys::DomRect = element.get_bounding_client_rect();
            let scroll_x = window().scroll_x().unwrap_or(0.0);
            let scroll_y = window().scroll_y().unwrap_or(0.0);
            offset.set(Some((rect.x() + scroll_x, rect.y() + scroll_y)));
        }
    });

    let nodes = RwSignal::new(Vec::new());

    Effect::new(move || {
        let width_input: web_sys::HtmlInputElement = width_node_ref.get().unwrap();
        width_input.set_value(&dimensions.get_untracked().width.to_string());

        let height_input: web_sys::HtmlInputElement = height_node_ref.get().unwrap();
        height_input.set_value(&dimensions.get_untracked().height.to_string());
    });

    let dimensions_initial = dimensions.get();
    let layout_dims = LayoutDimensions {
        width: dimensions_initial.width,
        height: dimensions_initial.height,
        margin: dimensions_initial.margin,
    };

    let node_positions: Vec<(usize, f64, f64)> = match algorithm {
        LayoutAlgorithm::Dimdraw => compute_dimdraw_layout(&edges, layout_dims),
        LayoutAlgorithm::Sugiyama => {
            compute_sugiyama_layout(&edges, layout_dims, graph_nodes.len())
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

    let edges_clone = edges.clone();
    let graph_nodes_clone = graph_nodes.clone();

    Effect::new(move || {
        let dims = dimensions.get();
        let layout_dims = LayoutDimensions {
            width: dims.width,
            height: dims.height,
            margin: dims.margin,
        };

        let node_positions_scaled: Vec<(usize, f64, f64)> = match algorithm {
            LayoutAlgorithm::Dimdraw => compute_dimdraw_layout(&edges_clone, layout_dims),
            LayoutAlgorithm::Sugiyama => {
                compute_sugiyama_layout(&edges_clone, layout_dims, graph_nodes_clone.len())
            }
        };

        nodes.set(
            graph_nodes_clone
                .iter()
                .map(|node| {
                    let pos = node_positions_scaled
                        .iter()
                        .find(|(id, _, _)| *id == node.id);
                    if let Some((_, x, y)) = pos {
                        Node::new(node.id, node.label.clone(), *x, *y)
                    } else {
                        Node::new(node.id, node.label.clone(), dims.width / 2.0, dims.margin)
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
                style:width=move || format!("{}px", dimensions.get().width)
                style:max-width="100%"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    style:width=move || format!("{}px", dimensions.get().width)
                    style:height=move || format!("{}px", dimensions.get().height)
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
                        if error == " " {
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

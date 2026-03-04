use core::f64;

use bit_set::BitSet;
use leptos::wasm_bindgen::closure::Closure;
use leptos::wasm_bindgen::JsCast;
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
    DimDraw,
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
    layout_algorithm: RwSignal<LayoutAlgorithm>,
) -> impl IntoView {
    let algorithm = layout_algorithm.get();
    let lattice_option = Lattice::from_index_concepts(&concepts, &context);

    let mut lattice = Lattice::new(odis::Order::new(odis::Graph::new(Vec::new(), Vec::new())));
    let mut error = " ";

    if let Some(n) = lattice_option {
        lattice = n;
    } else {
        error = "Cannot draw concept lattice from singular concept.";
    }

    let dimensions = RwSignal::new(Dimensions {
        width: 600.0,
        height: 600.0,
        margin: 70.0,
        radius: 8.0,
        font_size: 16,
    });

    let width_input_ref = NodeRef::<leptos::html::Input>::new();
    let height_input_ref = NodeRef::<leptos::html::Input>::new();

    let is_resizing = RwSignal::new(false);
    let resize_type = RwSignal::new(0i32);
    let resize_start_x = RwSignal::new(0.0);
    let resize_start_y = RwSignal::new(0.0);
    let resize_start_width = RwSignal::new(0.0);
    let resize_start_height = RwSignal::new(0.0);

    let on_width_change = move |ev: leptos::ev::Event| {
        let target: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into();
        if let Ok(val) = target.value().parse::<f64>() {
            dimensions.update(|d: &mut Dimensions| {
                d.width = val.clamp(200.0, 2000.0);
            });
        }
    };

    let on_height_change = move |ev: leptos::ev::Event| {
        let target: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into();
        if let Ok(val) = target.value().parse::<f64>() {
            dimensions.update(|d: &mut Dimensions| {
                d.height = val.clamp(200.0, 2000.0);
            });
        }
    };

    let on_mouse_down_width = move |ev: leptos::ev::MouseEvent| {
        is_resizing.set(true);
        resize_type.set(1);
        resize_start_x.set(ev.client_x() as f64);
        resize_start_width.set(dimensions.get_untracked().width);
        let _ = ev.prevent_default();
    };

    let on_mouse_down_height = move |ev: leptos::ev::MouseEvent| {
        is_resizing.set(true);
        resize_type.set(2);
        resize_start_y.set(ev.client_y() as f64);
        resize_start_height.set(dimensions.get_untracked().height);
        let _ = ev.prevent_default();
    };

    let on_mouse_down_corner = move |ev: leptos::ev::MouseEvent| {
        is_resizing.set(true);
        resize_type.set(3);
        resize_start_x.set(ev.client_x() as f64);
        resize_start_y.set(ev.client_y() as f64);
        resize_start_width.set(dimensions.get_untracked().width);
        resize_start_height.set(dimensions.get_untracked().height);
        let _ = ev.prevent_default();
    };

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

    let dimensions_initial = dimensions.get();
    let layout_dims = LayoutDimensions {
        width: dimensions_initial.width,
        height: dimensions_initial.height,
        margin: dimensions_initial.margin,
    };

    let node_positions: Vec<(usize, f64, f64)> = match algorithm {
        LayoutAlgorithm::DimDraw => compute_dimdraw_layout(&edges, layout_dims),
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
            LayoutAlgorithm::DimDraw => compute_dimdraw_layout(&edges_clone, layout_dims),
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

    let width_input_ref2 = width_input_ref.clone();
    let height_input_ref2 = height_input_ref.clone();
    let is_resizing2 = is_resizing.clone();
    let resize_type2 = resize_type.clone();
    let resize_start_x2 = resize_start_x.clone();
    let resize_start_y2 = resize_start_y.clone();
    let resize_start_width2 = resize_start_width.clone();
    let resize_start_height2 = resize_start_height.clone();
    let dimensions2 = dimensions.clone();

    let document = window().document().unwrap();
    let body = document.body().unwrap();

    let move_closure = Closure::wrap(Box::new(move |ev: web_sys::MouseEvent| {
        if is_resizing2.get() {
            let rtype = resize_type2.get();

            if rtype == 1 {
                let delta = ev.client_x() as f64 - resize_start_x2.get();
                let new_width = (resize_start_width2.get() + delta).clamp(200.0, 2000.0);
                dimensions2.update(|d| {
                    d.width = new_width;
                });
                if let Some(input) = width_input_ref2.get() {
                    input.set_value(&new_width.to_string());
                }
            } else if rtype == 2 {
                let delta = ev.client_y() as f64 - resize_start_y2.get();
                let new_height = (resize_start_height2.get() + delta).clamp(200.0, 2000.0);
                dimensions2.update(|d| {
                    d.height = new_height;
                });
                if let Some(input) = height_input_ref2.get() {
                    input.set_value(&new_height.to_string());
                }
            } else if rtype == 3 {
                let delta_x = ev.client_x() as f64 - resize_start_x2.get();
                let delta_y = ev.client_y() as f64 - resize_start_y2.get();
                let avg_delta = (delta_x + delta_y) / 2.0;
                let new_width = (resize_start_width2.get() + avg_delta).clamp(200.0, 2000.0);
                let new_height = (resize_start_height2.get() + avg_delta).clamp(200.0, 2000.0);
                dimensions2.update(|d| {
                    d.width = new_width;
                    d.height = new_height;
                });
                if let Some(input) = width_input_ref2.get() {
                    input.set_value(&new_width.to_string());
                }
                if let Some(input) = height_input_ref2.get() {
                    input.set_value(&new_height.to_string());
                }
            }
        }
    }) as Box<dyn FnMut(_)>);

    let up_closure = Closure::wrap(Box::new(move |_ev: web_sys::MouseEvent| {
        is_resizing.set(false);
        resize_type.set(0);
    }) as Box<dyn FnMut(_)>);

    let _ = document
        .add_event_listener_with_callback("mousemove", move_closure.as_ref().unchecked_ref());
    let _ =
        document.add_event_listener_with_callback("mouseup", up_closure.as_ref().unchecked_ref());
    let _ =
        body.add_event_listener_with_callback("mousemove", move_closure.as_ref().unchecked_ref());
    let _ = body.add_event_listener_with_callback("mouseup", up_closure.as_ref().unchecked_ref());

    move_closure.forget();
    up_closure.forget();

    view! {
        <div class="flex items-start">
            <div class="relative">
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
                        fill="white"
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

                <div
                    class="absolute top-0 right-0 w-6 h-full cursor-ew-resize hover:bg-dhbw-red-20 transition-colors flex items-center justify-center"
                    on:mousedown=on_mouse_down_width
                >
                    <div class="w-1 h-8 bg-dhbw-gray-30 rounded"></div>
                </div>
                <div
                    class="absolute bottom-0 left-0 w-full h-6 cursor-ns-resize hover:bg-dhbw-red-20 transition-colors flex items-center justify-center"
                    on:mousedown=on_mouse_down_height
                >
                    <div class="w-8 h-1 bg-dhbw-gray-30 rounded"></div>
                </div>
                <div
                    class="absolute bottom-0 right-0 w-8 h-8 cursor-nwse-resize flex items-center justify-center"
                    on:mousedown=on_mouse_down_corner
                >
                </div>
            </div>

            <div class="flex flex-col gap-4 ml-auto min-w-[180px] bg-gray-50 p-4 rounded-lg border border-dhbw-gray-25">
                <div class="flex flex-col gap-1">
                    <label class="text-xs text-dhbw-gray-50 uppercase tracking-wide font-medium">Layout</label>
                    <select
                        on:change=move |ev| {
                            let select: web_sys::HtmlSelectElement = ev.target().unwrap().unchecked_into();
                            let value = select.value();
                            let algorithm = match value.as_str() {
                                "Sugiyama" => LayoutAlgorithm::Sugiyama,
                                _ => LayoutAlgorithm::DimDraw,
                            };
                            layout_algorithm.set(algorithm);
                        }
                        class="w-full px-3 py-2 border border-dhbw-gray-25 rounded text-sm text-dhbw-gray focus:outline-none focus:border-dhbw-red"
                    >
                        <option value="DimDraw" selected=move || layout_algorithm.get() == LayoutAlgorithm::DimDraw>DimDraw</option>
                        <option value="Sugiyama" selected=move || layout_algorithm.get() == LayoutAlgorithm::Sugiyama>Sugiyama</option>
                    </select>
                </div>
                <div class="flex flex-col gap-1">
                    <label class="text-xs text-dhbw-gray-50 uppercase tracking-wide font-medium">Width</label>
                    <input
                        type="number"
                        node_ref=width_input_ref
                        value="600"
                        min="200"
                        max="2000"
                        on:change=on_width_change
                        class="w-full px-3 py-2 border border-dhbw-gray-25 rounded text-sm text-dhbw-gray focus:outline-none focus:border-dhbw-red"
                    />
                </div>
                <div class="flex flex-col gap-1">
                    <label class="text-xs text-dhbw-gray-50 uppercase tracking-wide font-medium">Height</label>
                    <input
                        type="number"
                        node_ref=height_input_ref
                        value="600"
                        min="200"
                        max="2000"
                        on:change=on_height_change
                        class="w-full px-3 py-2 border border-dhbw-gray-25 rounded text-sm text-dhbw-gray focus:outline-none focus:border-dhbw-red"
                    />
                </div>
                <div class="mt-2">
                    <SvgDownloadComp node_ref=graph_node/>
                </div>
            </div>
        </div>
    }
}

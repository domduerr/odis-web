use bit_set::BitSet;
use leptos::wasm_bindgen::JsCast;
use leptos::{either::Either, prelude::*};
use odis::{Drawing, FormalContext, Lattice};
use std::rc::Rc;

use crate::components::{
    svg::{edge::EdgeComp, node::NodeComp},
    svg_download::SvgDownloadComp,
};
use crate::core::layout_math::{dimdraw_drawing, dimflux_drawing, sugiyama_drawing};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayoutAlgorithm {
    DimDraw,
    DimFlux,
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

#[derive(Clone, Debug)]
struct GraphNode {
    id: usize,
    label: (Option<String>, Option<String>),
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

fn compute_raw_layout_coordinates(
    lattice: &Lattice<(BitSet, BitSet)>,
    algorithm: LayoutAlgorithm,
    object_count: usize,
    attribute_count: usize,
)
    -> Vec<(f64, f64)> {
    let drawing = match algorithm {
        LayoutAlgorithm::DimDraw => dimdraw_drawing(lattice),
        LayoutAlgorithm::DimFlux => dimflux_drawing(lattice, object_count, attribute_count),
        LayoutAlgorithm::Sugiyama => sugiyama_drawing(lattice),
    };

    drawing.map(|d| d.coordinates).unwrap_or_default()
}

fn nodes_from_positions(
    graph_nodes: &[GraphNode],
    node_positions: &[(f64, f64)],
    dimensions: &Dimensions,
) -> Vec<Node> {
    graph_nodes
        .iter()
        .map(|node| {
            if let Some(&(x, y)) = node_positions.get(node.id) {
                Node::new(node.id, node.label.clone(), x, y)
            } else {
                Node::new(
                    node.id,
                    node.label.clone(),
                    dimensions.width / 2.0,
                    dimensions.margin,
                )
            }
        })
        .collect()
}

#[component]
pub fn GraphComp(
    concepts: Vec<(BitSet, BitSet)>,
    context: FormalContext<String>,
    layout_algorithm: RwSignal<LayoutAlgorithm>,
) -> impl IntoView {
    let _concept_count = concepts.len();
    let object_count = context.objects.len();
    let attribute_count = context.attributes.len();
    let lattice_option = context.concept_lattice();
    let mut error = "";

    let (lattice, edges, graph_nodes) = if let Some(lattice) = lattice_option {
        let reduced = context.reduced_labels(&lattice);
        let graph_nodes = reduced
            .into_iter()
            .enumerate()
            .map(|(id, (obj_labels, attr_labels))| GraphNode {
                id,
                label: (
                    if obj_labels.is_empty() {
                        None
                    } else {
                        Some(obj_labels.join(", "))
                    },
                    if attr_labels.is_empty() {
                        None
                    } else {
                        Some(attr_labels.join(", "))
                    },
                ),
            })
            .collect::<Vec<_>>();
        let edges = lattice.poset.covering_edges.clone();
        (Some(Rc::new(lattice)), edges, graph_nodes)
    } else {
        error = "Cannot draw concept lattice from singular concept.";
        (None, Vec::new(), Vec::new())
    };

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
        ev.prevent_default();
    };

    let on_mouse_down_height = move |ev: leptos::ev::MouseEvent| {
        is_resizing.set(true);
        resize_type.set(2);
        resize_start_y.set(ev.client_y() as f64);
        resize_start_height.set(dimensions.get_untracked().height);
        ev.prevent_default();
    };

    let on_mouse_down_corner = move |ev: leptos::ev::MouseEvent| {
        is_resizing.set(true);
        resize_type.set(3);
        resize_start_x.set(ev.client_x() as f64);
        resize_start_y.set(ev.client_y() as f64);
        resize_start_width.set(dimensions.get_untracked().width);
        resize_start_height.set(dimensions.get_untracked().height);
        ev.prevent_default();
    };

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
    let layout_epoch = RwSignal::new(0u64);

    let dimensions_initial = dimensions.get_untracked();
    nodes.set(nodes_from_positions(&graph_nodes, &[], &dimensions_initial));

    let graph_nodes_clone = graph_nodes.clone();
    let lattice_for_layout = lattice.clone();
    let raw_layout_coordinates = RwSignal::new(if let Some(lattice_ref) = lattice_for_layout.as_ref()
    {
        compute_raw_layout_coordinates(
            lattice_ref,
            layout_algorithm.get_untracked(),
            object_count,
            attribute_count,
        )
    } else {
        Vec::new()
    });

    let raw_layout_coordinates_writer = raw_layout_coordinates;
    let lattice_for_layout_writer = lattice.clone();
    Effect::new(move || {
        let algo = layout_algorithm.get();
        let raw_coords = if let Some(lattice_ref) = lattice_for_layout_writer.as_ref() {
            compute_raw_layout_coordinates(lattice_ref, algo, object_count, attribute_count)
        } else {
            Vec::new()
        };
        raw_layout_coordinates_writer.set(raw_coords);
    });

    let raw_layout_coordinates_for_effect = raw_layout_coordinates;

    Effect::new(move || {
        let dims = dimensions.get();
        let raw_coords = raw_layout_coordinates_for_effect.get();
        let node_positions_scaled = Drawing::new(raw_coords)
            .scale_to_viewport(dims.width, dims.height, dims.margin);
        nodes.set(nodes_from_positions(
            &graph_nodes_clone,
            &node_positions_scaled,
            &dims,
        ));
        // NodeComp uses draggable hook state initialized from props; bump epoch so
        // algorithm/layout changes remount nodes and pick up fresh coordinates.
        layout_epoch.update(|epoch| *epoch = epoch.wrapping_add(1));
    });

    let width_input_ref2 = width_input_ref;
    let height_input_ref2 = height_input_ref;
    let is_resizing2 = is_resizing;
    let resize_type2 = resize_type;
    let resize_start_x2 = resize_start_x;
    let resize_start_y2 = resize_start_y;
    let resize_start_width2 = resize_start_width;
    let resize_start_height2 = resize_start_height;
    let dimensions2 = dimensions;

    let move_handle =
        window_event_listener(leptos::ev::mousemove, move |ev: leptos::ev::MouseEvent| {
            if is_resizing2.get_untracked() {
                let rtype = resize_type2.get_untracked();

                if rtype == 1 {
                    let delta = ev.client_x() as f64 - resize_start_x2.get_untracked();
                    let new_width =
                        (resize_start_width2.get_untracked() + delta).clamp(200.0, 2000.0);
                    dimensions2.update(|d| {
                        d.width = new_width;
                    });
                    if let Some(input) = width_input_ref2.get_untracked() {
                        input.set_value(&new_width.to_string());
                    }
                } else if rtype == 2 {
                    let delta = ev.client_y() as f64 - resize_start_y2.get_untracked();
                    let new_height =
                        (resize_start_height2.get_untracked() + delta).clamp(200.0, 2000.0);
                    dimensions2.update(|d| {
                        d.height = new_height;
                    });
                    if let Some(input) = height_input_ref2.get_untracked() {
                        input.set_value(&new_height.to_string());
                    }
                } else if rtype == 3 {
                    let delta_x = ev.client_x() as f64 - resize_start_x2.get_untracked();
                    let delta_y = ev.client_y() as f64 - resize_start_y2.get_untracked();
                    let avg_delta = (delta_x + delta_y) / 2.0;
                    let new_width =
                        (resize_start_width2.get_untracked() + avg_delta).clamp(200.0, 2000.0);
                    let new_height =
                        (resize_start_height2.get_untracked() + avg_delta).clamp(200.0, 2000.0);
                    dimensions2.update(|d| {
                        d.width = new_width;
                        d.height = new_height;
                    });
                    if let Some(input) = width_input_ref2.get_untracked() {
                        input.set_value(&new_width.to_string());
                    }
                    if let Some(input) = height_input_ref2.get_untracked() {
                        input.set_value(&new_height.to_string());
                    }
                }
            }
        });

    let up_handle =
        window_event_listener(leptos::ev::mouseup, move |_ev: leptos::ev::MouseEvent| {
            is_resizing.set(false);
            resize_type.set(0);
        });

    on_cleanup(move || {
        move_handle.remove();
        up_handle.remove();
    });

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
                        let current_nodes = nodes.get();
                        if let Some(off) = offset.get() {
                            Either::Left(view! {
                                {
                                    edges_for_view.iter().map(|edge| {
                                        let start = current_nodes.iter().position(|x| x.id == edge.0 as usize).unwrap();
                                        let end = current_nodes.iter().position(|x| x.id == edge.1 as usize).unwrap();
                                        view! {
                                            <EdgeComp
                                                start=(current_nodes[start].x_signal, current_nodes[start].y_signal)
                                                end=(current_nodes[end].x_signal, current_nodes[end].y_signal)
                                            />
                                        }
                                    }).collect_view()
                                }
                                <For
                                    each=move || {
                                        let epoch = layout_epoch.get();
                                        nodes
                                            .get()
                                            .into_iter()
                                            .map(|node| (epoch, node))
                                            .collect::<Vec<_>>()
                                    }
                                    key=|(layout_epoch, node)| (*layout_epoch, node.id)
                                    children=move |(_layout_epoch, node)| {
                                        view! {
                                            <NodeComp
                                                node=node
                                                offset=off
                                                dimensions=dimensions.get()
                                            />
                                        }
                                    }
                                />
                            })
                        } else {
                            Either::Right(())
                        }
                    }}
                    {move || {
                        if error.is_empty() {
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
                                "DimFlux" => LayoutAlgorithm::DimFlux,
                                _ => LayoutAlgorithm::DimDraw,
                            };
                            layout_algorithm.set(algorithm);
                        }
                        class="w-full px-3 py-2 border border-dhbw-gray-25 rounded text-sm text-dhbw-gray focus:outline-none focus:border-dhbw-red"
                    >
                        <option value="DimDraw" selected=move || layout_algorithm.get() == LayoutAlgorithm::DimDraw>DimDraw</option>
                        <option value="DimFlux" selected=move || layout_algorithm.get() == LayoutAlgorithm::DimFlux>DimFlux</option>
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

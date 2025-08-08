use core::f64;

use bit_set::BitSet;
use leptos::{either::Either, prelude::*};
use odis::FormalContext;

use crate::components::{
    svg::{edge::EdgeComp, node::NodeComp},
    svg_download::SvgDownloadComp,
};

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
pub fn GraphComp(concepts: Vec<(BitSet, BitSet)>, context: FormalContext<String>) -> impl IntoView {
    let graph_option = odis::Graph::from_concepts(&concepts, &context);

    let mut graph = odis::Graph::new();
    let mut error = " ";

    if let Some(n) = graph_option {
        graph = n;
    } else {
        error = "Cannot draw graph from singular concept.";
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

    let mut x_max = 0.0;
    let mut y_max = 0.0;
    for node in &graph.nodes {
        if node.x as f64 > x_max {
            x_max = node.x as f64;
        }
        if node.y as f64 > y_max {
            y_max = node.y as f64;
        }
    }

    let mut x_not_zero = false;
    if x_max > 0.0 {
        x_not_zero = true;
    }

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

    Effect::new(move || {
        let dimensions = dimensions.get();

        let x_coef = (dimensions.width - 2.0 * dimensions.margin) / x_max;
        let y_coef = (dimensions.height - 2.0 * dimensions.margin) / y_max;

        if x_not_zero {
            nodes.set(
                graph
                    .nodes
                    .iter()
                    .map(|node| {
                        Node::new(
                            node.id,
                            node.label.clone(),
                            node.x as f64 * x_coef + dimensions.margin,
                            node.y as f64 * y_coef + dimensions.margin,
                        )
                    })
                    .collect(),
            );
        } else {
            nodes.set(
                graph
                    .nodes
                    .iter()
                    .map(|node| {
                        Node::new(
                            node.id,
                            node.label.clone(),
                            dimensions.width / 2.0,
                            node.y as f64 * y_coef + dimensions.margin,
                        )
                    })
                    .collect(),
            );
        }
    });

    view! {
        <SvgDownloadComp node_ref=graph_node/>
        <div
            style:margin-top="20px"
            style:display="flex"
            style:align-items="center"
        >
            <label
                style:font-family="monospace"
                style:font-size="18px"
                style:white-space="pre"
            >"Control width of graph:  "</label>
            <input
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
            />
        </div>
        <div
            style:margin-top="10px"
            style:display="flex"
            style:align-items="center"
        >
            <label
                style:font-family="monospace"
                style:font-size="18px"
                style:white-space="pre"
            >"Control height of graph: "</label>
            <input
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
            />
        </div>

        <div
            style:width=move || {format!("{}px", dimensions.get().width)}
            style:height=move || {format!("{}px", dimensions.get().height)}
            style:margin-top="20px"
            style:margin-bottom="20px"
        >
        <svg
            xmlns="http://www.w3.org/2000/svg"
            style:width=move || {format!("{}px", dimensions.get().width)}
            style:height=move || {format!("{}px", dimensions.get().height)}
            // viewBox=move || {format!("0 0 {} {}", dimensions.get().width, dimensions.get().height)}
            node_ref=graph_node
        >
            <rect
                width="100%"
                height="100%"
                x="0"
                y="0"
                fill="transparent"
                stroke-width="3"
                stroke="red"
            />
            {move || {
                let nodes = nodes.get();
                if let Some(off) = offset.get() {
                    Either::Left(view! {
                        {
                            graph.edges.iter().map(|edge| {
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
                            <rect width="100%" height="100%" x="0" y="0" fill="white" stroke-width="3" stroke="red"/>
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
    }
}

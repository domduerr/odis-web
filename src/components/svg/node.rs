use crate::components::graph::{Dimensions, Node};
use leptos::{ev, prelude::*};
use leptos_use::{
    UseDraggableOptions, UseDraggableReturn, core::Position, use_draggable_with_options,
};
use web_sys::MouseEvent;

#[component]
pub fn NodeComp(node: Node, offset: (f64, f64), dimensions: Dimensions) -> impl IntoView {
    let node_ref: NodeRef<leptos::svg::G> = NodeRef::new();

    let UseDraggableReturn { x, y, .. } = use_draggable_with_options(
        node_ref,
        UseDraggableOptions::default()
            .initial_value(Position {
                x: node.x - dimensions.radius + offset.0,
                y: node.y - dimensions.radius + offset.1,
            })
            .prevent_default(true),
    );

    let scroll_offset = RwSignal::new((0.0, 0.0));

    let scroll = move |_: MouseEvent| {
        let scroll_x = window().scroll_x().unwrap();
        let scroll_y = window().scroll_y().unwrap();

        scroll_offset.set((scroll_x, scroll_y));
    };

    let handle = window_event_listener(ev::scroll, move |_| {
        let scroll_x = window().scroll_x().unwrap();
        let scroll_y = window().scroll_y().unwrap();

        scroll_offset.set((scroll_x, scroll_y));
    });
    on_cleanup(move || handle.remove());

    // border collision x
    let x_pos = move || {
        let pos = x.get() + dimensions.radius - offset.0 + scroll_offset.get_untracked().0;
        if pos > dimensions.width - dimensions.radius {
            node.x_signal.set(dimensions.width - dimensions.radius);
            return dimensions.width - dimensions.radius;
        } else if pos < dimensions.radius {
            node.x_signal.set(dimensions.radius);
            return dimensions.radius;
        } else {
            node.x_signal.set(pos);
            return pos;
        }
    };

    // border collision y
    let y_pos = move || {
        let pos = y.get() + dimensions.radius - offset.1 + scroll_offset.get_untracked().1;
        if pos > dimensions.height - dimensions.radius {
            node.y_signal.set(dimensions.height - dimensions.radius);
            return dimensions.height - dimensions.radius;
        } else if pos < dimensions.radius {
            node.y_signal.set(dimensions.radius);
            return dimensions.radius;
        } else {
            node.y_signal.set(pos);
            return pos;
        }
    };

    view! {
        <g
            class="prevent-select"
        >
            <g
                node_ref=node_ref
            >
                <circle
                    fill="white"
                    stroke="black"
                    stroke-width="2"
                    r=dimensions.radius
                    cx=x_pos
                    cy=y_pos
                    on:mouseover=scroll
                />
            </g>

            // object labels
            <text
                font-size=dimensions.font_size
                dy=".35em"
                text-anchor="middle"
                stroke="white"
                stroke-width="0.3em"
                font-family="monospace"
                x=x_pos
                y=move || {y_pos() + dimensions.radius * 2.8}
            >{
                if let Some(obj) = node.label.0.clone() {
                    let len = obj.len();
                    let string = "0".to_string().repeat(len);
                    string
                } else {
                    "".to_string()
                }
            }</text>
            <text
                font-size=dimensions.font_size
                dy=".35em"
                text-anchor="middle"
                stroke-width="2"
                fill="black"
                font-family="monospace"
                x=x_pos
                y=move || {y_pos() + dimensions.radius * 2.8}
            >{
                if let Some(obj) = node.label.0 {
                    obj
                } else {
                    "".to_string()
                }
            }</text>

            // attribute labels
            <text
                font-size=dimensions.font_size
                dy=".35em"
                text-anchor="middle"
                stroke="white"
                stroke-width="0.3em"
                font-style="italic"
                font-family="monospace"
                x=x_pos
                y=move || {y_pos() - dimensions.radius * 2.8}
            >{
                if let Some(attr) = node.label.1.clone() {
                    let len = attr.len();
                    let string = "0".to_string().repeat(len);
                    string
                } else {
                    "".to_string()
                }
            }</text>
            <text
                font-size=dimensions.font_size
                dy=".35em"
                text-anchor="middle"
                stroke-width="2"
                fill="black"
                font-style="italic"
                font-family="monospace"
                x=x_pos
                y=move || {y_pos() - dimensions.radius * 2.8}
            >{
                if let Some(attr) = node.label.1 {
                    attr
                } else {
                    "".to_string()
                }
            }</text>
        </g>
    }
}

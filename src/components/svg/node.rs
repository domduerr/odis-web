use crate::components::graph::{Dimensions, Node};
use leptos::{ev, prelude::*};
use leptos_use::{
    UseDraggableOptions, UseDraggableReturn, core::Position, use_draggable_with_options,
};
use web_sys::MouseEvent;

// pub fn format_label(label_vec: Vec<String>, dimensions: &Dimensions) -> String {
//     // 20px: 11 / 23.04
//     // 18px: 9.91 / 20.95
//     // 16px: 8.8 / 18.85
//     // 14px: 7.7 / 16.06
//     let font_char_width = 8.8;
//     let font_char_height = 18.85;
//     let x_coef = (dimensions.width - 2.0 * dimensions.margin) / dimensions.node_count_xy.0;
//     let y_coef = (dimensions.height - 2.0 * dimensions.margin) / dimensions.node_count_xy.1;
//     let max_char_per_line = (x_coef / font_char_width).ceil();
//     let max_lines = (y_coef / font_char_height).ceil();
//     leptos::logging::log!("{}", max_char_per_line);
//     leptos::logging::log!("{}", max_lines);
//     String::from("value")
// }

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
                text-anchor="middle"
                stroke="white"
                stroke-width="0.4em"
                font-family="monospace"
                x=x_pos
                y=move || {y_pos() + dimensions.radius * 3.0}
            >{
                if let Some(obj_vec) = node.label.0.clone() {
                    let mut len = 0;
                    len += obj_vec.len() - 1;
                    for obj in obj_vec {
                        len += obj.len();
                    }
                    let string = "N".to_string().repeat(len);
                    string
                } else {
                    "".to_string()
                }
            }</text>
            <text
                font-size=dimensions.font_size
                text-anchor="middle"
                stroke-width="2"
                fill="black"
                font-family="monospace"
                x=x_pos
                y=move || {y_pos() + dimensions.radius * 3.0}
            >{
                if let Some(obj_vec) = node.label.0 {
                    let mut obj_string = String::new();
                    for obj in obj_vec {
                        obj_string.push_str(&(obj + " "));
                    }
                    obj_string.pop();
                    obj_string
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
                stroke-width="0.4em"
                font-style="italic"
                font-family="monospace"
                x=x_pos
                y=move || {y_pos() - dimensions.radius * 3.0}
            >{
                if let Some(attr_vec) = node.label.1.clone() {
                    let mut len = 0;
                    len += attr_vec.len() - 1;
                    for attr in attr_vec {
                        len += attr.len();
                    }
                    let string = "N".to_string().repeat(len);
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
                y=move || {y_pos() - dimensions.radius * 3.0}
            >{
                if let Some(attr_vec) = node.label.1 {
                    let mut attr_string = String::new();
                    for attr in attr_vec {
                        attr_string.push_str(&(attr + " "));
                    }
                    attr_string.pop();
                    attr_string
                } else {
                    "".to_string()
                }
            }</text>
        </g>
    }
}

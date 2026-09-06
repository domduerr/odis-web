use crate::components::graph::{Dimensions, Node};
use crate::components::ui::SVG_FONT;
use leptos::wasm_bindgen::JsCast;
use leptos::{ev, prelude::*};
use leptos_use::{
    UseDraggableOptions, UseDraggableReturn, core::Position, use_draggable_with_options,
};
use web_sys::MouseEvent;

/// Gap kept between a label and the edge of the drawing.
const LABEL_PADDING: f64 = 4.0;

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
            dimensions.width - dimensions.radius
        } else if pos < dimensions.radius {
            node.x_signal.set(dimensions.radius);
            dimensions.radius
        } else {
            node.x_signal.set(pos);
            pos
        }
    };

    // border collision y
    let y_pos = move || {
        let pos = y.get() + dimensions.radius - offset.1 + scroll_offset.get_untracked().1;
        if pos > dimensions.height - dimensions.radius {
            node.y_signal.set(dimensions.height - dimensions.radius);
            dimensions.height - dimensions.radius
        } else if pos < dimensions.radius {
            node.y_signal.set(dimensions.radius);
            dimensions.radius
        } else {
            node.y_signal.set(pos);
            pos
        }
    };

    // A label is far wider than the node it belongs to, so a node near the
    // border would push its label off the drawing. Both labels are measured
    // once they are on screen and then held inside the canvas.
    let object_label_ref: NodeRef<leptos::svg::Text> = NodeRef::new();
    let attribute_label_ref: NodeRef<leptos::svg::Text> = NodeRef::new();
    let object_label_width = RwSignal::new(0.0);
    let attribute_label_width = RwSignal::new(0.0);

    Effect::new(move |_| {
        if let Some(el) = object_label_ref.get() {
            object_label_width.set(text_width(&el));
        }
    });
    Effect::new(move |_| {
        if let Some(el) = attribute_label_ref.get() {
            attribute_label_width.set(text_width(&el));
        }
    });

    let label_x = move |width: f64| {
        let half = width / 2.0 + LABEL_PADDING;
        if 2.0 * half >= dimensions.width {
            dimensions.width / 2.0
        } else {
            x_pos().clamp(half, dimensions.width - half)
        }
    };
    let object_label_x = move || label_x(object_label_width.get());
    let attribute_label_x = move || label_x(attribute_label_width.get());

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
                font-family=SVG_FONT
                x=object_label_x
                y=move || {y_pos() + dimensions.radius * 2.8}
            >{node.label.0.clone().unwrap_or_default()}</text>
            <text
                font-size=dimensions.font_size
                dy=".35em"
                text-anchor="middle"
                fill="black"
                font-family=SVG_FONT
                node_ref=object_label_ref
                x=object_label_x
                y=move || {y_pos() + dimensions.radius * 2.8}
            >{node.label.0.unwrap_or_default()}</text>

            // attribute labels
            <text
                font-size=dimensions.font_size
                dy=".35em"
                text-anchor="middle"
                stroke="white"
                stroke-width="0.3em"
                font-style="italic"
                font-family=SVG_FONT
                x=attribute_label_x
                y=move || {y_pos() - dimensions.radius * 2.8}
            >{node.label.1.clone().unwrap_or_default()}</text>
            <text
                font-size=dimensions.font_size
                dy=".35em"
                text-anchor="middle"
                fill="black"
                font-style="italic"
                font-family=SVG_FONT
                node_ref=attribute_label_ref
                x=attribute_label_x
                y=move || {y_pos() - dimensions.radius * 2.8}
            >{node.label.1.unwrap_or_default()}</text>
        </g>
    }
}

/// Width of an SVG `<text>` element as it is actually rendered.
fn text_width(element: &web_sys::SvgElement) -> f64 {
    element
        .unchecked_ref::<web_sys::SvgTextContentElement>()
        .get_computed_text_length() as f64
}

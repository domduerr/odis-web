use leptos::prelude::*;

use crate::components::ui::BTN_PRIMARY;
use crate::utils::browser::serialize_and_download_svg;

#[component]
pub fn SvgDownloadComp(node_ref: NodeRef<leptos::svg::Svg>) -> impl IntoView {
    let link: NodeRef<leptos::html::A> = NodeRef::new();

    view! {
        <button
            on:click=move |_| {
                if let Some(svg_element) = node_ref.get()
                    && let Err(e) = serialize_and_download_svg(&svg_element, "concept_lattice") {
                        leptos::logging::log!("Failed to download SVG: {:?}", e);
                    }
            }
            class=format!("{BTN_PRIMARY} w-full")
        >
            "Save SVG"
        </button>
        <a
            node_ref=link
            style="display: none"
        />
    }
}

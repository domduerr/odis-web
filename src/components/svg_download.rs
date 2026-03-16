use crate::utils::browser::serialize_and_download_svg;
use leptos::prelude::*;

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
            class="w-full px-4 py-2 bg-dhbw-red text-white rounded hover:bg-red-700 text-sm font-medium transition-colors"
        >
            "Save SVG"
        </button>
        <a
            node_ref=link
            style="display: none"
        />
    }
}

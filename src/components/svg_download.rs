use crate::utils::browser::serialize_and_download_svg;
use leptos::prelude::*;

#[component]
pub fn SvgDownloadComp(node_ref: NodeRef<leptos::svg::Svg>) -> impl IntoView {
    let link: NodeRef<leptos::html::A> = NodeRef::new();

    view! {
        <button style:margin-left="20px" on:click=move |_| {
            if let Some(svg_element) = node_ref.get() {
                if let Err(e) = serialize_and_download_svg(&svg_element, "Graph_SVG") {
                    leptos::logging::log!("Failed to download SVG: {:?}", e);
                }
            }
        } id="download_concept_lattice">"Download Concept Lattice"</button>
        <a
            node_ref=link
            style="display: none"
        />
    }
}

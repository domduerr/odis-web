use leptos::prelude::*;
use web_sys::{Blob, BlobPropertyBag, Url, XmlSerializer, wasm_bindgen::JsValue};

#[component]
pub fn SvgDownloadComp(node_ref: NodeRef<leptos::svg::Svg>) -> impl IntoView {
    let link: NodeRef<leptos::html::A> = NodeRef::new();

    view! {
        <button style:margin-left="20px" on:click=move |_| {
            let serializer = XmlSerializer::new().unwrap();
            let xml_text = vec![serializer
                .serialize_to_string(&node_ref.get().unwrap())
                .unwrap()];
            let property_bag = BlobPropertyBag::new();
            property_bag.set_type("image/svg+xml;charset=utf-8");
            let blob = Blob::new_with_u8_array_sequence_and_options(&JsValue::from(xml_text), &property_bag).unwrap();
            let url = Url::create_object_url_with_blob(&blob).unwrap();

            link.get().unwrap().set_download("Graph_SVG");
            link.get().unwrap().set_href(&url);
            link.get().unwrap().click();
        }>"Download Concept Lattice"</button>
        <a
            node_ref=link
            style="display: none"
        />
    }
}

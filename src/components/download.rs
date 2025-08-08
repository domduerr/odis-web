use leptos::prelude::*;

use odis::FormalContext;
use web_sys::{File, Url, wasm_bindgen::JsValue};

#[component]
pub fn DownloadComp(context: RwSignal<FormalContext<String>>) -> impl IntoView {
    let link: NodeRef<leptos::html::A> = NodeRef::new();

    view! {
        <button on:click=move |_| {
            let context = context.read_only().get();
            let mut content = format!("B\n\n{}\n{}\n\n", context.objects.len(), context.attributes.len());

            for object in context.objects.iter() {
                if object != &"".to_string() {
                    content.push_str(object);
                } else {
                    content.push_str("\"no name\"");
                }
                content.push_str("\n");
            }
            for attribute in context.attributes.iter() {
                if attribute != &"".to_string() {
                    content.push_str(attribute);
                } else {
                    content.push_str("\"no name\"");
                }
                content.push_str("\n");
            }
            for column in 0..context.objects.len() {
                for row in 0..context.attributes.len() {
                    if context.incidence.contains(&(column, row)) {
                        content.push_str("X");
                    } else {
                        content.push_str(".");
                    }
                }
                content.push_str("\n");
            }

            let content = vec![content];
            let mut name = content[0].lines().next().unwrap().to_owned();
            if name == "B".to_string() {
                name = "Formal_context.cxt".to_string();
            }

            let file = File::new_with_u8_slice_sequence(&JsValue::from(content), &name).unwrap();
            let url = Url::create_object_url_with_blob(&file).unwrap();

            link.get().unwrap().set_download(&name);
            link.get().unwrap().set_href(&url);
            link.get().unwrap().click();
        }>"Download context"</button>
        <a
            node_ref=link
            style="display: none"
        />
    }
}

use leptos::prelude::*;

use crate::core::export::{generate_cxt_filename, generate_cxt_string};
use crate::utils::browser::trigger_text_download;
use odis::FormalContext;

#[component]
pub fn DownloadComp() -> impl IntoView {
    let context = use_context::<RwSignal<FormalContext<String>>>().expect("Context not provided");

    let link: NodeRef<leptos::html::A> = NodeRef::new();

    view! {
        <button on:click=move |_| {
            context.with(|ctx| {
                let content = generate_cxt_string(ctx);
                let filename = generate_cxt_filename(ctx);

                if let Err(e) = trigger_text_download(&content, &filename, "text/plain") {
                    leptos::logging::log!("Failed to trigger download: {:?}", e);
                }
            });
        } id="download_context">"Download Context"</button>
        <a
            node_ref=link
            style="display: none"
        />
    }
}

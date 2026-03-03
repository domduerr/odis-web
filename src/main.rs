use console_error_panic_hook;
use leptos::{logging, prelude::*, wasm_bindgen, task::spawn_local, wasm_bindgen::JsCast};

use odis::{self, FormalContext};

use crate::components::layout::{Header, Sidebar, View};
use crate::components::table::TableComp;
use crate::components::views::{ConceptsView, ConceptLatticeView, CanonicalBasisView, ExplorationViewWrapper};

mod components {
    pub mod checkbox;
    pub mod context;
    pub mod download;
    pub mod exploration;
    pub mod graph;
    pub mod svg_download;
    pub mod table;
    pub mod svg {
        pub mod edge;
        pub mod node;
    }
    pub mod layout;
    pub mod views;
}

mod js_fn;

use crate::components::context::create_default_context;

#[component]
pub fn App() -> impl IntoView {
    let context = RwSignal::new(Some(create_default_context()));
    let current_view = RwSignal::new(View::FormalContext);

    let on_context_loaded = {
        let context = context.clone();
        move |new_context: Option<FormalContext<String>>| {
            logging::log!("on_context_loaded called with: {:?}", new_context.is_some());
            if let Some(ctx) = new_context {
                logging::log!("Context loaded - Objects: {}, Attributes: {}", ctx.objects.len(), ctx.attributes.len());
                context.set(Some(ctx));
            } else {
                logging::log!("Failed to parse context, keeping current");
            }
        }
    };

    let on_save_context = {
        let context = context.clone();
        move |_| {
            if let Some(ref ctx) = context.get() {
                let ctx_clone = ctx.clone();
                spawn_local(async move {
                    let ctx = ctx_clone;
                    let mut content = format!("B\n\n{}\n{}\n\n", ctx.objects.len(), ctx.attributes.len());

                    for object in ctx.objects.iter() {
                        if object != &"".to_string() {
                            content.push_str(object);
                        } else {
                            content.push_str("\"no name\"");
                        }
                        content.push_str("\n");
                    }
                    for attribute in ctx.attributes.iter() {
                        if attribute != &"".to_string() {
                            content.push_str(attribute);
                        } else {
                            content.push_str("\"no name\"");
                        }
                        content.push_str("\n");
                    }
                    for column in 0..ctx.objects.len() {
                        for row in 0..ctx.attributes.len() {
                            if ctx.incidence.contains(&(column, row)) {
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

                    let file = web_sys::File::new_with_u8_slice_sequence(&wasm_bindgen::JsValue::from(content), &name).unwrap();
                    let url = web_sys::Url::create_object_url_with_blob(&file).unwrap();

                    let window = web_sys::window().unwrap();
                    let document = window.document().unwrap();
                    let link = document.create_element("a").unwrap();
                    link.set_attribute("download", &name).unwrap();
                    link.set_attribute("href", &url).unwrap();
                    let html_link: &web_sys::HtmlElement = link.dyn_ref().unwrap();
                    html_link.click();
                });
            }
        }
    };

    view! {
        <div class="flex flex-col h-screen bg-white">
            <Header/>
            <div class="flex flex-1 overflow-hidden">
                <Sidebar current_view on_context_loaded on_save_context/>
                <main class="flex-1 overflow-auto p-6 bg-gray-50">
                    {move || match current_view.get() {
                        View::FormalContext => {
                            view! { <TableComp context=context/> }.into_any()
                        }
                        View::Concepts => {
                            view! { <ConceptsView context=context/> }.into_any()
                        }
                        View::ConceptLattice => {
                            view! { <ConceptLatticeView context=context/> }.into_any()
                        }
                        View::CanonicalBasis => {
                            view! { <CanonicalBasisView context=context/> }.into_any()
                        }
                        View::Exploration => {
                            view! { <ExplorationViewWrapper context=context/> }.into_any()
                        }
                    }}
                </main>
            </div>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

use leptos::{logging, prelude::*};

use odis::FormalContext;

use crate::components::layout::{Header, Sidebar, View};
use crate::components::repository::RepositoryDialog;
use crate::components::table::TableComp;
use crate::components::views::{ConceptsView, ConceptLatticeView, CanonicalBasisView, ExplorationViewWrapper};
use crate::components::iceberg::IcebergView;

mod core {
    pub mod export;
    pub mod exploration_state;
    pub mod formatters;
    pub mod layout_math;
}

mod utils {
    pub mod browser;
}

mod components {
    pub mod context;
    pub mod repository;
    pub mod exploration;
    pub mod graph;
    pub mod iceberg;
    pub mod svg_download;
    pub mod table;
    pub mod ui;
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
    let context = RwSignal::new(create_default_context());
    provide_context(context);

    let current_view = RwSignal::new(View::FormalContext);
    let repository_open = RwSignal::new(false);

    let context_version: RwSignal<u64> = RwSignal::new(0);
    provide_context(context_version);

    let on_context_loaded = {
        move |new_context: Option<FormalContext<String>>| {
            if let Some(ctx) = new_context {
                context.set(ctx);
                context_version.update(|v| *v += 1);
                current_view.set(View::FormalContext);
            }
        }
    };

    let on_save_context = {
        move |_| {
            context.with(|ctx| {
                let content = crate::core::export::generate_cxt_string(ctx);
                let filename = crate::core::export::generate_cxt_filename(ctx);
                
                if let Err(e) = crate::utils::browser::trigger_text_download(&content, &filename, "text/plain") {
                    logging::log!("Failed to trigger download: {:?}", e);
                }
            });
        }
    };

    view! {
        <div class="flex flex-col h-screen bg-white">
            <Header/>
            <div class="flex flex-1 overflow-hidden">
                <Sidebar current_view on_context_loaded on_save_context repository_open/>
                <main class="flex-1 overflow-auto p-6 bg-gray-50">
                    {move || match current_view.get() {
                        View::FormalContext => {
                            view! { <TableComp/> }.into_any()
                        }
                        View::Concepts => {
                            view! { <ConceptsView/> }.into_any()
                        }
                        View::ConceptLattice => {
                            view! { <ConceptLatticeView/> }.into_any()
                        }
                        View::CanonicalBasis => {
                            view! { <CanonicalBasisView/> }.into_any()
                        }
                        View::Exploration => {
                            view! { <ExplorationViewWrapper/> }.into_any()
                        }
                        View::IcebergLattice => {
                            view! { <IcebergView/> }.into_any()
                        }
                    }}
                </main>
            </div>
            <RepositoryDialog open=repository_open on_context_loaded/>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

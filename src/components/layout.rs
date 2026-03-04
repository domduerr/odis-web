use leptos::logging;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::rc::Rc;

use odis::FormalContext;

use crate::js_fn;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum View {
    FormalContext,
    Concepts,
    ConceptLattice,
    CanonicalBasis,
    Exploration,
}

#[component]
fn SidebarItem(
    is_active: impl Fn() -> bool + Send + 'static,
    on_click: impl Fn() + Send + 'static,
    icon_path: &'static str,
    label: &'static str,
) -> impl IntoView {
    view! {
        <button
            on:click=move |_| on_click()
            class=move || {
                format!(
                    "w-full text-left px-3 py-2.5 rounded-md text-sm transition-all flex items-center gap-2 {}",
                    if is_active() {
                        "bg-dhbw-red text-white font-medium shadow-sm"
                    } else {
                        "text-dhbw-gray hover:bg-dhbw-gray/5"
                    }
                )
            }
        >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={icon_path}></path>
            </svg>
            {label}
        </button>
    }
}

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="h-16 bg-white border-b border-dhbw-gray-25 flex items-center justify-between px-8">
            <h1 class="text-dhbw-gray font-bold text-xl">Odis-Web</h1>
            <span class="text-dhbw-gray-50 font-mono text-sm">v0.1.0</span>
        </header>
    }
}

#[component]
pub fn Sidebar(
    current_view: RwSignal<View>,
    on_context_loaded: impl Fn(Option<FormalContext<String>>) + 'static,
    on_save_context: impl Fn(web_sys::MouseEvent) + 'static,
) -> impl IntoView {
    let file_input_element: NodeRef<leptos::html::Input> = NodeRef::new();
    let on_context_loaded_rc = Rc::new(on_context_loaded);

    let handle_file_change = move |_ev| {
        logging::log!("File change event fired");
        
        let file_input = file_input_element.get();
        if file_input.is_none() {
            logging::log!("File input not found");
            return;
        }

        let files = file_input.as_ref().unwrap().files();
        logging::log!("Files: {:?}", files.is_some());
        
        if files.is_none() {
            logging::log!("No files");
            return;
        }

        let files_list = files.as_ref().unwrap();
        let file_count = files_list.length();
        logging::log!("File count: {}", file_count);
        
        if file_count == 0 {
            logging::log!("No files selected (count is 0)");
            return;
        }

        let file = files_list.item(0);
        
        if file.is_none() {
            logging::log!("No file selected (item is None)");
            return;
        }

        let file = file.unwrap();
        logging::log!("File selected, name: {:?}", file.name());
        
        let on_context_loaded_clone = Rc::clone(&on_context_loaded_rc);
        spawn_local(async move {
            logging::log!("Starting to read file...");
            let contents = js_fn::file_contents(file.clone()).await;
            logging::log!("File contents length: {}", contents.len());
            
            match FormalContext::<String>::from(contents.as_bytes()) {
                Ok(ctx) => {
                    logging::log!("Context parsed successfully, objects: {}, attributes: {}", ctx.objects.len(), ctx.attributes.len());
                    on_context_loaded_clone(Some(ctx));
                }
                Err(e) => {
                    logging::log!("Parsing error: {:?}", e);
                }
            }
        });
    };

    let is_formalcontext = Signal::derive(move || current_view.get() == View::FormalContext);
    let is_concepts = Signal::derive(move || current_view.get() == View::Concepts);
    let is_lattice = Signal::derive(move || current_view.get() == View::ConceptLattice);
    let is_basis = Signal::derive(move || current_view.get() == View::CanonicalBasis);
    let is_exploration = Signal::derive(move || current_view.get() == View::Exploration);

    view! {
        <aside class="w-56 bg-white border-r border-dhbw-gray-25 flex flex-col">
            <nav class="p-2">
                <div class="space-y-1">
                    <input 
                        type="file" 
                        node_ref=file_input_element 
                        accept=".cxt,.cxtx,.txt"
                        on:change=handle_file_change
                        class="hidden"
                    />
                    <button
                        on:click=move |_| {
                            if let Some(el) = file_input_element.get() {
                                el.click();
                            }
                        }
                        type="button"
                        class="w-full text-left px-3 py-2.5 rounded-md text-sm transition-all flex items-center gap-2 text-dhbw-gray hover:bg-dhbw-gray/5"
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"></path>
                        </svg>
                        Load Context
                    </button>
                    <button
                        on:click=move |_| on_save_context(web_sys::MouseEvent::new("click").unwrap())
                        class="w-full text-left px-3 py-2.5 rounded-md text-sm transition-all flex items-center gap-2 text-dhbw-gray hover:bg-dhbw-gray/5"
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"></path>
                        </svg>
                        Save Context
                    </button>

                    <div class="border-b border-dhbw-gray-25 my-2"></div>

                    <SidebarItem
                        is_active=move || is_formalcontext.get()
                        on_click=move || current_view.set(View::FormalContext)
                        icon_path="M3 10h18M3 14h18m-9-4v8m-7 0h14a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z"
                        label="Formal Context"
                    />

                    <SidebarItem
                        is_active=move || is_concepts.get()
                        on_click=move || current_view.set(View::Concepts)
                        icon_path="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
                        label="Concepts"
                    />

                    <SidebarItem
                        is_active=move || is_lattice.get()
                        on_click=move || current_view.set(View::ConceptLattice)
                        icon_path="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z"
                        label="Concept Lattice"
                    />

                    <SidebarItem
                        is_active=move || is_basis.get()
                        on_click=move || current_view.set(View::CanonicalBasis)
                        icon_path="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01"
                        label="Canonical Basis"
                    />

                    <SidebarItem
                        is_active=move || is_exploration.get()
                        on_click=move || current_view.set(View::Exploration)
                        icon_path="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                        label="Exploration"
                    />
                </div>
            </nav>
        </aside>
    }
}

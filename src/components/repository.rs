//! Browsing the [FCA repository](https://fcarepository.org/) of published contexts.
//!
//! The catalogue and the context files are downloaded by `odis::repository`, the same
//! code the Rust library and the Python bindings use. On `wasm32` it routes through the
//! browser's fetch API, so this component only supplies the user interface around it.

use leptos::prelude::*;
use leptos::task::spawn_local;

use odis::repository::{fetch_catalog, fetch_context, RepositoryEntry};
use odis::FormalContext;

use crate::components::ui::{Panel, BTN_SECONDARY, INPUT};
use crate::core::formatters::{context_size, count};

/// What the dialog is showing while the catalogue is on its way.
#[derive(Clone, PartialEq)]
enum Status {
    Loading,
    Ready,
    Failed(String),
}

/// The context's size as stated by the catalogue, where it states it. Phrased
/// like the size beside the context's name in the Formal Context panel.
fn format_size(entry: &RepositoryEntry) -> Option<String> {
    match (entry.objects, entry.attributes) {
        (Some(objects), Some(attributes)) => Some(context_size(objects, attributes)),
        _ => None,
    }
}

/// True if `entry` matches the search term, which is compared in lower case
/// against everything the row puts on screen.
fn matches(entry: &RepositoryEntry, term: &str) -> bool {
    if term.is_empty() {
        return true;
    }
    let haystack = [
        entry.title.as_str(),
        entry.filename.as_str(),
        entry.description.as_deref().unwrap_or(""),
        entry.language.as_deref().unwrap_or(""),
    ];
    haystack
        .iter()
        .any(|field| field.to_lowercase().contains(term))
}

/// Modal listing the repository, one row per context. Picking a row downloads it
/// and hands it to `on_context_loaded`, exactly as the file picker does.
#[component]
pub fn RepositoryDialog(
    open: RwSignal<bool>,
    on_context_loaded: impl Fn(Option<FormalContext<String>>) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let entries: RwSignal<Vec<RepositoryEntry>> = RwSignal::new(Vec::new());
    let status = RwSignal::new(Status::Loading);
    let filter = RwSignal::new(String::new());
    // File name of the context currently downloading, so its row can say so and
    // the others can be disabled.
    let pending: RwSignal<Option<String>> = RwSignal::new(None);

    let load_catalog = move || {
        status.set(Status::Loading);
        spawn_local(async move {
            match fetch_catalog().await {
                Ok(list) => {
                    entries.set(list);
                    status.set(Status::Ready);
                }
                Err(err) => status.set(Status::Failed(err.to_string())),
            }
        });
    };

    // Fetch once, the first time the dialog is opened; afterwards the catalogue is
    // kept, so reopening it is instant.
    Effect::new(move |_| {
        if open.get() && entries.with_untracked(Vec::is_empty) {
            load_catalog();
        }
    });

    let load_context = move |filename: String| {
        pending.set(Some(filename.clone()));
        spawn_local(async move {
            match fetch_context(&filename).await {
                Ok(context) => {
                    on_context_loaded(Some(context));
                    open.set(false);
                }
                Err(err) => status.set(Status::Failed(err.to_string())),
            }
            pending.set(None);
        });
    };

    let visible = move || {
        let term = filter.get().to_lowercase();
        entries.with(|list| {
            list.iter()
                .filter(|entry| matches(entry, &term))
                .cloned()
                .collect::<Vec<_>>()
        })
    };

    view! {
        <Show when=move || open.get()>
            <div
                class="fixed inset-0 z-50 flex items-center justify-center bg-dhbw-gray/40 p-6"
                on:click=move |_| open.set(false)
            >
                <div
                    class="flex max-h-full w-full max-w-2xl"
                    on:click=|ev| ev.stop_propagation()
                >
                    <Panel
                        title=|| "FCA Repository"
                        meta=Signal::derive(move || match status.get() {
                            Status::Ready => count(visible().len(), "context"),
                            _ => String::new(),
                        })
                        // A fixed height, so the dialog keeps still while the search is typed.
                        class="h-[36rem] max-h-full w-full shadow-xl"
                    >

                    <div class="flex-shrink-0 border-b border-dhbw-gray-25 px-4 py-3">
                        <input
                            class=INPUT
                            type="search"
                            placeholder="Search by name, description or language"
                            prop:value=move || filter.get()
                            on:input=move |ev| filter.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="min-h-0 flex-1 overflow-y-auto">
                        {move || match status.get() {
                            Status::Loading => view! {
                                <p class="px-4 py-8 text-center text-sm text-dhbw-gray-50">
                                    "Loading the catalogue…"
                                </p>
                            }.into_any(),

                            Status::Failed(message) => view! {
                                <div class="space-y-3 px-4 py-8 text-center">
                                    <p class="text-sm text-dhbw-red">{message}</p>
                                    <button class=BTN_SECONDARY on:click=move |_| load_catalog()>
                                        "Try again"
                                    </button>
                                </div>
                            }.into_any(),

                            Status::Ready if visible().is_empty() => view! {
                                <p class="px-4 py-8 text-center text-sm text-dhbw-gray-50">
                                    "No context matches this search"
                                </p>
                            }.into_any(),

                            Status::Ready => view! {
                                <ul class="divide-y divide-dhbw-gray-25">
                                    {move || visible().into_iter().map(|entry| {
                                        let filename = entry.filename.clone();
                                        let is_pending = {
                                            let filename = filename.clone();
                                            move || pending.get().as_deref() == Some(filename.as_str())
                                        };
                                        let size = format_size(&entry);

                                        view! {
                                            <li>
                                                <button
                                                    class="w-full px-4 py-3 text-left transition-colors \
                                                           hover:bg-dhbw-gray/5 disabled:cursor-not-allowed disabled:opacity-50"
                                                    disabled=move || pending.get().is_some()
                                                    on:click=move |_| load_context(filename.clone())
                                                >
                                                    <div class="flex items-baseline gap-3">
                                                        <span class="min-w-0 flex-1 text-sm font-medium text-dhbw-gray">
                                                            {entry.title.clone()}
                                                        </span>
                                                        <span class="shrink-0 text-xs text-dhbw-gray-50">
                                                            {move || if is_pending() {
                                                                "loading…".to_string()
                                                            } else {
                                                                size.clone().unwrap_or_default()
                                                            }}
                                                        </span>
                                                    </div>
                                                    {entry.description.clone().map(|description| view! {
                                                        <p class="mt-0.5 text-xs text-dhbw-gray-50">{description}</p>
                                                    })}
                                                    <p class="mt-0.5 font-mono text-xs text-dhbw-gray-50">
                                                        {entry.filename.clone()}
                                                        {entry.language.clone().map(|l| format!(" · {l}")).unwrap_or_default()}
                                                    </p>
                                                </button>
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                            }.into_any(),
                        }}
                    </div>

                    <footer class="flex flex-shrink-0 justify-end border-t border-dhbw-gray-25 px-4 py-3">
                        <button class=BTN_SECONDARY on:click=move |_| open.set(false)>
                            "Close"
                        </button>
                    </footer>

                    </Panel>
                </div>
            </div>
        </Show>
    }
}

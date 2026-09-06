use leptos::prelude::*;
use leptos::wasm_bindgen::closure::Closure;
use leptos::wasm_bindgen::JsCast;

use bit_set::BitSet;
use odis::FormalContext;

use crate::components::ui::{
    Arrow, Implication, Panel, ARROW_CELL, BTN_PRIMARY, BTN_SECONDARY, SET_TEXT, TH,
};
use crate::core::exploration_state::{ExplorationInput, ExplorationMachine, ExplorationState};
use crate::core::formatters::{context_size, count, format_set};

#[derive(Clone, PartialEq)]
enum ViewState {
    Validating,
    Counterexample,
    Finished,
}

fn render_table_row(obj_idx: usize, context: RwSignal<FormalContext<String>>) -> impl IntoView {
    let obj_name = context.with(|ctx| ctx.objects[obj_idx].clone());
    let row_idx = obj_idx;

    view! {
        <tr>
            <td class="border border-dhbw-gray-25 bg-gray-50 p-1 font-medium text-dhbw-gray">{obj_name}</td>
            <For
                each=move || context.with(|ctx| (0..ctx.attributes.len()).collect::<Vec<_>>())
                key=|&idx| idx
                children=move |attr_idx| {
                    let checked = context.with_untracked(|ctx| ctx.incidence.contains(&(row_idx, attr_idx)));
                    view! {
                        <td class="border border-dhbw-gray-25 bg-white p-1 text-center">
                            <input
                                type="checkbox"
                                checked=checked
                                disabled=true
                                class="h-4 w-4 accent-dhbw-red"
                            />
                        </td>
                    }
                }
            />
        </tr>
    }
}

#[component]
pub fn ExplorationComp() -> impl IntoView {
    let context = use_context::<RwSignal<FormalContext<String>>>().expect("Context not provided");

    let machine = RwSignal::new(ExplorationMachine::new());
    let view_state = RwSignal::new(ViewState::Validating);
    let is_initialized = RwSignal::new(false);

    let context_scroll_ref = NodeRef::<leptos::html::Div>::new();
    let implications_scroll_ref = NodeRef::<leptos::html::Div>::new();

    let scroll_to_bottom = move |node_ref: NodeRef<leptos::html::Div>| {
        if let Some(window) = web_sys::window()
            && let Some(el) = node_ref.get_untracked() {
                let cb = Closure::wrap(Box::new(move || {
                    el.set_scroll_top(el.scroll_height());
                }) as Box<dyn FnMut()>);
                let _ = window.set_timeout_with_callback(cb.as_ref().unchecked_ref());
                cb.forget();
            }
    };

    let init_exploration = move || {
        if !is_initialized.get_untracked() {
            let context_clone = context;
            machine.update(|m| {
                let _ = m.process_input(move || context_clone.get_untracked(), ExplorationInput::Start);
            });
            is_initialized.set(true);

            let state = machine.with_untracked(|m| m.state.clone());
            match state {
                ExplorationState::Finished => view_state.set(ViewState::Finished),
                ExplorationState::ValidatingImplication { .. } => {
                    view_state.set(ViewState::Validating)
                }
                ExplorationState::AwaitingCounterexample { .. } => {
                    view_state.set(ViewState::Counterexample)
                }
                _ => {}
            }
        }
    };

    init_exploration();

    let new_object_name = RwSignal::new(String::new());
    let new_object_checkboxes: RwSignal<Vec<RwSignal<bool>>> = RwSignal::new(Vec::new());
    let new_object_checkboxes_disabled: RwSignal<Vec<bool>> = RwSignal::new(Vec::new());
    let new_object_input_ref = NodeRef::<leptos::html::Span>::new();
    let counterexample_valid = RwSignal::new(true);

    // A counterexample is only one if it misses part of the conclusion.
    let conclusion_fully_selected = move || {
        let conclusion: BitSet = machine.with(|m| match &m.state {
            ExplorationState::ValidatingImplication {
                premise,
                conclusion,
            }
            | ExplorationState::AwaitingCounterexample {
                premise,
                conclusion,
            } => conclusion.difference(premise).collect(),
            _ => BitSet::new(),
        });

        let checkboxes = new_object_checkboxes.get();
        conclusion
            .iter()
            .all(|i| checkboxes.get(i).is_some_and(|cb| cb.get()))
    };

    Effect::new(move |_| {
        let named = !new_object_name.get().trim().is_empty();
        counterexample_valid.set(named && !conclusion_fully_selected());
    });

    let attr_len = context.with_untracked(|ctx| ctx.attributes.len());
    new_object_checkboxes.set(vec![RwSignal::new(false); attr_len]);

    let handle_yes = move |_| {
        let context_clone = context;
        machine.update(|m| {
            let _ = m.process_input(move || context_clone.get_untracked(), ExplorationInput::Yes);
        });

        let state = machine.with_untracked(|m| m.state.clone());
        match state {
            ExplorationState::ValidatingImplication { .. } => view_state.set(ViewState::Validating),
            ExplorationState::Finished => view_state.set(ViewState::Finished),
            _ => {}
        }
        scroll_to_bottom(implications_scroll_ref);
    };

    let handle_no = move |_| {
        new_object_name.set(String::new());
        let attr_len = context.with_untracked(|ctx| ctx.attributes.len());

        let premise: BitSet = machine.with_untracked(|m| {
            if let ExplorationState::ValidatingImplication { premise, .. } = &m.state {
                premise.clone()
            } else {
                BitSet::new()
            }
        });

        let checkboxes: Vec<RwSignal<bool>> = (0..attr_len)
            .map(|i| RwSignal::new(premise.contains(i)))
            .collect();
        new_object_checkboxes.set(checkboxes);

        let disabled: Vec<bool> = (0..attr_len).map(|i| premise.contains(i)).collect();
        new_object_checkboxes_disabled.set(disabled);

        let context_clone = context;
        machine.update(|m| {
            let _ = m.process_input(move || context_clone.get_untracked(), ExplorationInput::No);
        });

        view_state.set(ViewState::Counterexample);

        let new_object_input_ref = new_object_input_ref;
        let context_scroll_ref = context_scroll_ref;
        let cb = Closure::wrap(Box::new(move || {
            let span_el = new_object_input_ref.get_untracked();
            let ctx_el = context_scroll_ref.get_untracked();
            if let Some(span) = &span_el {
                let _ = span.focus();
            }
            if let Some(el) = &ctx_el {
                el.set_scroll_top(el.scroll_height());
            }
        }) as Box<dyn FnMut()>);
        web_sys::window()
            .expect("no window")
            .set_timeout_with_callback(cb.as_ref().unchecked_ref())
            .ok();
        cb.forget();
    };

    let handle_submit = move |_| {
        if conclusion_fully_selected() {
            return;
        }

        let mut attribute_set = BitSet::new();
        for (i, checkbox) in new_object_checkboxes.get_untracked().iter().enumerate() {
            if checkbox.get_untracked() {
                attribute_set.insert(i);
            }
        }

        let object_name = new_object_name.get_untracked();
        if !object_name.is_empty() {
            context.update(|ctx| {
                ctx.add_object(object_name, &attribute_set);
            });
        }

        let context_clone = context;
        machine.update(|m| {
            let _ = m.process_input(
                move || context_clone.get_untracked(),
                ExplorationInput::Submit {
                    counterexample: attribute_set,
                },
            );
        });

        let state = machine.with_untracked(|m| m.state.clone());
        match state {
            ExplorationState::ValidatingImplication { .. } => view_state.set(ViewState::Validating),
            ExplorationState::Finished => view_state.set(ViewState::Finished),
            _ => {}
        }
        scroll_to_bottom(context_scroll_ref);
    };

    // The implication currently on the table, rendered once and reused by the
    // question, the counterexample prompt and the validation checks.
    let premise_text = Signal::derive(move || {
        machine.with(|m| context.with(|ctx| format_set(&m.temp_set, &ctx.attributes)))
    });
    let conclusion_text = Signal::derive(move || {
        machine.with(|m| {
            context.with(|ctx| {
                let conclusion: BitSet = m.temp_hull.difference(&m.temp_set).collect();
                format_set(&conclusion, &ctx.attributes)
            })
        })
    });

    let size = Signal::derive(move || {
        context.with(|ctx| context_size(ctx.objects.len(), ctx.attributes.len()))
    });
    let implication_count =
        Signal::derive(move || count(machine.with(|m| m.basis.len()), "implication"));

    view! {
        <div class="flex h-[calc(100vh-7rem)] flex-col gap-6">
            <div class="grid min-h-0 flex-1 grid-cols-1 gap-6 overflow-hidden lg:grid-cols-2">
                <Panel title=|| "Current Context" meta=size class="min-h-0">
                    <div node_ref=context_scroll_ref class="flex-1 overflow-auto p-3">
                        <table class="border-collapse text-sm">
                            <tbody>
                                <tr>
                                    <td class="p-1"></td>
                                    <For
                                        each=move || context.with(|ctx| ctx.attributes.to_vec())
                                        key=|attr| attr.clone()
                                        children=|attr| {
                                            view! {
                                                <td class="border border-dhbw-gray-25 bg-gray-50 p-1 text-center font-medium text-dhbw-gray">{attr}</td>
                                            }
                                        }
                                    />
                                </tr>
                                <For
                                    each=move || {
                                        let ctx = context.get();
                                        (0..ctx.objects.len()).collect::<Vec<_>>()
                                    }
                                    key=|&obj_idx| obj_idx
                                    children=move |obj_idx| {
                                        render_table_row(obj_idx, context)
                                    }
                                />

                                {move || {
                                    if view_state.get() == ViewState::Counterexample {
                                        let checkboxes = new_object_checkboxes.get();
                                        let disabled = new_object_checkboxes_disabled.get();
                                        let checkbox_data: Vec<(usize, RwSignal<bool>, bool)> = checkboxes
                                            .into_iter()
                                            .enumerate()
                                            .map(|(i, cb)| (i, cb, disabled[i]))
                                            .collect();
                                            Some(view! {
                                            <tr>
                                                <td class="border border-dhbw-red-50 bg-dhbw-red/5 p-1">
                                                    <span
                                                        node_ref=new_object_input_ref
                                                        contenteditable=true
                                                        data-placeholder="new object"
                                                        class="block cursor-text px-1 text-dhbw-gray outline-none"
                                                        on:input=move |ev| {
                                                            let target = ev.target().unwrap();
                                                            let span: web_sys::HtmlElement = target.unchecked_into();
                                                            let val = span.text_content().unwrap_or_default().trim().to_string();
                                                            new_object_name.set(val);
                                                        }
                                                    ></span>
                                                </td>
                                                <For
                                                    each=move || checkbox_data.clone()
                                                    key=|item| item.0
                                                    children=|(_idx, checkbox, is_disabled)| {
                                                        view! {
                                                            <td class="border border-dhbw-red-50 bg-dhbw-red/5 p-1 text-center">
                                                                <input
                                                                    type="checkbox"
                                                                    checked=checkbox.get()
                                                                    disabled=is_disabled
                                                                    on:change=move |ev| {
                                                                        let input: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into();
                                                                        checkbox.set(input.checked());
                                                                    }
                                                                    class="h-4 w-4 accent-dhbw-red"
                                                                />
                                                            </td>
                                                        }
                                                    }
                                                />
                                            </tr>
                                        }.into_any())
                                    } else {
                                        None
                                    }
                                }}
                            </tbody>
                        </table>
                    </div>
                </Panel>

                <Panel title=|| "Discovered Implications" meta=implication_count class="min-h-0">
                    <div node_ref=implications_scroll_ref class="flex-1 overflow-auto">
                        {move || {
                            let basis = machine.with(|m| m.basis.clone());
                            let attrs = context.with(|ctx| ctx.attributes.clone());

                            if basis.is_empty() {
                                view! {
                                    <p class="p-4 text-center text-sm text-dhbw-gray-50">
                                        No implications discovered yet
                                    </p>
                                }.into_any()
                            } else {
                                view! {
                                    <table class="w-full table-fixed text-sm">
                                        <thead class="sticky top-0 bg-white">
                                            <tr class="border-b border-dhbw-gray-25">
                                                <th class=format!("{TH} w-12")>#</th>
                                                <th class=format!("{TH} w-1/2")>Premise</th>
                                                <th class=format!("{TH} w-10")></th>
                                                <th class=format!("{TH} w-1/2")>Conclusion</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {basis.into_iter().enumerate().map(|(idx, implication)| {
                                                let premise_str = format_set(&implication.0, &attrs);
                                                let conclusion_str = format_set(&implication.1, &attrs);
                                                view! {
                                                    <tr class="border-t border-dhbw-gray-25 hover:bg-gray-50">
                                                        <td class="px-4 py-2 align-top text-dhbw-gray-50">{idx + 1}</td>
                                                        <td class=format!("{SET_TEXT} px-4 py-2 align-top")>{premise_str}</td>
                                                        <td class=ARROW_CELL><Arrow/></td>
                                                        <td class=format!("{SET_TEXT} px-4 py-2 align-top")>{conclusion_str}</td>
                                                    </tr>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                }.into_any()
                            }
                        }}
                    </div>
                </Panel>
            </div>

            {move || {
                match view_state.get() {
                    ViewState::Finished => {
                        view! {
                            <Panel title=|| "Exploration finished" class="flex-shrink-0">
                                <p class="p-4 text-sm text-dhbw-gray">
                                    {move || format!(
                                        "The canonical basis of the explored context contains {}.",
                                        implication_count.get(),
                                    )}
                                </p>
                            </Panel>
                        }.into_any()
                    }
                    ViewState::Counterexample => {
                        view! {
                            <Panel title=|| "Provide a counterexample to this implication" class="max-h-[45vh] flex-shrink-0">
                                <div class="flex min-h-0 flex-col gap-3 p-4">
                                    <div class="min-h-0 flex-1 overflow-auto rounded-md border border-dhbw-gray-25 bg-gray-50 px-3 py-2">
                                        <Implication premise=premise_text conclusion=conclusion_text/>
                                    </div>
                                    <div class="flex flex-shrink-0 flex-wrap items-center gap-3">
                                        <button
                                            on:click=handle_submit
                                            disabled=move || !counterexample_valid.get()
                                            class=BTN_PRIMARY
                                        >
                                            Accept
                                        </button>
                                        <p class="text-sm text-dhbw-red">
                                            {move || {
                                                if counterexample_valid.get() {
                                                    return String::new();
                                                }
                                                let mut messages = Vec::new();
                                                if new_object_name.get().trim().is_empty() {
                                                    messages.push("Name the new object.");
                                                }
                                                if conclusion_fully_selected() {
                                                    messages.push("Leave at least one conclusion attribute unchecked.");
                                                }
                                                messages.join(" ")
                                            }}
                                        </p>
                                    </div>
                                </div>
                            </Panel>
                        }.into_any()
                    }
                    ViewState::Validating => {
                        view! {
                            <Panel title=|| "Is the following implication valid?" class="max-h-[45vh] flex-shrink-0">
                                <div class="flex min-h-0 flex-col gap-3 p-4">
                                    <div class="min-h-0 flex-1 overflow-auto rounded-md border border-dhbw-gray-25 bg-gray-50 px-3 py-2">
                                        <Implication premise=premise_text conclusion=conclusion_text/>
                                    </div>
                                    <div class="flex flex-shrink-0 gap-3">
                                        <button on:click=handle_yes class=BTN_PRIMARY>Yes</button>
                                        <button on:click=handle_no class=BTN_SECONDARY>No</button>
                                    </div>
                                </div>
                            </Panel>
                        }.into_any()
                    }
                }
            }}
        </div>
    }
}

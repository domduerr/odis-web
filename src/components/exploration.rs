use leptos::prelude::*;
use leptos::wasm_bindgen::closure::Closure;
use leptos::wasm_bindgen::JsCast;

use bit_set::BitSet;
use odis::FormalContext;

use crate::core::exploration_state::{ExplorationInput, ExplorationMachine, ExplorationState};
use crate::core::formatters::format_attribute_set;

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
            <td class="p-1 border border-dhbw-gray-25 bg-gray-100 text-dhbw-gray font-medium">{obj_name}</td>
            <For
                each=move || context.with(|ctx| (0..ctx.attributes.len()).collect::<Vec<_>>())
                key=|&idx| idx
                children=move |attr_idx| {
                    let checked = context.with_untracked(|ctx| ctx.incidence.contains(&(row_idx, attr_idx)));
                    view! {
                        <td class="p-1 text-center border border-dhbw-gray-25 bg-white">
                            <input
                                type="checkbox"
                                checked=checked
                                disabled=true
                                class="accent-dhbw-red w-4 h-4"
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
        if let Some(window) = web_sys::window() {
            if let Some(el) = node_ref.get_untracked() {
                let cb = Closure::wrap(Box::new(move || {
                    el.set_scroll_top(el.scroll_height() as i32);
                }) as Box<dyn FnMut()>);
                let _ = window.set_timeout_with_callback(cb.as_ref().unchecked_ref());
                cb.forget();
            }
        }
    };

    let init_exploration = move || {
        if !is_initialized.get() {
            let context_clone = context.clone();
            machine.update(|m| {
                let _ = m.process_input(move || context_clone.get(), ExplorationInput::Start);
            });
            is_initialized.set(true);

            let state = machine.with(|m| m.state.clone());
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

    let checkboxes_clone = new_object_checkboxes.clone();
    let machine_clone = machine.clone();
    let counterexample_valid_clone = counterexample_valid.clone();
    let new_object_name_clone = new_object_name.clone();

    Effect::new(move |_| {
        let checkboxes = checkboxes_clone.get();
        let object_name = new_object_name_clone.get();
        let conclusion: BitSet = machine_clone.with(|m| match &m.state {
            ExplorationState::ValidatingImplication {
                premise,
                conclusion,
            }
            | ExplorationState::AwaitingCounterexample {
                premise,
                conclusion,
            } => conclusion.difference(&premise).collect(),
            _ => BitSet::new(),
        });

        let mut selected = BitSet::new();
        for (i, cb) in checkboxes.iter().enumerate() {
            if cb.get() {
                selected.insert(i);
            }
        }

        let all_conclusion_selected = conclusion.iter().all(|i| selected.contains(i));
        let is_valid = !all_conclusion_selected && !object_name.trim().is_empty();
        counterexample_valid_clone.set(is_valid);
    });

    let attr_len = context.with(|ctx| ctx.attributes.len());
    new_object_checkboxes.set(vec![RwSignal::new(false); attr_len]);

    let handle_yes = move |_| {
        let context_clone = context.clone();
        machine.update(|m| {
            let _ = m.process_input(move || context_clone.get(), ExplorationInput::Yes);
        });

        let state = machine.with(|m| m.state.clone());
        match state {
            ExplorationState::ValidatingImplication { .. } => view_state.set(ViewState::Validating),
            ExplorationState::Finished => view_state.set(ViewState::Finished),
            _ => {}
        }
        scroll_to_bottom(implications_scroll_ref.clone());
    };

    let handle_no = move |_| {
        new_object_name.set(String::new());
        let attr_len = context.with(|ctx| ctx.attributes.len());

        let premise: BitSet = machine.with(|m| {
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

        let context_clone = context.clone();
        machine.update(|m| {
            let _ = m.process_input(move || context_clone.get(), ExplorationInput::No);
        });

        view_state.set(ViewState::Counterexample);

        let new_object_input_ref = new_object_input_ref.clone();
        let context_scroll_ref = context_scroll_ref.clone();
        let cb = Closure::wrap(Box::new(move || {
            let span_el = new_object_input_ref.get_untracked();
            let ctx_el = context_scroll_ref.get_untracked();
            if let Some(span) = &span_el {
                let _ = span.focus();
            }
            if let Some(el) = &ctx_el {
                el.set_scroll_top(el.scroll_height() as i32);
            }
        }) as Box<dyn FnMut()>);
        web_sys::window()
            .expect("no window")
            .set_timeout_with_callback(cb.as_ref().unchecked_ref())
            .ok();
        cb.forget();
    };

    let handle_submit = move |_| {
        let conclusion: BitSet = machine.with(|m| match &m.state {
            ExplorationState::ValidatingImplication {
                premise,
                conclusion,
            }
            | ExplorationState::AwaitingCounterexample {
                premise,
                conclusion,
            } => conclusion.difference(&premise).collect(),
            _ => BitSet::new(),
        });

        let mut attribute_set = BitSet::new();
        for (i, checkbox) in new_object_checkboxes.get().iter().enumerate() {
            if checkbox.get() {
                attribute_set.insert(i);
            }
        }

        let all_conclusion_selected = conclusion.iter().all(|i| attribute_set.contains(i));
        if all_conclusion_selected {
            return;
        }

        let object_name = new_object_name.get();
        if !object_name.is_empty() {
            context.update(|ctx| {
                ctx.add_object(object_name, &attribute_set);
            });
        }

        let context_clone = context.clone();
        machine.update(|m| {
            let _ = m.process_input(
                move || context_clone.get(),
                ExplorationInput::Submit {
                    counterexample: attribute_set,
                },
            );
        });

        let state = machine.with(|m| m.state.clone());
        match state {
            ExplorationState::ValidatingImplication { .. } => view_state.set(ViewState::Validating),
            ExplorationState::Finished => view_state.set(ViewState::Finished),
            _ => {}
        }
        scroll_to_bottom(context_scroll_ref.clone());
    };

    let handle_reset = move |_: leptos::ev::MouseEvent| {
        machine.update(|m| m.reset());
        is_initialized.set(false);

        let context_clone = context.clone();
        machine.update(|m| {
            let _ = m.process_input(move || context_clone.get(), ExplorationInput::Start);
        });

        let state = machine.with(|m| m.state.clone());
        match state {
            ExplorationState::Finished => view_state.set(ViewState::Finished),
            ExplorationState::ValidatingImplication { .. } => view_state.set(ViewState::Validating),
            ExplorationState::AwaitingCounterexample { .. } => {
                view_state.set(ViewState::Counterexample)
            }
            _ => {}
        }
    };

    view! {
        <div class="flex flex-col h-[calc(100vh-7rem)] gap-6">
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 flex-1 min-h-0 overflow-hidden">
                <div class="bg-white rounded-lg border border-dhbw-gray-25 overflow-hidden flex flex-col">
                    <div class="bg-dhbw-gray-25 px-4 py-2 border-b border-dhbw-gray-25 flex-shrink-0">
                        <span class="text-dhbw-gray font-medium">Current Context</span>
                        <span class="text-dhbw-gray-50 text-sm ml-2">
                            {move || format!("({} objects)", context.with(|ctx| ctx.objects.len()))}
                        </span>
                    </div>
                    <div node_ref=context_scroll_ref class="overflow-auto flex-1">
                        <table class="border-collapse">
                            <tbody>
                                <tr>
                                    <td class="p-1"></td>
                                    <For
                                        each=move || context.with(|ctx| ctx.attributes.iter().cloned().collect::<Vec<_>>())
                                        key=|attr| attr.clone()
                                        children=|attr| {
                                            view! {
                                                <td class="p-1 text-center text-dhbw-gray font-medium border border-dhbw-gray-25 bg-gray-100">{attr}</td>
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
                                                <td class="p-1 border border-dhbw-gray-25 bg-gray-100">
                                                    <span
                                                        node_ref=new_object_input_ref
                                                        contenteditable=true
                                                        class="px-1 text-sm cursor-text hover:bg-gray-50 block outline-none"
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
                                                        let checkbox = checkbox.clone();
                                                        view! {
                                                            <td class="p-1 text-center border border-dhbw-gray-25 bg-white">
                                                                <input
                                                                    type="checkbox"
                                                                    checked=checkbox.get()
                                                                    disabled=is_disabled
                                                                    on:change=move |ev| {
                                                                        let input: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into();
                                                                        checkbox.set(input.checked());
                                                                    }
                                                                    class="accent-dhbw-red w-4 h-4"
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
                </div>

                <div class="bg-white rounded-lg border border-dhbw-gray-25 overflow-hidden flex flex-col">
                    <div class="bg-dhbw-gray-25 px-4 py-2 border-b border-dhbw-gray-25 flex-shrink-0">
                        <span class="text-dhbw-gray font-medium">Discovered Implications</span>
                        <span class="text-dhbw-gray-50 text-sm ml-2">
                            {move || format!("({})", machine.with(|m| m.basis.len()))}
                        </span>
                    </div>
                    <div node_ref=implications_scroll_ref class="overflow-auto flex-1">
                        {move || {
                            let basis = machine.with(|m| m.basis.clone());
                            let attrs = context.with(|ctx| ctx.attributes.clone());

                            if basis.is_empty() {
                                view! {
                                    <div class="p-4 text-dhbw-gray-50 text-sm text-center">
                                        No implications discovered yet
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <table class="w-full text-sm">
                                        <thead class="bg-gray-50 sticky top-0">
                                            <tr>
                                                <th class="px-4 py-2 text-left text-dhbw-gray-50 font-medium w-16">#</th>
                                                <th class="px-4 py-2 text-left text-dhbw-gray-50 font-medium w-1/2">Premise</th>
                                                <th class="px-4 py-2 text-center text-dhbw-gray-50 font-medium w-8"></th>
                                                <th class="px-4 py-2 text-left text-dhbw-gray-50 font-medium w-1/2">Conclusion</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {basis.into_iter().enumerate().map(|(idx, implication)| {
                                                let premise_str = format_attribute_set(&implication.0, &attrs);
                                                let conclusion_str = format_attribute_set(&implication.1, &attrs);
                                                view! {
                                                    <tr class="border-t border-dhbw-gray-25 hover:bg-gray-50">
                                                        <td class="px-4 py-2 text-dhbw-gray-50 align-top">{idx + 1}</td>
                                                        <td class="px-4 py-2 text-dhbw-gray font-mono whitespace-normal break-all align-top">{premise_str}</td>
                                                        <td class="px-4 py-2 text-black text-center align-top">{"→"}</td>
                                                        <td class="px-4 py-2 text-dhbw-gray font-mono whitespace-normal break-all align-top">{conclusion_str}</td>
                                                    </tr>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>
            </div>

            <div class="bg-white rounded-lg border border-dhbw-gray-25 p-4 flex-shrink-0 h-32">
                {move || {
                    match view_state.get() {
                        ViewState::Finished => {
                            view! {
                                <div class="flex flex-col h-full justify-between">
                                    <p class="text-dhbw-gray text-lg font-medium">Exploration finished.</p>
                                    <p class="text-dhbw-gray">
                                        {format!("{} implication(s) discovered.", machine.with(|m| m.basis.len()))}
                                    </p>
                                </div>
                            }.into_any()
                        }
                        ViewState::Counterexample => {
                            view! {
                                <div class="flex flex-col h-full justify-between">
                                    <p class="text-dhbw-gray">Provide a counterexample to:{" "}
                                        <div class="flex items-center text-dhbw-gray font-mono mb-2">
                                            {move || {
                                                machine.with(|m| {
                                                    context.with(|ctx| {
                                                        let premise: Vec<String> = m.temp_set.iter()
                                                            .filter(|&i| i < ctx.attributes.len())
                                                            .map(|i| ctx.attributes[i].clone())
                                                            .collect();
                                                        format!("{{{}}}", premise.join(", "))
                                                    })
                                                })
                                            }}
                                            <span class="text-black px-2">{"→"}</span>
                                            {move || {
                                                machine.with(|m| {
                                                    context.with(|ctx| {
                                                        let conclusion: Vec<String> = m.temp_hull.difference(&m.temp_set)
                                                        .filter(|&i| i < ctx.attributes.len())
                                                        .map(|i| ctx.attributes[i].clone())
                                                        .collect();
                                                    format!("{{{}}}", conclusion.join(", "))
                                                })
                                            })
                                        }}</div>
                                    </p>
                                    <div class="h-6 flex items-center">
                                        {move || {
                                            if !counterexample_valid.get() {
                                                let name_empty = new_object_name.get().trim().is_empty();
                                                let conclusion_selected = {
                                                    let checkboxes = new_object_checkboxes.get();
                                                    let conclusion: BitSet = machine.with(|m| {
                                                        match &m.state {
                                                            ExplorationState::ValidatingImplication { premise, conclusion }
                                                            | ExplorationState::AwaitingCounterexample { premise, conclusion } => {
                                                                conclusion.difference(&premise).collect()
                                                            }
                                                            _ => BitSet::new(),
                                                        }
                                                    });
                                                    let mut selected = BitSet::new();
                                                    for (i, cb) in checkboxes.iter().enumerate() {
                                                        if cb.get() {
                                                            selected.insert(i);
                                                        }
                                                    }
                                                    conclusion.iter().all(|i| selected.contains(i))
                                                };

                                                let mut messages = Vec::new();
                                                if name_empty {
                                                    messages.push("Enter a name.");
                                                }
                                                if conclusion_selected {
                                                    messages.push("Uncheck a conclusion attribute.");
                                                }

                                                Some(view! {
                                                    <p class="text-dhbw-red text-sm">
                                                        {messages.join(" ")}
                                                    </p>
                                                }.into_any())
                                            } else {
                                                None
                                            }
                                        }}
                                    </div>
                                    <div class="flex gap-3">
                                        <button
                                            on:click=handle_submit
                                            disabled=move || !counterexample_valid.get()
                                            class="px-4 py-2 rounded text-sm text-white disabled:bg-dhbw-gray-25 disabled:text-dhbw-gray-50"
                                            class:bg-dhbw-red=move || counterexample_valid.get()
                                            class:hover:bg-red-700=move || counterexample_valid.get()
                                        >
                                            Accept
                                        </button>
                                    </div>
                                </div>
                            }.into_any()
                        }
                        ViewState::Validating => {
                            view! {
                                <div class="flex flex-col h-full justify-between">
                                    <p class="text-dhbw-gray">Is the following implication valid?</p>
                                    <div class="flex items-center text-dhbw-gray font-mono">
                                        {move || {
                                            machine.with(|m| {
                                                context.with(|ctx| {
                                                    let premise: Vec<String> = m.temp_set.iter()
                                                        .filter(|&i| i < ctx.attributes.len())
                                                        .map(|i| ctx.attributes[i].clone())
                                                        .collect();
                                                    format!("{{{}}}", premise.join(", "))
                                                })
                                            })
                                        }}
                                        <span class="text-black px-2">{"→"}</span>
                                        {move || {
                                            machine.with(|m| {
                                                context.with(|ctx| {
                                                    let conclusion: Vec<String> = m.temp_hull.difference(&m.temp_set)
                                                        .filter(|&i| i < ctx.attributes.len())
                                                        .map(|i| ctx.attributes[i].clone())
                                                        .collect();
                                                    format!("{{{}}}", conclusion.join(", "))
                                                })
                                            })
                                        }}
                                    </div>
                                    <div class="flex gap-3">
                                        <button
                                            on:click=handle_yes
                                            class="px-4 py-2 bg-dhbw-red text-white rounded hover:bg-red-700 text-sm"
                                        >
                                            Yes
                                        </button>
                                        <button
                                            on:click=handle_no
                                            class="px-4 py-2 bg-dhbw-gray/5 border border-dhbw-gray-25 rounded hover:bg-dhbw-gray/10 text-dhbw-gray text-sm"
                                        >
                                            No
                                        </button>
                                    </div>
                                </div>
                            }.into_any()
                        }
                    }
                }}
            </div>
        </div>
    }
}

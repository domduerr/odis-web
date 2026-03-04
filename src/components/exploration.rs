use leptos::prelude::*;

use bit_set::BitSet;
use odis::FormalContext;

use crate::core::exploration_state::{ExplorationMachine, ExplorationState};

#[derive(Clone, PartialEq)]
enum ViewState {
    Start,
    Validating,
    Counterexample,
    Finished,
}

#[component]
pub fn ExplorationComp(
    row_key: RwSignal<usize>,
    object_names: RwSignal<Vec<NodeRef<leptos::html::Input>>>,
) -> impl IntoView {
    let context = use_context::<RwSignal<FormalContext<String>>>().expect("Context not provided");

    let machine = RwSignal::new(ExplorationMachine::new());
    let view_state = RwSignal::new(ViewState::Start);

    let checkboxes: RwSignal<Vec<(usize, RwSignal<bool>)>> = RwSignal::new(Vec::new());
    let box_key: RwSignal<usize> = RwSignal::new(0);
    let new_object: NodeRef<leptos::html::Input> = NodeRef::new();

    let start_exploration = move |_| {
        let ctx = context.get();
        machine.update(|m| {
            let _ = m.process_input(
                &ctx,
                crate::core::exploration_state::ExplorationInput::Start,
            );
        });

        let state = machine.with(|m| m.state.clone());
        match state {
            ExplorationState::ValidatingImplication { .. } => view_state.set(ViewState::Validating),
            ExplorationState::Finished => view_state.set(ViewState::Finished),
            _ => {}
        }
    };

    let handle_yes = move |_| {
        let ctx = context.get();
        machine.update(|m| {
            let _ = m.process_input(&ctx, crate::core::exploration_state::ExplorationInput::Yes);
        });

        let state = machine.with(|m| m.state.clone());
        match state {
            ExplorationState::ValidatingImplication { .. } => view_state.set(ViewState::Validating),
            ExplorationState::Finished => view_state.set(ViewState::Finished),
            _ => {}
        }
    };

    let handle_no = move |_| {
        checkboxes.set(Vec::new());
        let attr_len = context.with(|ctx| ctx.attributes.len());
        for n in 0..attr_len {
            let is_checked = machine.with(|m| m.temp_set.contains(n));
            checkboxes
                .write()
                .push((box_key.get(), RwSignal::new(is_checked)));
            *box_key.write() += 1;
        }
        view_state.set(ViewState::Counterexample);
    };

    let handle_submit = move |_| {
        let mut attribute_set = BitSet::new();
        for item in checkboxes.get().iter().enumerate() {
            if item.1 .1.get() {
                attribute_set.insert(item.0);
            }
        }

        row_key.update(|key| *key += 1);

        context.update(|ctx| {
            if let Some(el) = new_object.get() {
                ctx.add_object(el.value(), &attribute_set);
            }
        });

        object_names.update(|list| list.push(NodeRef::new()));

        let ctx = context.get();
        machine.update(|m| {
            let _ = m.process_input(
                &ctx,
                crate::core::exploration_state::ExplorationInput::Submit {
                    counterexample: attribute_set,
                },
            );
        });

        let state = machine.with(|m| m.state.clone());
        match state {
            ExplorationState::ValidatingImplication { .. } => view_state.set(ViewState::Validating),
            ExplorationState::Finished => view_state.set(ViewState::Finished),
            _ => view_state.set(ViewState::Start),
        }
    };

    let handle_stop = move |_| {
        machine.update(|m| m.reset());
        view_state.set(ViewState::Start);
        checkboxes.set(Vec::new());
    };

    let handle_exit = move |_| {
        machine.update(|m| m.reset());
        view_state.set(ViewState::Start);
        checkboxes.set(Vec::new());
    };

    view! {
        <button
            on:click=start_exploration
            class="px-4 py-2 bg-dhbw-gray/5 border border-dhbw-gray-25 rounded hover:bg-dhbw-gray/10 text-dhbw-gray text-sm"
            class:hidden=move || view_state.get() != ViewState::Start
        >
            Start Exploration
        </button>

        <div
            class="fixed inset-0 bg-dhbw-gray-50/60 z-10"
            class:hidden=move || view_state.get() == ViewState::Start
        />

        <div
            class="fixed inset-0 z-11 flex items-center justify-center"
            class:hidden=move || view_state.get() != ViewState::Validating
        >
            <div class="bg-white border border-dhbw-gray-25 rounded-lg shadow-xl p-6 max-w-md w-full mx-4">
                <h3 class="text-dhbw-gray font-semibold text-lg mb-4">Validation</h3>
                <p class="text-dhbw-gray mb-4">Is the following implication valid?</p>
                <div class="bg-dhbw-gray-5 p-4 rounded mb-4 font-mono text-sm">
                    <p class="mb-2">{move || {
                        machine.with(|m| {
                            context.with(|ctx| {
                                let mut premise_string: Vec<String> = Vec::new();
                                for index in &m.temp_set {
                                    if index < ctx.attributes.len() {
                                        premise_string.push(ctx.attributes[index].to_string());
                                    }
                                }
                                format!("{:?}", premise_string)
                            })
                        })
                    }}</p>
                    <p class="text-dhbw-red font-semibold">=></p>
                    <p>{move || {
                        machine.with(|m| {
                            context.with(|ctx| {
                                let conclusion = m.temp_hull.difference(&m.temp_set).collect::<BitSet>();
                                let mut conclusion_string: Vec<String> = Vec::new();
                                for index in &conclusion {
                                    if index < ctx.attributes.len() {
                                        conclusion_string.push(ctx.attributes[index].to_string());
                                    }
                                }
                                format!("{:?}", conclusion_string)
                            })
                        })
                    }}</p>
                </div>

                <div class="flex gap-3">
                    <button
                        on:click=handle_yes
                        class="flex-1 px-4 py-2 bg-dhbw-red text-white rounded hover:bg-red-700 text-sm"
                    >
                        Yes
                    </button>

                    <button
                        on:click=handle_no
                        class="flex-1 px-4 py-2 bg-dhbw-gray/5 border border-dhbw-gray-25 rounded hover:bg-dhbw-gray/10 text-dhbw-gray text-sm"
                    >
                        No
                    </button>
                </div>

                <button
                    on:click=handle_stop
                    class="w-full mt-3 px-4 py-2 border border-dhbw-red/50 text-dhbw-red rounded hover:bg-dhbw-red/5 text-sm"
                >
                    Stop exploration
                </button>
            </div>
        </div>

        <div
            class="fixed inset-0 z-11 flex items-center justify-center overflow-auto"
            class:hidden=move || view_state.get() != ViewState::Counterexample
        >
            <div class="bg-white border border-dhbw-gray-25 rounded-lg shadow-xl p-6 max-w-lg w-full mx-4 my-8">
                <h3 class="text-dhbw-gray font-semibold text-lg mb-4">Provide a counterexample</h3>

                <div class="overflow-auto border border-dhbw-gray-25 rounded mb-4">
                    <table class="bg-gray-100">
                        <tbody>
                            <tr>
                                <td class="p-2 border-r border-dhbw-gray-25"></td>
                                <For
                                    each=move || context.with(|ctx| 0..ctx.attributes.len())
                                    key=move |key| *key
                                    children=move |index| {
                                        view! {
                                            <td class="p-2 text-sm font-medium text-dhbw-gray whitespace-nowrap">
                                                <p>{move || context.with(|ctx| ctx.attributes[index].to_string())}</p>
                                            </td>
                                        }
                                    }
                                />
                            </tr>

                            <tr>
                                <td class="p-2 border-r border-dhbw-gray-25">
                                    <input
                                        id="enter_object_name"
                                        type="text"
                                        placeholder="Enter object name..."
                                        node_ref=new_object
                                        class="w-40 px-2 py-1 text-sm border border-dhbw-gray-25 rounded focus:outline-none focus:border-dhbw-red"
                                    />
                                </td>

                                <For
                                    each=move || checkboxes.get()
                                    key=move |key| key.0
                                    children=move |checkbox| {
                                        view! {
                                            <td class="p-2 text-center">
                                                <input
                                                    type="checkbox"
                                                    bind:checked=checkbox.1
                                                    class="accent-dhbw-red cursor-pointer"
                                                />
                                            </td>
                                        }
                                    }
                                />
                            </tr>
                        </tbody>
                    </table>
                </div>

                <button
                    on:click=handle_submit
                    class="w-full px-4 py-2 bg-dhbw-gray text-white rounded hover:bg-dhbw-gray/80 text-sm"
                >
                    Submit
                </button>
            </div>
        </div>

        <div
            class="fixed inset-0 z-11 flex items-center justify-center"
            class:hidden=move || view_state.get() != ViewState::Finished
        >
            <div class="bg-white border border-dhbw-gray-25 rounded-lg shadow-xl p-6 max-w-sm w-full mx-4 text-center">
                <div class="mb-4">
                    <svg class="w-16 h-16 text-green-500 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                    </svg>
                </div>
                <h3 class="text-dhbw-gray font-semibold text-lg mb-2">Exploration Complete</h3>
                <p class="text-dhbw-gray-50 mb-4">Attribute exploration complete.</p>

                <button
                    on:click=handle_exit
                    class="w-full px-4 py-2 bg-dhbw-gray text-white rounded hover:bg-dhbw-gray/80 text-sm"
                >
                    Exit
                </button>
            </div>
        </div>
    }
}

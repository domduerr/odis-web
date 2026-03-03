use leptos::{logging, prelude::*};

use bit_set::BitSet;
use odis::{self, algorithms::canonical_basis, FormalContext};

use crate::components::table::Table;

#[component]
pub fn ExplorationComp(
    context: RwSignal<FormalContext<String>>,
    table: RwSignal<Table>,
    row_key: RwSignal<usize>,
    object_names: RwSignal<Vec<NodeRef<leptos::html::Input>>>,
) -> impl IntoView {
    let show_question_1 = RwSignal::new(false);
    let show_question_2 = RwSignal::new(false);
    let show_finished = RwSignal::new(false);
    let input_block = RwSignal::new(false);

    let start_node = NodeRef::new();

    let basis: RwSignal<Vec<(BitSet, BitSet)>> = RwSignal::new(Vec::new());
    let temp_set: RwSignal<BitSet> = RwSignal::new(BitSet::new());
    let temp_set_hull: RwSignal<BitSet> = RwSignal::new(BitSet::new());

    let break_while_2 = RwSignal::new(false);

    let new_object: NodeRef<leptos::html::Input> = NodeRef::new();
    let checkboxes: RwSignal<Vec<(usize, RwSignal<bool>)>> = RwSignal::new(Vec::new());
    let box_key: RwSignal<usize> = RwSignal::new(0);

    view! {
        <button
            node_ref=start_node
            on:click=move |_| {
                input_block.set(true);

                while temp_set.get() != (0..context.get().attributes.len()).collect() {

                    *temp_set_hull.write() = context.get().index_attribute_hull(&temp_set.get());

                    if temp_set.get() != temp_set_hull.get() && !break_while_2.get() {

                        show_question_1.set(true);
                        break;

                    } else {

                        break_while_2.set(false);
                        *temp_set.write() = canonical_basis::index_next_preclosure(&context.get(), &basis.get(), &temp_set.get());

                    }
                }
                if temp_set.get() == (0..context.get().attributes.len()).collect() {
                    show_finished.set(true);
                }
        } class="px-4 py-2 bg-dhbw-gray/5 border border-dhbw-gray-25 rounded hover:bg-dhbw-gray/10 text-dhbw-gray text-sm">
            Start Exploration
        </button>

        <div
            class="fixed inset-0 bg-dhbw-gray-50/60 z-10"
            class:hidden=move || !input_block.get()
        />

        <div
            class="fixed inset-0 z-11 flex items-center justify-center"
            class:hidden=move || !show_question_1.get()
        >
            <div class="bg-white border border-dhbw-gray-25 rounded-lg shadow-xl p-6 max-w-md w-full mx-4">
                <h3 class="text-dhbw-gray font-semibold text-lg mb-4">Validation</h3>
                <p class="text-dhbw-gray mb-4">Is the following implication valid?</p>
                <div class="bg-dhbw-gray-5 p-4 rounded mb-4 font-mono text-sm">
                    <p class="mb-2">{move || {
                        let mut premise_string: Vec<String> = Vec::new();
                        for index in &temp_set.get() {
                            premise_string.push(context.get().attributes[index].to_string());
                        }
                        format!("{:?}", premise_string)
                    }}</p>
                    <p class="text-dhbw-red font-semibold">=></p>
                    <p>{move || {
                        let mut conclusion_stirng: Vec<String> = Vec::new();
                        for index in &temp_set_hull.get().difference(&temp_set.get()).collect::<BitSet>() {
                            conclusion_stirng.push(context.get().attributes[index].to_string());
                        }
                        format!("{:?}", conclusion_stirng)
                    }}</p>
                </div>

                <div class="flex gap-3">
                    <button
                        on:click=move |_| {
                            basis.write().push((temp_set.get(), temp_set_hull.get()));
                            break_while_2.set(true);
                            show_question_1.set(false);
                            start_node.get().unwrap().click();
                    } class="flex-1 px-4 py-2 bg-dhbw-red text-white rounded hover:bg-red-700 text-sm">
                        Yes
                    </button>

                    <button
                        on:click=move |_| {
                            checkboxes.set(Vec::new());
                            for n in 0..context.get().attributes.len() {
                                if temp_set.get().contains(n) {
                                    checkboxes.write().push((box_key.get(), RwSignal::new(true)));
                                    *box_key.write() += 1;
                                } else {
                                    checkboxes.write().push((box_key.get(), RwSignal::new(false)));
                                    *box_key.write() += 1;
                                }
                            }

                            show_question_1.set(false);
                            show_question_2.set(true);
                    } class="flex-1 px-4 py-2 bg-dhbw-gray/5 border border-dhbw-gray-25 rounded hover:bg-dhbw-gray/10 text-dhbw-gray text-sm">
                        No
                    </button>
                </div>

                <button
                    on:click=move |_| {
                        input_block.set(false);
                        show_question_1.set(false);

                        basis.set(Vec::new());
                        temp_set.set(BitSet::new());
                        temp_set_hull.set(BitSet::new());

                        break_while_2.set(false);

                        checkboxes.set(Vec::new());
                } class="w-full mt-3 px-4 py-2 border border-dhbw-red/50 text-dhbw-red rounded hover:bg-dhbw-red/5 text-sm">
                    Stop exploration
                </button>
            </div>
        </div>


        <div
            class="fixed inset-0 z-11 flex items-center justify-center overflow-auto"
            class:hidden=move || !show_question_2.get()
        >
            <div class="bg-white border border-dhbw-gray-25 rounded-lg shadow-xl p-6 max-w-lg w-full mx-4 my-8">
                <h3 class="text-dhbw-gray font-semibold text-lg mb-4">Provide a counterexample</h3>

                <div class="overflow-auto border border-dhbw-gray-25 rounded mb-4">
                    <table class="bg-gray-100">
                        <tbody>
                            <tr>
                                <td class="p-2 border-r border-dhbw-gray-25"></td>
                                <For
                                    each=move || 0..context.get().attributes.len()
                                    key=move |key| *key
                                    children=move |index| {
                                        view! {
                                            <td class="p-2 text-sm font-medium text-dhbw-gray whitespace-nowrap">
                                                <p>{move || context.get().attributes[index].to_string()}</p>
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
                                        on:change=move |_| {

                                        }
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
                    on:click=move |_| {
                        let mut attribute_set = BitSet::new();
                        for item in checkboxes.get().iter().enumerate() {
                            if item.1.1.get() {
                                attribute_set.insert(item.0);
                            }
                        }

                        let num_obj = table.get().row_data.last().unwrap().1;

                        table.update(|table| {
                            table.row_data.push((row_key.get(), num_obj + 1))
                        });
                        row_key.update(|key| *key += 1);

                        for n in 0..table.read_only().get().column_data.len() {
                            table.update(|table| {
                                table.boxes.insert((num_obj + 1, n), RwSignal::new(checkboxes.get()[n].1.get()));
                            });
                        };

                        context.update(|context| {
                            context.add_object(new_object.get().unwrap().value(), &attribute_set);
                        });

                        object_names.update(|list| list.push(NodeRef::new()));

                        show_question_2.set(false);
                        let a = context.get().atomic_attribute_derivations;
                        let b = context.get().atomic_object_derivations;

                        logging::log!("Atomic attr: {:?}\n", a);
                        logging::log!("Atomic obj: {:?}\n", b);
                        start_node.get().unwrap().click();
                    }
                    class="w-full px-4 py-2 bg-dhbw-gray text-white rounded hover:bg-dhbw-gray/80 text-sm"
                >
                    Submit
                </button>
            </div>
        </div>

        <div
            class="fixed inset-0 z-11 flex items-center justify-center"
            class:hidden=move || !show_finished.get()
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
                    on:click=move |_| {
                        show_finished.set(false);
                        input_block.set(false);

                        basis.set(Vec::new());
                        temp_set.set(BitSet::new());
                        temp_set_hull.set(BitSet::new());

                        break_while_2.set(false);

                        checkboxes.set(Vec::new());
                    }
                    class="w-full px-4 py-2 bg-dhbw-gray text-white rounded hover:bg-dhbw-gray/80 text-sm"
                >
                    Exit
                </button>
            </div>
        </div>
    }
}

use leptos::{logging, prelude::*};

use bit_set::BitSet;
use odis::{self, FormalContext, algorithms::canonical_basis};

use crate::components::table::Table;

#[component]
pub fn ExplorationComp(
    context: RwSignal<FormalContext<String>>,
    table: RwSignal<Table>,
    row_key: RwSignal<usize>,
    object_names: RwSignal<Vec<NodeRef<leptos::html::Input>>>,
) -> impl IntoView {
    let show_question_1 = RwSignal::new("none");
    let show_question_2 = RwSignal::new("none");
    let show_finished = RwSignal::new("none");
    let input_block = RwSignal::new("none");

    let start_node = NodeRef::new();

    let basis: RwSignal<Vec<(BitSet, BitSet)>> = RwSignal::new(Vec::new());
    let temp_set: RwSignal<BitSet> = RwSignal::new(BitSet::new());
    let temp_set_hull = RwSignal::new(BitSet::new());

    let break_while_2 = RwSignal::new(false);

    let new_object: NodeRef<leptos::html::Input> = NodeRef::new();
    let checkboxes: RwSignal<Vec<(usize, RwSignal<bool>)>> = RwSignal::new(Vec::new());
    let box_key: RwSignal<usize> = RwSignal::new(0);

    view! {
        <button
            node_ref=start_node
            on:click=move |_| {
                input_block.set("block");

                while temp_set.get() != (0..context.get().attributes.len()).collect() {

                    *temp_set_hull.write() = context.get().index_attribute_hull(&temp_set.get());

                    if temp_set.get() != temp_set_hull.get() && !break_while_2.get() {

                        show_question_1.set("block");
                        break;

                    } else {

                        break_while_2.set(false);
                        *temp_set.write() = canonical_basis::next_preclosure(&context.get(), &basis.get(), &temp_set.get());

                    }
                }
                if temp_set.get() == (0..context.get().attributes.len()).collect() {
                    show_finished.set("block");
                }
        }>"Start Exploration"</button>

        <div
            style:opacity="0.6"
            style:background-color="#ccc"
            style:position="fixed"
            style:width="100%"
            style:height="100%"
            style:top="0px"
            style:left="0px"
            style:z-index="10"
            style:display=input_block
        />

        // Question: 1
        <div
            style:position="absolute"
            style:top="50%"
            style:left="50%"
            style:transform="translate(-50%, -50%)"
            style:background="white"
            style:border="thin solid black"
            style:z-index="11"
            style:display=show_question_1
        >
            <div
                style:margin="10px"
            >
                // Question text
                <p>"Is the following implication valid?"</p>
                <p>{move || {
                    let mut premise_string: Vec<String> = Vec::new();
                    for index in &temp_set.get() {
                        premise_string.push(context.get().attributes[index].to_string());
                    }
                    format!("{:?}", premise_string)
                }}</p>
                <p>"=>"</p>
                <p>{move || {
                    let mut conclusion_stirng: Vec<String> = Vec::new();
                    for index in &temp_set_hull.get().difference(&temp_set.get()).collect::<BitSet>() {
                        conclusion_stirng.push(context.get().attributes[index].to_string());
                    }
                    format!("{:?}", conclusion_stirng)
                }}</p>

                <button
                    on:click=move |_| {
                        basis.write().push((temp_set.get(), temp_set_hull.get()));
                        break_while_2.set(true);
                        show_question_1.set("none");
                        start_node.get().unwrap().click();
                }>"Yes"</button>

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

                        show_question_1.set("none");
                        show_question_2.set("block");
                }>"No"</button>

                <button
                    on:click=move |_| {
                        input_block.set("none");
                        show_question_1.set("none");

                        basis.set(Vec::new());
                        temp_set.set(BitSet::new());
                        temp_set_hull.set(BitSet::new());

                        break_while_2.set(false);

                        checkboxes.set(Vec::new());
                }>"Stop exploration"</button>
            </div>
        </div>


        // Question: 2
        <div
            style:position="absolute"
            style:top="50%"
            style:left="50%"
            style:transform="translate(-50%, -50%)"
            style:background="white"
            style:border="thin solid black"
            style:z-index="11"
            style:display=show_question_2
        >
            <div
                style:margin="10px"
            >
                <p>"Provide a counterexample:"</p>

                <table
                    style:background="#D3D3D3"
                >
                    <tbody>
                        <tr>
                            <td/>
                            <For
                                each=move || (0..context.get().attributes.len())
                                key=move |key| *key
                                children=move |index| {
                                    view! {
                                        <td>
                                            <p>
                                                {format!("{}", context.get().attributes[index].clone())}
                                            </p>
                                        </td>
                                    }
                                }
                            />
                        </tr>

                        <tr>
                            <td>
                                <input
                                    type="text"
                                    placeholder="Enter object name..."
                                    on:change=move |_| {

                                    }
                                    node_ref=new_object
                                />
                            </td>

                            <For
                                each=move || checkboxes.get()
                                key=move |key| key.0
                                children=move |checkbox| {
                                    view! {
                                        <td>
                                            <input
                                                type="checkbox"
                                                bind:checked=checkbox.1
                                            />
                                        </td>
                                    }
                                }
                            />
                        </tr>
                    </tbody>
                </table>

                <br/>
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

                        show_question_2.set("none");
                        let a = context.get().atomic_attribute_derivations;
                        let b = context.get().atomic_object_derivations;

                        logging::log!("Atomic attr: {:?}\n", a);
                        logging::log!("Atomic obj: {:?}\n", b);
                        start_node.get().unwrap().click();
                    }
                >"Submit"</button>
            </div>
        </div>

        <div
            style:position="absolute"
            style:top="50%"
            style:left="50%"
            style:transform="translate(-50%, -50%)"
            style:background="white"
            style:border="thin solid black"
            style:z-index="11"
            style:display=show_finished
        >
            <div
                style:margin="10px"
                style:text-align="center"
            >
                <p>"Attribute exploration complete."</p>

                <button
                    on:click=move |_| {
                        show_finished.set("none");
                        input_block.set("none");

                        basis.set(Vec::new());
                        temp_set.set(BitSet::new());
                        temp_set_hull.set(BitSet::new());

                        break_while_2.set(false);

                        checkboxes.set(Vec::new());
                    }
                >"Exit"</button>
            </div>
        </div>
    }
}

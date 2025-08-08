use leptos::{either::Either, prelude::*};

use bit_set::BitSet;
use odis::FormalContext;
use std::collections::HashMap;
use web_sys::MouseEvent;

use crate::components::{
    checkbox::CheckboxComp, download::DownloadComp, exploration::ExplorationComp, graph::GraphComp,
};

#[derive(Debug, Clone)]
pub struct Table {
    // (key, position)
    pub row_data: Vec<(usize, usize)>,
    // (key, position)
    pub column_data: Vec<(usize, usize)>,
    // ((pos_row, pos_column), checked)
    pub boxes: HashMap<(usize, usize), RwSignal<bool>>,
}

#[component]
pub fn TableComp(context: RwSignal<Option<FormalContext<String>>>) -> impl IntoView {
    let mut temp_context = RwSignal::new(FormalContext::new());
    if let Some(n) = context.get() {
        temp_context = RwSignal::new(n);
    } else {
        for n in 0..5 {
            temp_context
                .write()
                .add_object(format!("Object {}", n), &BitSet::new());
            temp_context
                .write()
                .add_attribute(format!("Attribute {}", n), &BitSet::new());
        }
    }
    let context = temp_context;

    let row_key = RwSignal::new(0);
    let column_key = RwSignal::new(0);
    let table = RwSignal::new(Table {
        row_data: Vec::new(),
        column_data: Vec::new(),
        boxes: HashMap::new(),
    });

    let object_names: RwSignal<Vec<NodeRef<leptos::html::Input>>> = RwSignal::new(Vec::new());
    let attribute_names: RwSignal<Vec<NodeRef<leptos::html::Input>>> = RwSignal::new(Vec::new());

    let concepts = RwSignal::new(None);
    let concept_lattice = RwSignal::new(false);
    let basis = RwSignal::new(None);

    let focus_pos: RwSignal<(usize, usize)> = RwSignal::new((0, 0));
    let delete_hover_obj = RwSignal::new(false);
    let delete_hover_attr = RwSignal::new(false);

    for g in 0..context.get_untracked().objects.len() {
        table.update(|table| {
            table.row_data.push((row_key.get_untracked(), g));
        });
        row_key.update(|key| *key += 1);

        object_names.update(|list| list.push(NodeRef::new()));
    }

    for m in 0..context.get_untracked().attributes.len() {
        table.update(|table| {
            table.column_data.push((column_key.get_untracked(), m));
        });
        column_key.update(|key| *key += 1);

        attribute_names.update(|list| list.push(NodeRef::new()));
    }

    for g in 0..context.get_untracked().objects.len() {
        for m in 0..context.get_untracked().attributes.len() {
            if context.get_untracked().incidence.contains(&(g, m)) {
                table.update(|table| {
                    table.boxes.insert((g, m), RwSignal::new(true));
                });
            } else {
                table.update(|table| {
                    table.boxes.insert((g, m), RwSignal::new(false));
                });
            }
        }
    }

    let add_object = move |_| {
        let num_obj = table.get().row_data.last().unwrap().1;

        table.update(|table| table.row_data.push((row_key.get(), num_obj + 1)));
        row_key.update(|key| *key += 1);

        for n in 0..table.read_only().get().column_data.len() {
            table.update(|table| {
                table.boxes.insert((num_obj + 1, n), RwSignal::new(false));
            });
        }

        context.update(|context| {
            context.add_object("Object".to_string(), &BitSet::new());
        });

        object_names.update(|list| list.push(NodeRef::new()));
    };

    let remove_object = move |_| {
        if table.read_only().get().row_data.len() <= 1 {
            return;
        };

        let index = focus_pos.read_only().get().0;

        for attr in 0..table.read_only().get().column_data.len() {
            table.update(|table| {
                table.boxes.remove(&(index, attr));
            });
            for obj in (index + 1)..table.read_only().get().row_data.len() {
                table.update(|table| {
                    let signal = table.boxes.remove(&(obj, attr)).unwrap();
                    table.boxes.insert((obj - 1, attr), signal);
                });
            }
        }

        table.update(|table| {
            table.row_data.remove(index);
            table.row_data = table
                .row_data
                .iter()
                .map(|x| {
                    if x.1 > index {
                        let new_row = (row_key.get(), x.1 - 1);
                        row_key.update(|key| *key += 1);
                        new_row
                    } else {
                        *x
                    }
                })
                .collect::<Vec<(usize, usize)>>();
        });

        context.update(|context| {
            context.remove_object(index);
        });

        object_names.update(|list| {
            list.remove(index);
        });

        if focus_pos.read_only().get().0 >= table.read_only().get().row_data.len() {
            focus_pos.update(|pos| *pos = (table.read_only().get().row_data.len() - 1, pos.1));
        }
    };

    let add_attribute = move |_| {
        let num_attr = table.get().column_data.last().unwrap().1;

        table.update(|table| table.column_data.push((column_key.get(), num_attr + 1)));
        column_key.update(|key| *key += 1);

        for n in 0..table.read_only().get().row_data.len() {
            table.update(|table| {
                table.boxes.insert((n, num_attr + 1), RwSignal::new(false));
            });
        }

        context.update(|context| {
            context.add_attribute("Attribute".to_string(), &BitSet::new());
        });

        attribute_names.update(|list| list.push(NodeRef::new()));
    };

    let remove_attribute = move |_| {
        if table.read_only().get().column_data.len() <= 1 {
            return;
        };

        let index = focus_pos.read_only().get().1;

        for obj in 0..table.read_only().get().row_data.len() {
            table.update(|table| {
                table.boxes.remove(&(obj, index));
            });
            for attr in (index + 1)..table.read_only().get().column_data.len() {
                table.update(|table| {
                    let signal = table.boxes.remove(&(obj, attr)).unwrap();
                    table.boxes.insert((obj, attr - 1), signal);
                });
            }
        }

        table.update(|table| {
            table.column_data.remove(index);
            table.column_data = table
                .column_data
                .iter()
                .map(|x| {
                    if x.1 > index {
                        let new_column = (column_key.get(), x.1 - 1);
                        column_key.update(|key| *key += 1);
                        new_column
                    } else {
                        *x
                    }
                })
                .collect::<Vec<(usize, usize)>>();
        });

        context.update(|context| {
            context.remove_attribute(index);
        });

        attribute_names.update(|list| {
            list.remove(index);
        });

        if focus_pos.read_only().get().1 >= table.read_only().get().column_data.len() {
            focus_pos.update(|pos| *pos = (pos.0, table.read_only().get().column_data.len() - 1));
        }
    };

    let calc_concepts = move |_| {
        let mut result: Vec<(BitSet, BitSet)> =
            context.read_only().get().fcbo_index_concepts().collect();
        context.get().sort_lectic_order(&mut result);
        concepts.set(Some(result));
    };

    let calc_basis = move |_| {
        let result = context.read_only().get().canonical_basis();
        basis.set(Some(result));
    };

    view! {
        <DownloadComp context=context/>
        <br/><br/><br/>

        <button on:click=add_object>"Add Object"</button>
        <button
            on:click=remove_object
            on:mouseover=move |_| {delete_hover_obj.set(true)}
            on:mouseout=move |_| {delete_hover_obj.set(false)}
        >"Remove Object"</button>
        <br/><br/>
        <button on:click=add_attribute>"Add Attribute"</button>
        <button
            on:click=remove_attribute
            on:mouseover=move |_| {delete_hover_attr.set(true)}
            on:mouseout=move |_| {delete_hover_attr.set(false)}
        >"Remove Attribute"</button>
        <br/><br/>

        <table
            style:background="#D3D3D3"
            style:max-width="100%"
            style:table-layout="auto"
        >
            <tbody>
                <tr>
                    <td></td> // top left corner
                    <For
                        each=move || table.get().column_data
                        key=move |key| key.0
                        children=move |column| {
                            view! {
                                <td>
                                    <input type="text" style:width="120px"
                                        on:focus=move |_| {
                                            focus_pos.update(|pos| {
                                                *pos = (pos.0, column.1)
                                            });
                                        }
                                        on:change=move |_| {
                                            let name = attribute_names.read_only().get()[column.1].get().unwrap().value();
                                            context.update(|context| context.change_attribute_name(name, column.1));
                                        }
                                        value=context.get().attributes[column.1].clone()
                                        node_ref=attribute_names.get()[column.1]
                                    />
                                </td>
                            }
                        }
                    />
                </tr>
                <For
                    each=move || table.get().row_data
                    key=|key| key.0
                    children=move |row| {
                        view! {
                            <tr>
                                <td>
                                    <input type="text" style:width="120px"
                                        on:focus=move |_| {
                                            focus_pos.update(|pos| {
                                                *pos = (row.1, pos.1)
                                            });
                                        }
                                        on:change=move |_| {
                                            let name = object_names.get()[row.1].get().unwrap().value();
                                            context.update(|context| context.change_object_name(name, row.1));
                                        }
                                        value=context.get().objects[row.1].clone()
                                        node_ref=object_names.get()[row.1]
                                    />
                                </td>
                                <For
                                    each=move || table.get().column_data
                                    key=|column| column.0
                                    children=move |column| {
                                        view! {
                                            <td
                                                style:text-align="center"
                                                // style:border="1px solid black"
                                                style:background-color=move || {
                                                    if delete_hover_obj.get() && row.1 == focus_pos.get().0 {
                                                        "lightblue"
                                                    } else if delete_hover_attr.get() && column.1 == focus_pos.get().1 {
                                                        "lightblue"
                                                    } else if (row.1, column.1) == focus_pos.get() {
                                                        "lightblue"
                                                    } else {
                                                        "#D3D3D3"
                                                    }
                                                }
                                                on:click=move |_| {
                                                    focus_pos.update(|pos| *pos = (row.1, column.1));
                                                }
                                            >
                                                <CheckboxComp context=context row=row.1 column=column.1 table=table position=focus_pos/>
                                            </td>
                                        }
                                    }
                                />
                            </tr>
                        }
                    }
                />
            </tbody>
        </table>
        <br/>
        <div style:display="flex">
            <div style:min-width="200px" style:max-width="40%">
                <button on:click=calc_concepts>"Compute Concepts"</button>
                {move || {
                    if let Some(n) = concepts.get() {
                        let concepts_clone: Vec<(usize, (BitSet, BitSet))> = concepts.get().unwrap().into_iter().enumerate().collect();
                        Either::Left(view! {
                            <p>{format!("The number of concepts is: {}", n.len())}</p>
                            <ul style:max-height="300px" style:overflow-y="scroll">
                                <For
                                    each=move || concepts_clone.clone()
                                    key=|key| key.0
                                    children=move |concept| {
                                        view! {
                                            <li style:white-space="pre">
                                                {
                                                    let mut obj_string = String::new();
                                                    obj_string.push('{');

                                                    for n in &concept.1.0 {
                                                        obj_string.push_str(
                                                            &(" ".to_string() + &context.get().objects[n] + " ,")
                                                        );
                                                    }

                                                    if concept.1.0.len() > 0 {
                                                        obj_string.pop();
                                                    } else {
                                                        obj_string.push(' ');
                                                    }
                                                    obj_string.push('}');

                                                    let mut white_spaces = String::from("   ");
                                                    if concept.0 >= 9 {
                                                        white_spaces.truncate(1);
                                                    }

                                                    format!("{}:{}{},", concept.0 + 1, white_spaces, obj_string)
                                                }
                                                <br/>
                                                {
                                                    let mut attr_string = String::new();
                                                    attr_string.push('{');

                                                    for n in &concept.1.1 {
                                                        attr_string.push_str(
                                                            &(" ".to_string() + &context.get().attributes[n] + " ,")
                                                        );
                                                    }
                                                    if concept.1.1.len() > 0 {
                                                        attr_string.pop();
                                                    } else {
                                                        attr_string.push(' ');
                                                    }
                                                    attr_string.push('}');

                                                    let white_spaces = String::from("      ");

                                                    format!("{}{}", white_spaces, attr_string)
                                                }
                                            </li>
                                        }
                                    }
                                />
                            </ul>
                        })
                    } else {
                        Either::Right(view! {<p>"..."</p>})
                    }
                }}
            </div>
            <div style:min-width="200px" style:max-width="40%">
                <button on:click=calc_basis>"Compute Canonical Base"</button>
                {move || {
                    if let Some(n) = basis.get() {
                        let basis_clone: Vec<(usize, (BitSet, BitSet))> = basis.get().unwrap().into_iter().enumerate().collect();
                        Either::Left(view! {
                            <p>{format!("The number of implications is: {}", n.len())}</p>
                            <ul style:max-height="300px" style:overflow-y="scroll">
                                <For
                                    each=move || basis_clone.clone()
                                    key=|key| key.0
                                    children=move |basis| {
                                        view! {
                                            <li style:white-space="pre">
                                                {
                                                    let mut premise = String::new();
                                                    premise.push('{');

                                                    for n in &basis.1.0 {
                                                        premise.push_str(
                                                            &(" ".to_string() + &context.get().attributes[n] + " ,")
                                                        );
                                                    }

                                                    if basis.1.0.len() > 0 {
                                                        premise.pop();
                                                    } else {
                                                        premise.push(' ');
                                                    }
                                                    premise.push('}');

                                                    let mut white_spaces = String::from("   ");
                                                    if basis.0 >= 9 {
                                                        white_spaces.truncate(1);
                                                    }

                                                    format!("{}:{}{},", basis.0 + 1, white_spaces, premise)
                                                }
                                                <br/>
                                                {
                                                    let mut conclusion = String::new();
                                                    conclusion.push('{');

                                                    for n in &basis.1.1 {
                                                        conclusion.push_str(
                                                            &(" ".to_string() + &context.get().attributes[n] + " ,")
                                                        );
                                                    }
                                                    if basis.1.1.len() > 0 {
                                                        conclusion.pop();
                                                    } else {
                                                        conclusion.push(' ');
                                                    }
                                                    conclusion.push('}');

                                                    let white_spaces = String::from("      ");

                                                    format!("{}{}", white_spaces, conclusion)
                                                }
                                            </li>
                                        }
                                    }
                                />
                            </ul>
                        })
                    } else {
                        Either::Right(view! {<p>"..."</p>})
                    }
                }}
            </div>
            <div>
                <ExplorationComp
                    context=context
                    table=table
                    row_key=row_key
                    object_names=object_names
                />
            </div>

        </div>

        <button on:click=move |_| {
            calc_concepts(MouseEvent::new("click").unwrap());
            concept_lattice.set(true);
        }>"Draw Concept Lattice"</button>
        {move || {
            if concept_lattice.get() {
                Either::Left(view! {
                    <GraphComp concepts=concepts.get_untracked().unwrap() context=context.get_untracked()/>
                })
            } else {
                Either::Right(view! {
                    <br/><br/>
                })
            }
        }}
    }
}

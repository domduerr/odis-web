use leptos::prelude::*;

use odis::{self, FormalContext};

use crate::components::table::Table;

#[component]
pub fn CheckboxComp(
    context: RwSignal<FormalContext<String>>,
    row: usize,
    column: usize,
    table: RwSignal<Table>,
    position: RwSignal<(usize, usize)>,
) -> impl IntoView {
    view! {
        <input
            on:click=move |_| {
                position.update(|pos| *pos = (row, column));

                if !table.get().boxes.get(&(row, column)).unwrap().get() {
                    context.update(|context| {
                        context.incidence.insert((row, column));
                        context.atomic_object_derivations[row].insert(column);
                        context.atomic_attribute_derivations[column].insert(row);
                    });
                } else {
                    context.update(|context| {
                        context.incidence.remove(&(row, column));
                        context.atomic_object_derivations[row].remove(column);
                        context.atomic_attribute_derivations[column].remove(row);
                    });
                }
            }
            type="checkbox" bind:checked=*table.get_untracked().boxes.get(&(row, column)).unwrap()
        />
    }
}

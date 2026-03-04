use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

use bit_set::BitSet;
use odis::FormalContext;
use std::collections::HashMap;

use crate::components::context::{create_default_context, index_to_column_name};

#[derive(Debug, Clone)]
pub struct Table {
    pub row_data: Vec<(usize, usize)>,
    pub column_data: Vec<(usize, usize)>,
    pub boxes: HashMap<(usize, usize), RwSignal<bool>>,
}

#[component]
pub fn TableComp(context: RwSignal<Option<FormalContext<String>>>) -> impl IntoView {
    let effective_context =
        Signal::derive(move || context.get().unwrap_or_else(|| create_default_context()));

    let add_object = move |_| {
        let current_context = effective_context.get_untracked();
        let mut new_context = current_context.clone();
        let new_name = format!("{}", current_context.objects.len() + 1);
        new_context.add_object(new_name, &BitSet::new());
        context.set(Some(new_context));
    };

    let remove_object = move |idx: usize| {
        let current_context = effective_context.get_untracked();
        if current_context.objects.len() <= 1 {
            return;
        }

        let mut new_context = current_context.clone();
        new_context.remove_object(idx);
        context.set(Some(new_context));
    };

    let add_attribute = move |_| {
        let current_context = effective_context.get_untracked();
        let mut new_context = current_context.clone();
        let new_name = index_to_column_name(current_context.attributes.len());
        new_context.add_attribute(new_name, &BitSet::new());
        context.set(Some(new_context));
    };

    let remove_attribute = move |idx: usize| {
        let current_context = effective_context.get_untracked();
        if current_context.attributes.len() <= 1 {
            return;
        }

        let mut new_context = current_context.clone();
        new_context.remove_attribute(idx);
        context.set(Some(new_context));
    };

    let change_object_name = move |index: usize, name: String| {
        let current_context = effective_context.get_untracked();
        let mut new_context = current_context.clone();
        new_context.change_object_name(name, index);
        context.set(Some(new_context));
    };

    let change_attribute_name = move |index: usize, name: String| {
        let current_context = effective_context.get_untracked();
        let mut new_context = current_context.clone();
        new_context.change_attribute_name(name, index);
        context.set(Some(new_context));
    };

let toggle_cell = move |obj: usize, attr: usize, value: bool| {
    let current_context = effective_context.get_untracked();
    let mut new_context = current_context.clone();

    if value {
        new_context.incidence.insert((obj, attr));
        // WICHTIG: Ableitungen hinzufügen
        new_context.atomic_object_derivations[obj].insert(attr);
        new_context.atomic_attribute_derivations[attr].insert(obj);
    } else {
        new_context.incidence.remove(&(obj, attr));
        // WICHTIG: Ableitungen entfernen
        new_context.atomic_object_derivations[obj].remove(attr);
        new_context.atomic_attribute_derivations[attr].remove(obj);
    }

    context.set(Some(new_context));
};

    view! {
        <div class="bg-gray-50 min-h-full p-2">
            <div class="overflow-auto">
                <table class="border-collapse">
                    <tbody>
                        <tr>
                            <td class="p-1"></td>
                            {move || {
                                let attrs: Vec<_> = effective_context.get().attributes.iter().cloned().enumerate().collect();
                                let attrs_views: Vec<_> = attrs.into_iter().map(|(col_idx, attr_name)| {
                                    view! {
                                        <td class="p-1 border border-dhbw-gray-25 bg-gray-100">
                                            <div class="flex flex-col items-center">
                                                <button
                                                    on:click=move |_| { remove_attribute(col_idx); }
                                                    class="text-dhbw-gray hover:text-dhbw-red text-sm font-bold"
                                                    title="Delete Attribute"
                                                >x</button>
                                                <span
                                                    class="px-1 text-sm cursor-text hover:bg-gray-50"
                                                    contenteditable=true
                                                    on:keydown=move |ev| {
                                                        if ev.key() == "Enter" {
                                                            ev.prevent_default();
                                                        }
                                                    }
                                                    on:blur=move |ev| {
                                                        let target = ev.target().unwrap();
                                                        let span: web_sys::HtmlElement = target.unchecked_into();
                                                        let new_name = span.text_content().unwrap_or_default().trim().to_string();
                                                        if !new_name.is_empty() {
                                                            change_attribute_name(col_idx, new_name);
                                                        }
                                                    }
                                                >{attr_name.clone()}</span>
                                            </div>
                                        </td>
                                    }
                                }).collect();
                                view! {
                                    <>
                                        {attrs_views}
                                        <td class="p-1 border border-dhbw-gray-25 bg-gray-50">
                                            <button
                                                on:click=add_attribute
                                                class="text-dhbw-gray hover:text-dhbw-red text-lg font-bold px-2"
                                                title="Add Attribute"
                                            >+</button>
                                        </td>
                                    </>
                                }
                            }}
                        </tr>
                        {move || {
                            let objs: Vec<_> = effective_context.get().objects.iter().cloned().enumerate().collect();
                            let attrs_count = effective_context.get().attributes.len();
                            objs.into_iter().map(|(row_idx, obj_name)| {
                                view! {
                                    <tr>
                                        <td class="p-1 border border-dhbw-gray-25 bg-gray-100">
                                            <div class="flex items-center gap-1">
                                                <button
                                                    on:click=move |_| { remove_object(row_idx); }
                                                    class="text-dhbw-gray hover:text-dhbw-red text-sm font-bold"
                                                    title="Delete Object"
                                                >x</button>
                                                <span
                                                    class="px-1 text-sm cursor-text hover:bg-gray-50"
                                                    contenteditable=true
                                                    on:keydown=move |ev| {
                                                        if ev.key() == "Enter" {
                                                            ev.prevent_default();
                                                        }
                                                    }
                                                    on:blur=move |ev| {
                                                        let target = ev.target().unwrap();
                                                        let span: web_sys::HtmlElement = target.unchecked_into();
                                                        let new_name = span.text_content().unwrap_or_default().trim().to_string();
                                                        if !new_name.is_empty() {
                                                            change_object_name(row_idx, new_name);
                                                        }
                                                    }
                                                >{obj_name.clone()}</span>
                                            </div>
                                        </td>
                                        {(0..attrs_count).into_iter().map(|col_idx| {
                                            view! {
                                                <td class="p-1 text-center border border-dhbw-gray-25 bg-white">
                                                    <input
                                                        type="checkbox"
                                                        checked=move || {
                                                            effective_context.get().incidence.contains(&(row_idx, col_idx))
                                                        }
                                                        on:change=move |ev| {
                                                            let input: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into();
                                                            toggle_cell(row_idx, col_idx, input.checked());
                                                        }
                                                        class="accent-dhbw-red cursor-pointer w-4 h-4"
                                                    />
                                                </td>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </tr>
                                }
                            }).collect::<Vec<_>>()
                        }}
                        {move || {
                            view! {
                                <tr>
                                    <td class="p-1 border border-dhbw-gray-25 bg-gray-50 text-center">
                                        <button
                                            on:click=add_object
                                            class="text-dhbw-gray hover:text-dhbw-red text-lg font-bold"
                                            title="Add Object"
                                        >+</button>
                                    </td>
                                </tr>
                            }
                        }}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

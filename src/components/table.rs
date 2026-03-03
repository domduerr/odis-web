use leptos::prelude::*;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

use bit_set::BitSet;
use odis::FormalContext;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Table {
    pub row_data: Vec<(usize, usize)>,
    pub column_data: Vec<(usize, usize)>,
    pub boxes: HashMap<(usize, usize), RwSignal<bool>>,
}

fn create_default_context() -> FormalContext<String> {
    let mut ctx = FormalContext::new();
    for n in 0..5 {
        ctx.add_object(format!("Object {}", n), &BitSet::new());
        ctx.add_attribute(format!("Attribute {}", n), &BitSet::new());
    }
    ctx
}

#[component]
pub fn TableComp(context: RwSignal<Option<FormalContext<String>>>) -> impl IntoView {
    let effective_context =
        Signal::derive(move || context.get().unwrap_or_else(|| create_default_context()));

    let focus_pos: RwSignal<(usize, usize)> = RwSignal::new((0, 0));
    let delete_hover_obj = RwSignal::new(false);
    let delete_hover_attr = RwSignal::new(false);

    let add_object = move |_| {
        let current_context = effective_context.get_untracked();
        let mut new_context = current_context.clone();
        new_context.add_object("Object".to_string(), &BitSet::new());
        context.set(Some(new_context));
    };

    let remove_object = move |_| {
        let current_context = effective_context.get_untracked();
        if current_context.objects.len() <= 1 {
            return;
        }

        let index = focus_pos.get_untracked().0;

        let mut new_context = current_context.clone();
        new_context.remove_object(index);
        context.set(Some(new_context));
    };

    let add_attribute = move |_| {
        let current_context = effective_context.get_untracked();
        let mut new_context = current_context.clone();
        new_context.add_attribute("Attribute".to_string(), &BitSet::new());
        context.set(Some(new_context));
    };

    let remove_attribute = move |_| {
        let current_context = effective_context.get_untracked();
        if current_context.attributes.len() <= 1 {
            return;
        }

        let index = focus_pos.get_untracked().1;

        let mut new_context = current_context.clone();
        new_context.remove_attribute(index);
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
        } else {
            new_context.incidence.remove(&(obj, attr));
        }

        context.set(Some(new_context));
    };

    view! {
        <div class="bg-white min-h-full">
            <div class="mb-6">
                <div class="flex items-center justify-between mb-4">
                    <h2 class="text-dhbw-gray font-semibold text-lg">Cross Table</h2>
                    <div class="flex gap-2">
                        <button on:click=add_object class="px-3 py-1.5 bg-dhbw-red text-white rounded hover:bg-red-700 text-sm flex items-center gap-1">
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path>
                            </svg>
                            Add Object
                        </button>
                        <button
                            on:click=remove_object
                            on:mouseover=move |_| {delete_hover_obj.set(true)}
                            on:mouseout=move |_| {delete_hover_obj.set(false)}
                            class="px-3 py-1.5 border border-dhbw-gray-25 rounded hover:bg-dhbw-gray/10 text-dhbw-gray text-sm flex items-center gap-1"
                        >
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4"></path>
                            </svg>
                            Remove Object
                        </button>
                        <button on:click=add_attribute class="px-3 py-1.5 bg-dhbw-red text-white rounded hover:bg-red-700 text-sm flex items-center gap-1">
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path>
                            </svg>
                            Add Attribute
                        </button>
                        <button
                            on:click=remove_attribute
                            on:mouseover=move |_| {delete_hover_attr.set(true)}
                            on:mouseout=move |_| {delete_hover_attr.set(false)}
                            class="px-3 py-1.5 border border-dhbw-gray-25 rounded hover:bg-dhbw-gray/10 text-dhbw-gray text-sm flex items-center gap-1"
                        >
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4"></path>
                            </svg>
                            Remove Attribute
                        </button>
                    </div>
                </div>

                <div class="overflow-auto border border-dhbw-gray-25 rounded-lg">
                    <table class="w-full bg-gray-100">
                        <tbody>
                            <tr>
                                <td class="w-10 h-10 bg-gray-200 border border-dhbw-gray-25"></td>
                                {move || {
                                    let attrs: Vec<_> = effective_context.get().attributes.iter().cloned().enumerate().collect();
                                    attrs.into_iter().map(|(col_idx, attr_name)| {
                                        view! {
                                            <td class="p-2 bg-gray-200 border border-dhbw-gray-25 min-w-32">
                                                <input type="text" class="w-full px-2 py-1 text-sm border border-dhbw-gray-25 rounded focus:outline-none focus:border-dhbw-red bg-white"
                                                    on:focus=move |_| {
                                                        focus_pos.update(|pos| {
                                                            *pos = (pos.0, col_idx)
                                                        });
                                                    }
                                                    on:change=move |ev| {
                                                        let input: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into();
                                                        change_attribute_name(col_idx, input.value());
                                                    }
                                                    prop:value=attr_name.clone()
                                                />
                                            </td>
                                        }
                                    }).collect::<Vec<_>>()
                                }}
                            </tr>
                            {move || {
                                let objs: Vec<_> = effective_context.get().objects.iter().cloned().enumerate().collect();
                                let attrs_count = effective_context.get().attributes.len();
                                objs.into_iter().map(|(row_idx, obj_name)| {
                                    view! {
                                        <tr>
                                            <td class="p-2 bg-gray-200 border border-dhbw-gray-25">
                                                <input type="text" class="w-32 px-2 py-1 text-sm border border-dhbw-gray-25 rounded focus:outline-none focus:border-dhbw-red bg-white"
                                                    on:focus=move |_| {
                                                        focus_pos.update(|pos| {
                                                            *pos = (row_idx, pos.1)
                                                        });
                                                    }
                                                    on:change=move |ev| {
                                                        let input: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into();
                                                        change_object_name(row_idx, input.value());
                                                    }
                                                    prop:value=obj_name.clone()
                                                />
                                            </td>
                                            {(0..attrs_count).into_iter().map(|col_idx| {
                                                let row_idx = row_idx;
                                                let base_class = Signal::derive(move || {
                                                    let base = "p-1 text-center cursor-pointer border border-dhbw-gray-25";
                                                    let bg = if delete_hover_obj.get() && row_idx == focus_pos.get().0 {
                                                        "bg-blue-100"
                                                    } else if delete_hover_attr.get() && col_idx == focus_pos.get().1 {
                                                        "bg-blue-100"
                                                    } else if (row_idx, col_idx) == focus_pos.get() {
                                                        "bg-blue-100"
                                                    } else {
                                                        "bg-white"
                                                    };
                                                    format!("{} {}", base, bg)
                                                });

                                                view! {
                                                    <td
                                                        class=base_class
                                                        on:click=move |_| {
                                                            focus_pos.update(|pos| *pos = (row_idx, col_idx));
                                                        }
                                                    >
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
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}

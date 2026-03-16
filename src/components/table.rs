use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

use bit_set::BitSet;
use odis::FormalContext;

use crate::components::context::index_to_column_name;

fn render_incidence_cell(
    row_idx: usize,
    col_idx: usize,
    context: RwSignal<FormalContext<String>>,
    on_toggle: impl Fn(usize, usize, bool) + Send + Clone + 'static,
) -> impl IntoView {
    let initial_checked = context.with_untracked(|ctx| ctx.incidence.contains(&(row_idx, col_idx)));

    let on_toggle = on_toggle.clone();
    view! {
        <td class="p-1 text-center border border-dhbw-gray-25 bg-white">
            <input
                type="checkbox"
                checked=initial_checked
                on:change=move |ev| {
                    let input: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into();
                    on_toggle(row_idx, col_idx, input.checked());
                }
                class="accent-dhbw-red cursor-pointer w-4 h-4"
            />
        </td>
    }
}

fn render_object_row(
    row_idx: usize,
    obj_name: String,
    context: RwSignal<FormalContext<String>>,
    on_delete: impl Fn(usize) + Send + Clone + 'static,
    on_rename: impl Fn(usize, String) + Send + Clone + 'static,
    on_toggle: impl Fn(usize, usize, bool) + Send + Clone + 'static,
) -> impl IntoView {
    let delete_handler = on_delete.clone();
    let on_rename_closure = on_rename.clone();
    let on_toggle_closure = on_toggle.clone();

    let delete_click = move |_| delete_handler(row_idx);
    let rename_blur = move |ev: web_sys::FocusEvent| {
        let target = ev.target().unwrap();
        let span: web_sys::HtmlElement = target.unchecked_into();
        let new_name = span.text_content().unwrap_or_default().trim().to_string();
        if !new_name.is_empty() {
            on_rename_closure(row_idx, new_name);
        }
    };

    view! {
        <tr>
            <td class="p-1 border border-dhbw-gray-25 bg-gray-100">
                <div class="flex items-center gap-1">
                    <button
                        on:click=delete_click
                        class="text-dhbw-gray hover:text-dhbw-red text-sm font-bold"
                        title="Delete Object"
                    >x</button>
                    <span
                        class="px-1 text-sm cursor-text hover:bg-gray-50"
                        contenteditable=true
                        on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                            if ev.key() == "Enter" {
                                ev.prevent_default();
                            }
                        }
                        on:blur=rename_blur
                    >{obj_name}</span>
                </div>
            </td>
            <For
                each=move || {
                    context.with(|ctx| ctx.attributes.iter().cloned().enumerate().collect::<Vec<_>>())
                }
                key=|(col_idx, name)| (*col_idx, name.clone())
                children=move |(col_idx, _name)| {
                    render_incidence_cell(row_idx, col_idx, context, on_toggle_closure.clone())
                }
            />
        </tr>
    }
}

fn render_attribute_header(
    col_idx: usize,
    attr_name: String,
    on_delete: impl Fn(usize) + Send + Clone + 'static,
    on_rename: impl Fn(usize, String) + Send + Clone + 'static,
) -> impl IntoView {
    let on_delete_closure = on_delete.clone();
    let on_rename_closure = on_rename.clone();

    let delete_click = move |_| on_delete_closure(col_idx);
    let rename_blur = move |ev: web_sys::FocusEvent| {
        let target = ev.target().unwrap();
        let span: web_sys::HtmlElement = target.unchecked_into();
        let new_name = span.text_content().unwrap_or_default().trim().to_string();
        if !new_name.is_empty() {
            on_rename_closure(col_idx, new_name);
        }
    };

    view! {
        <td class="p-1 border border-dhbw-gray-25 bg-gray-100">
            <div class="flex flex-col items-center">
                <button
                    on:click=delete_click
                    class="text-dhbw-gray hover:text-dhbw-red text-sm font-bold"
                    title="Delete Attribute"
                >x</button>
                <span
                    class="px-1 text-sm cursor-text hover:bg-gray-50"
                    contenteditable=true
                    on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                        if ev.key() == "Enter" {
                            ev.prevent_default();
                        }
                    }
                    on:blur=rename_blur
                >{attr_name}</span>
            </div>
        </td>
    }
}

fn render_add_attribute_button(on_click: impl Fn() + Send + Clone + 'static) -> impl IntoView {
    let on_click = on_click.clone();
    view! {
        <td class="p-1 border border-dhbw-gray-25 bg-gray-50">
            <button
                on:click=move |_| on_click()
                class="text-dhbw-gray hover:text-dhbw-red text-lg font-bold px-2"
                title="Add Attribute"
            >+</button>
        </td>
    }
}

fn render_add_object_button(on_click: impl Fn() + Send + Clone + 'static) -> impl IntoView {
    let on_click = on_click.clone();
    view! {
        <tr>
            <td class="p-1 border border-dhbw-gray-25 bg-gray-50 text-center">
                <button
                    on:click=move |_| on_click()
                    class="text-dhbw-gray hover:text-dhbw-red text-lg font-bold"
                    title="Add Object"
                >+</button>
            </td>
        </tr>
    }
}

#[component]
pub fn TableComp() -> impl IntoView {
    let context = use_context::<RwSignal<FormalContext<String>>>().expect("Context not provided");
    let on_context_change = use_context::<RwSignal<u64>>().unwrap_or_else(|| RwSignal::new(0));

    let add_object = {
        move || {
            context.update(|ctx| {
                let new_name = format!("{}", ctx.objects.len() + 1);
                ctx.add_object(new_name, &BitSet::new());
            });
            on_context_change.update(|v| *v += 1);
        }
    };

    let remove_object = {
        move |idx: usize| {
            let should_remove = context.with(|ctx| ctx.objects.len() > 1);
            if !should_remove {
                return;
            }
            context.update(|ctx| {
                ctx.remove_object(idx);
            });
            on_context_change.update(|v| *v += 1);
        }
    };

    let add_attribute = {
        move || {
            context.update(|ctx| {
                let new_name = index_to_column_name(ctx.attributes.len());
                ctx.add_attribute(new_name, &BitSet::new());
            });
            on_context_change.update(|v| *v += 1);
        }
    };

    let remove_attribute = {
        move |idx: usize| {
            let should_remove = context.with(|ctx| ctx.attributes.len() > 1);
            if !should_remove {
                return;
            }
            context.update(|ctx| {
                ctx.remove_attribute(idx);
            });
            on_context_change.update(|v| *v += 1);
        }
    };

    let change_object_name = {
        move |index: usize, name: String| {
            context.update_untracked(|ctx| {
                ctx.change_object_name(name, index);
            });
            on_context_change.update(|v| *v += 1);
        }
    };

    let change_attribute_name = {
        move |index: usize, name: String| {
            context.update_untracked(|ctx| {
                ctx.change_attribute_name(name, index);
            });
            on_context_change.update(|v| *v += 1);
        }
    };

    let toggle_cell = {
        move |obj: usize, attr: usize, value: bool| {
            context.update_untracked(|ctx| {
                if value {
                    ctx.incidence.insert((obj, attr));
                    ctx.atomic_object_derivations[obj].insert(attr);
                    ctx.atomic_attribute_derivations[attr].insert(obj);
                } else {
                    ctx.incidence.remove(&(obj, attr));
                    ctx.atomic_object_derivations[obj].remove(attr);
                    ctx.atomic_attribute_derivations[attr].remove(obj);
                }
            });
            on_context_change.update(|v| *v += 1);
        }
    };

    view! {
        <div class="h-full">
            <div class="overflow-auto">
                <table class="border-collapse">
                    <tbody>
                        <tr>
                            <td class="p-1"></td>
                            <For
                                each=move || {
                                    context.with(|ctx| {
                                        ctx.attributes
                                            .iter()
                                            .cloned()
                                            .enumerate()
                                            .collect::<Vec<_>>()
                                    })
                                }
                                key=|(idx, name)| (*idx, name.clone())
                                children=move |(col_idx, attr_name)| {
                                    render_attribute_header(
                                        col_idx,
                                        attr_name,
                                        remove_attribute,
                                        change_attribute_name,
                                    )
                                }
                            />
                            {render_add_attribute_button(add_attribute)}
                        </tr>
                        <For
                            each=move || {
                                context.with(|ctx| {
                                    ctx.objects
                                        .iter()
                                        .cloned()
                                        .enumerate()
                                        .collect::<Vec<_>>()
                                })
                            }
                            key=|(idx, name)| (*idx, name.clone())
                            children=move |(row_idx, obj_name)| {
                                render_object_row(
                                    row_idx,
                                    obj_name,
                                    context,
                                    remove_object,
                                    change_object_name,
                                    toggle_cell,
                                )
                            }
                        />
                        {render_add_object_button(add_object)}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

use leptos::{either::Either, prelude::*};

use bit_set::BitSet;
use leptos::wasm_bindgen::JsCast;
use odis::FormalContext;

use crate::components::context::create_default_context;
use crate::components::graph::{GraphComp, LayoutAlgorithm};

#[component]
pub fn FormalContextView(context: RwSignal<Option<FormalContext<String>>>) -> impl IntoView {
    let effective_context =
        Signal::derive(move || context.get().unwrap_or_else(|| create_default_context()));

    view! {
        <div class="bg-white min-h-full p-6">
            <h2 class="text-dhbw-gray font-semibold text-lg mb-4">Formal Context</h2>
            <div class="bg-gray-50 rounded-lg p-4 border border-dhbw-gray-25">
                {move || {
                    let ctx = effective_context.get();
                    view! {
                        <div class="grid grid-cols-2 gap-4 text-sm">
                            <div>
                                <span class="text-dhbw-gray-50 font-medium">Objects:</span>
                                <span class="text-dhbw-gray ml-2">{ctx.objects.len()}</span>
                            </div>
                            <div>
                                <span class="text-dhbw-gray-50 font-medium">Attributes:</span>
                                <span class="text-dhbw-gray ml-2">{ctx.attributes.len()}</span>
                            </div>
                            <div>
                                <span class="text-dhbw-gray-50 font-medium">Incidence:</span>
                                <span class="text-dhbw-gray ml-2">{ctx.incidence.len()}</span>
                            </div>
                            <div>
                                <span class="text-dhbw-gray-50 font-medium">Density:</span>
                                <span class="text-dhbw-gray ml-2">{if ctx.objects.len() > 0 && ctx.attributes.len() > 0 {
                                    format!("{:.2}%", (ctx.incidence.len() as f64 / (ctx.objects.len() * ctx.attributes.len()) as f64) * 100.0)
                                } else {
                                    "0%".to_string()
                                }}</span>
                            </div>
                        </div>
                    }
                }}
            </div>

            <div class="mt-6">
                <h3 class="text-dhbw-gray font-medium mb-3">Objects</h3>
                <ul class="border border-dhbw-gray-25 rounded divide-y divide-dhbw-gray-25">
                    {move || {
                        let objs: Vec<_> = effective_context.get().objects.iter().cloned().collect();
                        objs.into_iter().map(|name| {
                            view! {
                                <li class="px-4 py-2 text-sm text-dhbw-gray">{name}</li>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </ul>
            </div>

            <div class="mt-4">
                <h3 class="text-dhbw-gray font-medium mb-3">Attributes</h3>
                <ul class="border border-dhbw-gray-25 rounded divide-y divide-dhbw-gray-25">
                    {move || {
                        let attrs: Vec<_> = effective_context.get().attributes.iter().cloned().collect();
                        attrs.into_iter().map(|name| {
                            view! {
                                <li class="px-4 py-2 text-sm text-dhbw-gray">{name}</li>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </ul>
            </div>
        </div>
    }
}

#[component]
pub fn ConceptsView(context: RwSignal<Option<FormalContext<String>>>) -> impl IntoView {
    let effective_context =
        Signal::derive(move || context.get().unwrap_or_else(|| create_default_context()));

    let concepts = RwSignal::new(None);

    let calc_concepts = move || {
        let current_context = effective_context.get();
        let mut result: Vec<(BitSet, BitSet)> = current_context.fcbo_index_concepts().collect();
        current_context.index_sort_lectic_order(&mut result);
        concepts.set(Some(result));
    };

    view! {
        <div class="bg-white min-h-full p-6">
            <h2 class="text-dhbw-gray font-semibold text-lg mb-4">Concepts</h2>
            <button on:click=move |_| calc_concepts() class="px-4 py-2 bg-dhbw-red text-white rounded hover:bg-red-700 text-sm mb-4">
                Compute Concepts
            </button>
            {move || {
                if let Some(n) = concepts.get() {
                    let concepts_clone: Vec<(usize, (BitSet, BitSet))> = concepts.get().unwrap().into_iter().enumerate().collect();
                    Either::Left(view! {
                        <p class="text-sm text-dhbw-gray-50 mb-2">{format!("The number of concepts is: {}", n.len())}</p>
                        <ul class="h-96 overflow-y-auto border border-dhbw-gray-25 rounded bg-white">
                            {concepts_clone.into_iter().map(|(idx, concept)| {
                                view! {
                                    <li class="p-2 border-b border-dhbw-gray-25 last:border-0 whitespace-pre text-sm font-mono">
                                        {
                                            let mut obj_string = String::new();
                                            obj_string.push('{');

                                            for n in &concept.0 {
                                                obj_string.push_str(
                                                    &(" ".to_string() + &effective_context.get().objects[n] + " ,")
                                                );
                                            }

                                            if concept.0.len() > 0 {
                                                obj_string.pop();
                                            } else {
                                                obj_string.push(' ');
                                            }
                                            obj_string.push('}');

                                            let mut white_spaces = String::from("   ");
                                            if idx >= 9 {
                                                white_spaces.truncate(1);
                                            }

                                            format!("{}:{}{},", idx + 1, white_spaces, obj_string)
                                        }
                                        <br/>
                                        {
                                            let mut attr_string = String::new();
                                            attr_string.push('{');

                                            for n in &concept.1 {
                                                attr_string.push_str(
                                                    &(" ".to_string() + &effective_context.get().attributes[n] + " ,")
                                                );
                                            }
                                            if concept.1.len() > 0 {
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
                            }).collect::<Vec<_>>()}
                        </ul>
                    })
                } else {
                    Either::Right(view! {<p class="text-dhbw-gray-50 text-sm">Click "Compute Concepts" to view all concepts</p>})
                }
            }}
        </div>
    }
}

#[component]
pub fn CanonicalBasisView(context: RwSignal<Option<FormalContext<String>>>) -> impl IntoView {
    let effective_context =
        Signal::derive(move || context.get().unwrap_or_else(|| create_default_context()));

    let basis = RwSignal::new(None);

    let calc_basis = move || {
        let current_context = effective_context.get();
        let result = current_context.index_canonical_basis();
        basis.set(Some(result));
    };

    view! {
        <div class="bg-white min-h-full p-6">
            <h2 class="text-dhbw-gray font-semibold text-lg mb-4">Canonical Basis</h2>
            <button on:click=move |_| calc_basis() class="px-4 py-2 bg-dhbw-red text-white rounded hover:bg-red-700 text-sm mb-4">
                Compute Canonical Base
            </button>
            {move || {
                if let Some(n) = basis.get() {
                    let basis_clone: Vec<(usize, (BitSet, BitSet))> = basis.get().unwrap().into_iter().enumerate().collect();
                    Either::Left(view! {
                        <p class="text-sm text-dhbw-gray-50 mb-2">{format!("The number of implications is: {}", n.len())}</p>
                        <ul class="h-96 overflow-y-auto border border-dhbw-gray-25 rounded bg-white">
                            {basis_clone.into_iter().map(|(idx, basis)| {
                                view! {
                                    <li class="p-2 border-b border-dhbw-gray-25 last:border-0 whitespace-pre text-sm font-mono">
                                        {
                                            let mut premise = String::new();
                                            premise.push('{');

                                            for n in &basis.0 {
                                                premise.push_str(
                                                    &(" ".to_string() + &effective_context.get().attributes[n] + " ,")
                                                );
                                            }

                                            if basis.0.len() > 0 {
                                                premise.pop();
                                            } else {
                                                premise.push(' ');
                                            }
                                            premise.push('}');

                                            let mut white_spaces = String::from("   ");
                                            if idx >= 9 {
                                                white_spaces.truncate(1);
                                            }

                                            format!("{}:{}{}", idx + 1, white_spaces, premise)
                                        }
                                        <br/>
                                        {
                                            let mut conclusion = String::new();
                                            conclusion.push('{');

                                            for n in &basis.1 {
                                                conclusion.push_str(
                                                    &(" ".to_string() + &effective_context.get().attributes[n] + " ,")
                                                );
                                            }
                                            if basis.1.len() > 0 {
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
                            }).collect::<Vec<_>>()}
                        </ul>
                    })
                } else {
                    Either::Right(view! {<p class="text-dhbw-gray-50 text-sm">Click "Compute Canonical Base" to view all implications</p>})
                }
            }}
        </div>
    }
}

#[component]
pub fn ConceptLatticeView(context: RwSignal<Option<FormalContext<String>>>) -> impl IntoView {
    let effective_context =
        Signal::derive(move || context.get().unwrap_or_else(|| create_default_context()));

    let concepts = RwSignal::new(None);
    let concept_lattice = RwSignal::new(false);
    let layout_algorithm = RwSignal::new(LayoutAlgorithm::Dimdraw);

    let calc_concepts = move || {
        let current_context = effective_context.get();
        let mut result: Vec<(BitSet, BitSet)> = current_context.fcbo_index_concepts().collect();
        current_context.index_sort_lectic_order(&mut result);
        concepts.set(Some(result));
    };

    view! {
        <div class="bg-white min-h-full p-6">
            <h2 class="text-dhbw-gray font-semibold text-lg mb-4">Concept Lattice</h2>
            <div class="flex items-center gap-4 mb-4">
                <label class="text-dhbw-gray font-medium">Layout Algorithm:</label>
                <select
                    on:change=move |ev| {
                        let select: web_sys::HtmlSelectElement = ev.target().unwrap().unchecked_into();
                        let value = select.value();
                        let algorithm = match value.as_str() {
                            "Sugiyama" => LayoutAlgorithm::Sugiyama,
                            _ => LayoutAlgorithm::Dimdraw,
                        };
                        layout_algorithm.set(algorithm);
                    }
                    class="px-3 py-2 border border-dhbw-gray-25 rounded text-dhbw-gray text-sm focus:outline-none focus:border-dhbw-red"
                >
                    <option value="Dimdraw" selected=move || layout_algorithm.get() == LayoutAlgorithm::Dimdraw>Dimdraw</option>
                    <option value="Sugiyama" selected=move || layout_algorithm.get() == LayoutAlgorithm::Sugiyama>Sugiyama</option>
                </select>
            </div>
            <button on:click=move |_| {
                calc_concepts();
                concept_lattice.set(true);
            } class="px-4 py-2 bg-dhbw-red text-white rounded hover:bg-red-700 text-sm mb-4">
                Draw Concept Lattice
            </button>
            <div>
                {move || {
                    if concept_lattice.get() {
                        if let Some(concepts_data) = concepts.get() {
                            view! {
                                <div class="mt-4">
                                    <GraphComp
                                        concepts=concepts_data
                                        context=effective_context.get()
                                        algorithm=layout_algorithm.get()
                                    />
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <p class="text-dhbw-gray-50 text-sm">No concepts computed yet</p>
                            }.into_any()
                        }
                    } else {
                        view! {
                            <p class="text-dhbw-gray-50 text-sm">Click "Draw Concept Lattice" to view the graph</p>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
pub fn ExplorationViewWrapper(context: RwSignal<Option<FormalContext<String>>>) -> impl IntoView {
    let effective_context =
        Signal::derive(move || context.get().unwrap_or_else(|| create_default_context()));

    view! {
        <div class="bg-white min-h-full p-6">
            <h2 class="text-dhbw-gray font-semibold text-lg mb-4">Attribute Exploration</h2>
            <p class="text-dhbw-gray-50 text-sm mb-4">Attribute exploration allows you to interactively discover implications by validating proposed implications and providing counterexamples.</p>

            <div class="bg-gray-50 rounded-lg p-6 border border-dhbw-gray-25">
                <div class="text-center">
                    <svg class="w-16 h-16 text-dhbw-gray-25 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
                    </svg>
                    <h3 class="text-dhbw-gray font-medium mb-2">Ready to Explore</h3>
                    <p class="text-dhbw-gray-50 text-sm mb-4">Click the button below to start attribute exploration with the current formal context.</p>
                    <button class="px-4 py-2 bg-dhbw-red text-white rounded hover:bg-red-700 text-sm">
                        Start Exploration
                    </button>
                </div>
            </div>

            <div class="mt-6 grid grid-cols-3 gap-4">
                <div class="bg-gray-50 rounded-lg p-4 border border-dhbw-gray-25 text-center">
                    <div class="text-2xl font-bold text-dhbw-red">{effective_context.get().attributes.len()}</div>
                    <div class="text-sm text-dhbw-gray-50">Attributes</div>
                </div>
                <div class="bg-gray-50 rounded-lg p-4 border border-dhbw-gray-25 text-center">
                    <div class="text-2xl font-bold text-dhbw-red">{effective_context.get().objects.len()}</div>
                    <div class="text-sm text-dhbw-gray-50">Objects</div>
                </div>
                <div class="bg-gray-50 rounded-lg p-4 border border-dhbw-gray-25 text-center">
                    <div class="text-2xl font-bold text-dhbw-red">?</div>
                    <div class="text-sm text-dhbw-gray-50">Implications</div>
                </div>
            </div>
        </div>
    }
}

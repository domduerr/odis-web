use leptos::{either::Either, prelude::*};

use bit_set::BitSet;
use leptos::wasm_bindgen::JsCast;
use odis::FormalContext;

use crate::components::exploration::ExplorationComp;
use crate::components::graph::{GraphComp, LayoutAlgorithm};
use crate::core::formatters::format_implication;

fn format_object_set_table(indices: &BitSet, names: &[String]) -> String {
    let items: Vec<String> = indices
        .iter()
        .filter(|&n| n < names.len())
        .map(|n| names[n].clone())
        .collect();

    if items.is_empty() {
        String::from("{}")
    } else {
        format!("{{{}}}", items.join(", "))
    }
}

fn format_attribute_set_table(indices: &BitSet, names: &[String]) -> String {
    let items: Vec<String> = indices
        .iter()
        .filter(|&n| n < names.len())
        .map(|n| names[n].clone())
        .collect();

    if items.is_empty() {
        String::from("{}")
    } else {
        format!("{{{}}}", items.join(", "))
    }
}

#[component]
pub fn FormalContextView() -> impl IntoView {
    let context = use_context::<RwSignal<FormalContext<String>>>().expect("Context not provided");

    let effective_context = Signal::derive(move || context.get());

    view! {
        <div class="bg-gray-50 min-h-full p-6">
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
pub fn ConceptsView() -> impl IntoView {
    let context = use_context::<RwSignal<FormalContext<String>>>().expect("Context not provided");
    let effective_context = Signal::derive(move || context.get());

    let concepts_data: RwSignal<Option<Vec<(BitSet, BitSet)>>> = RwSignal::new(None);
    let is_loaded = RwSignal::new(false);
    let last_context_hash: RwSignal<u64> = RwSignal::new(0);

    let load_concepts = {
        let concepts_data = concepts_data.clone();
        let effective_context = effective_context.clone();
        let is_loaded = is_loaded.clone();
        move || {
            if !is_loaded.get() {
                let ctx = effective_context.get();
                let mut result: Vec<(BitSet, BitSet)> = ctx.fcbo_index_concepts().collect();
                ctx.index_sort_lectic_order(&mut result);
                concepts_data.set(Some(result));
                is_loaded.set(true);
            }
        }
    };

    let context_version = use_context::<RwSignal<u64>>().unwrap_or_else(|| RwSignal::new(0));

    let is_loaded_for_effect = is_loaded.clone();
    let last_context_hash_for_effect = last_context_hash.clone();
    let context_version_for_effect = context_version.clone();
    let load_concepts_for_effect = load_concepts.clone();

    Effect::new(move |_| {
        let cv = context_version_for_effect.get();
        let last = last_context_hash_for_effect.get_untracked();
        if cv != last {
            is_loaded_for_effect.set(false);
            last_context_hash_for_effect.set(cv);
        }
    });

    Effect::new(move |_| {
        load_concepts_for_effect();
    });

    view! {
        <div class="bg-gray-50 min-h-full p-6">
            {move || {
                if is_loaded.get() {
                    if let Some(concepts) = concepts_data.get() {
                        let ctx = effective_context.get();
                        let objects = ctx.objects.clone();
                        let attributes = ctx.attributes.clone();

                        Either::Left(view! {
                            <div class="bg-white rounded-lg border border-dhbw-gray-25 overflow-hidden">
                                <div class="bg-dhbw-gray-25 px-4 py-2 border-b border-dhbw-gray-25">
                                    <span class="text-dhbw-gray font-medium">Concepts</span>
                                    <span class="text-dhbw-gray-50 text-sm ml-2">{format!("({} concepts)", concepts.len())}</span>
                                </div>
                                <div class="overflow-auto max-h-[70vh]">
                                    <table class="w-full text-sm">
                                        <thead class="bg-gray-50 sticky top-0">
                                            <tr>
                                                <th class="px-4 py-2 text-left text-dhbw-gray-50 font-medium w-16">#</th>
                                                <th class="px-4 py-2 text-left text-dhbw-gray-50 font-medium">Extent{" "}(Objects)</th>
                                                <th class="px-4 py-2 text-left text-dhbw-gray-50 font-medium">Intent{" "}(Attributes)</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {concepts.into_iter().enumerate().map(|(idx, concept)| {
                                                let obj_set = format_object_set_table(&concept.0, &objects);
                                                let attr_set = format_attribute_set_table(&concept.1, &attributes);
                                                view! {
                                                    <tr class="border-t border-dhbw-gray-25 hover:bg-gray-50">
                                                        <td class="px-4 py-2 text-dhbw-gray-50">{idx + 1}</td>
                                                        <td class="px-4 py-2 text-dhbw-gray">{obj_set}</td>
                                                        <td class="px-4 py-2 text-dhbw-gray">{attr_set}</td>
                                                    </tr>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        }.into_any())
                    } else {
                        Either::Right(view! {
                            <div class="bg-white rounded-lg border border-dhbw-gray-25 p-8 text-center">
                                <p class="text-dhbw-gray font-medium mb-2">Loading concepts...</p>
                            </div>
                        }.into_any())
                    }
                } else {
                    Either::Right(view! {
                        <div class="bg-white rounded-lg border border-dhbw-gray-25 p-8 text-center">
                            <p class="text-dhbw-gray font-medium mb-2">Loading concepts...</p>
                        </div>
                    }.into_any())
                }
            }}
        </div>
    }
}

#[component]
pub fn CanonicalBasisView() -> impl IntoView {
    let context = use_context::<RwSignal<FormalContext<String>>>().expect("Context not provided");

    let effective_context = Signal::derive(move || context.get());

    let basis = RwSignal::new(None);

    let calc_basis = move || {
        let current_context = effective_context.get();
        let result = current_context.index_canonical_basis();
        basis.set(Some(result));
    };

    view! {
        <div class="bg-gray-50 min-h-full p-6">
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
                                let attributes = effective_context.get().attributes;
                                let (premise_line, conclusion_line) = format_implication(&basis.0, &basis.1, &attributes, idx);
                                view! {
                                    <li class="p-2 border-b border-dhbw-gray-25 last:border-0 whitespace-pre text-sm font-mono">
                                        {premise_line}
                                        <br/>
                                        {conclusion_line}
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
pub fn ConceptLatticeView() -> impl IntoView {
    let context = use_context::<RwSignal<FormalContext<String>>>().expect("Context not provided");

    let effective_context = Signal::derive(move || context.get());

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
        <div class="bg-gray-50 min-h-full p-6">
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
pub fn ExplorationViewWrapper() -> impl IntoView {
    let context = use_context::<RwSignal<FormalContext<String>>>().expect("Context not provided");

    let effective_context = Signal::derive(move || context.get());

    let row_key = RwSignal::new(0);
    let object_names = RwSignal::new(Vec::new());

    view! {
        <div class="bg-gray-50 min-h-full p-6">
            <h2 class="text-dhbw-gray font-semibold text-lg mb-4">Attribute Exploration</h2>
            <p class="text-dhbw-gray-50 text-sm mb-4">Attribute exploration allows you to interactively discover implications by validating proposed implications and providing counterexamples.</p>

            <ExplorationComp
                row_key=row_key
                object_names=object_names
            />

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

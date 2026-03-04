use leptos::{either::Either, prelude::*};

use bit_set::BitSet;
use odis::FormalContext;

use crate::components::exploration::ExplorationComp;
use crate::components::graph::{GraphComp, LayoutAlgorithm};
use crate::core::formatters::format_attribute_set;

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
                                <table class="w-full text-sm table-fixed">
                                    <thead class="bg-gray-50 sticky top-0">
                                        <tr>
                                            <th class="px-4 py-2 text-left text-dhbw-gray-50 font-medium w-16">#</th>
                                            <th class="px-4 py-2 text-left text-dhbw-gray-50 font-medium w-1/2">Extent{" "}(Objects)</th>
                                            <th class="px-4 py-2 text-left text-dhbw-gray-50 font-medium w-1/2">Intent{" "}(Attributes)</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {concepts.into_iter().enumerate().map(|(idx, concept)| {
                                            let obj_set = format_object_set_table(&concept.0, &objects);
                                            let attr_set = format_attribute_set_table(&concept.1, &attributes);
                                            view! {
                                            <tr class="border-t border-dhbw-gray-25 hover:bg-gray-50">
                                                <td class="px-4 py-2 text-dhbw-gray-50 w-16 align-top">{idx + 1}</td>
                                                <td class="px-4 py-2 text-dhbw-gray w-1/2 align-top whitespace-normal break-all">{obj_set}</td>
                                                <td class="px-4 py-2 text-dhbw-gray w-1/2 align-top whitespace-normal break-all">{attr_set}</td>
                                            </tr>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </tbody>
                                </table>
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

    calc_basis();

    view! {
        <div class="bg-gray-50 min-h-full p-6">
            {move || {
                if let Some(n) = basis.get() {
                    let len = n.len();
                    let basis_clone: Vec<(usize, (BitSet, BitSet))> = n.into_iter().enumerate().collect();
                    let ctx = effective_context.get();
                    let attributes = ctx.attributes.clone();

                    Either::Left(view! {
                        <div class="bg-white rounded-lg border border-dhbw-gray-25 overflow-hidden">
                            <div class="bg-dhbw-gray-25 px-4 py-2 border-b border-dhbw-gray-25">
                                <span class="text-dhbw-gray font-medium">Canonical Basis</span>
                                <span class="text-dhbw-gray-50 text-sm ml-2">{format!("({} implications)", len)}</span>
                            </div>
                            <table class="w-full text-sm table-fixed">
                                <thead class="bg-gray-50 sticky top-0">
                                    <tr>
                                        <th class="px-4 py-2 text-left text-dhbw-gray-50 font-medium w-16">#</th>
                                        <th class="px-4 py-2 text-left text-dhbw-gray-50 font-medium w-1/2">Premise</th>
                                        <th class="px-4 py-2 text-center text-dhbw-gray-50 font-medium w-20"></th>
                                        <th class="px-4 py-2 text-left text-dhbw-gray-50 font-medium w-1/2">Conclusion</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {basis_clone.into_iter().map(|(idx, implication)| {
                                        let premise_set = format_attribute_set(&implication.0, &attributes);
                                        let conclusion_set = format_attribute_set(&implication.1, &attributes);
                                        view! {
                                            <tr class="border-t border-dhbw-gray-25 hover:bg-gray-50">
                                                <td class="px-4 py-2 text-dhbw-gray-50 w-16 align-top">{idx + 1}</td>
                                                <td class="px-4 py-2 text-dhbw-gray font-mono w-1/2 align-top whitespace-normal break-all">{premise_set}</td>
                                                <td class="px-4 py-2 text-dhbw-red font-mono text-center w-20 align-top">{"->"}</td>
                                                <td class="px-4 py-2 text-dhbw-gray font-mono w-1/2 align-top whitespace-normal break-all">{conclusion_set}</td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
                                </tbody>
                            </table>
                        </div>
                    }.into_any())
                } else {
                    Either::Right(view! {
                        <div class="bg-white rounded-lg border border-dhbw-gray-25 p-8 text-center">
                            <p class="text-dhbw-gray font-medium mb-2">Loading canonical basis...</p>
                        </div>
                    }.into_any())
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
    let layout_algorithm = RwSignal::new(LayoutAlgorithm::Sugiyama);

    let calc_concepts = move || {
        let current_context = effective_context.get();
        let mut result: Vec<(BitSet, BitSet)> = current_context.fcbo_index_concepts().collect();
        current_context.index_sort_lectic_order(&mut result);
        concepts.set(Some(result));
    };

    calc_concepts();

    view! {
        <div class="bg-gray-50 min-h-full p-6">
            {move || {
                if let Some(concepts_data) = concepts.get() {
                    view! {
                        <GraphComp
                            concepts=concepts_data
                            context=effective_context.get()
                            layout_algorithm=layout_algorithm
                        />
                    }.into_any()
                } else {
                    view! {
                        <p class="text-dhbw-gray-50 text-sm">No concepts computed yet</p>
                    }.into_any()
                }
            }}
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

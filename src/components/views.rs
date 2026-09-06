use leptos::{either::Either, prelude::*};

use bit_set::BitSet;
use odis::FormalContext;

use crate::components::exploration::ExplorationComp;
use crate::components::graph::{GraphComp, LayoutAlgorithm};
use crate::components::ui::{Arrow, Panel, ARROW_CELL, SET_TEXT, TH};
use crate::core::formatters::{count, format_set};

#[component]
pub fn ConceptsView() -> impl IntoView {
    let context = use_context::<RwSignal<FormalContext<String>>>().expect("Context not provided");

    let concepts_data: RwSignal<Option<Vec<(BitSet, BitSet)>>> = RwSignal::new(None);
    let is_loaded = RwSignal::new(false);
    let last_context_hash: RwSignal<u64> = RwSignal::new(0);

    let load_concepts = {
        move || {
            if !is_loaded.get() {
                context.with(|ctx| {
                    let mut result: Vec<(BitSet, BitSet)> = ctx.index_fcbo_concepts().collect();
                    ctx.index_sort_lectic_order(&mut result);
                    concepts_data.set(Some(result));
                });
                is_loaded.set(true);
            }
        }
    };

    let context_version = use_context::<RwSignal<u64>>().unwrap_or_else(|| RwSignal::new(0));

    let is_loaded_for_effect = is_loaded;
    let last_context_hash_for_effect = last_context_hash;
    let context_version_for_effect = context_version;
    let load_concepts_for_effect = load_concepts;

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
        <div class="h-full">
            {move || {
                if is_loaded.get() {
                    if let Some(concepts) = concepts_data.get() {
                        let (objects, attributes) = context.with(|ctx| (ctx.objects.clone(), ctx.attributes.clone()));

                        let meta = count(concepts.len(), "concept");

                        Either::Left(view! {
                            <Panel title=|| "Concepts" meta=meta>
                                <table class="w-full table-fixed text-sm">
                                    <thead class="sticky top-0 bg-white">
                                        <tr class="border-b border-dhbw-gray-25">
                                            <th class=format!("{TH} w-12")>#</th>
                                            <th class=format!("{TH} w-1/2")>Extent{" "}(Objects)</th>
                                            <th class=format!("{TH} w-1/2")>Intent{" "}(Attributes)</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {concepts.into_iter().enumerate().map(|(idx, concept)| {
                                            let obj_set = format_set(&concept.0, &objects);
                                            let attr_set = format_set(&concept.1, &attributes);
                                            view! {
                                            <tr class="border-t border-dhbw-gray-25 hover:bg-gray-50">
                                                <td class="w-12 px-4 py-2 align-top text-dhbw-gray-50">{idx + 1}</td>
                                                <td class=format!("{SET_TEXT} w-1/2 px-4 py-2 align-top")>{obj_set}</td>
                                                <td class=format!("{SET_TEXT} w-1/2 px-4 py-2 align-top")>{attr_set}</td>
                                            </tr>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </tbody>
                                </table>
                            </Panel>
                        }.into_any())
                    } else {
                        Either::Right(view! {
                            <div class="rounded-lg border border-dhbw-gray-25 bg-white p-8 text-center">
                                <p class="text-sm text-dhbw-gray-50">"Computing concepts…"</p>
                            </div>
                        }.into_any())
                    }
                } else {
                    Either::Right(view! {
                        <div class="rounded-lg border border-dhbw-gray-25 bg-white p-8 text-center">
                            <p class="text-sm text-dhbw-gray-50">"Computing concepts…"</p>
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

    let basis = RwSignal::new(None);

    let calc_basis = {
        move || {
            context.with_untracked(|ctx| {
                let result = ctx.index_canonical_basis();
                basis.set(Some(result));
            });
        }
    };

    calc_basis();

    view! {
        <div class="h-full">
            {move || {
                if let Some(n) = basis.get() {
                    let len = n.len();
                    let basis_clone: Vec<(usize, (BitSet, BitSet))> = n.into_iter().enumerate().collect();
                    let attributes = context.with(|ctx| ctx.attributes.clone());

                    let meta = count(len, "implication");

                    Either::Left(view! {
                        <Panel title=|| "Canonical Basis" meta=meta>
                            <table class="w-full table-fixed text-sm">
                                <thead class="sticky top-0 bg-white">
                                    <tr class="border-b border-dhbw-gray-25">
                                        <th class=format!("{TH} w-12")>#</th>
                                        <th class=format!("{TH} w-1/2")>Premise</th>
                                        <th class=format!("{TH} w-10")></th>
                                        <th class=format!("{TH} w-1/2")>Conclusion</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {basis_clone.into_iter().map(|(idx, implication)| {
                                        let premise_set = format_set(&implication.0, &attributes);
                                        let conclusion_set = format_set(&implication.1, &attributes);
                                        view! {
                                            <tr class="border-t border-dhbw-gray-25 hover:bg-gray-50">
                                                <td class="w-12 px-4 py-2 align-top text-dhbw-gray-50">{idx + 1}</td>
                                                <td class=format!("{SET_TEXT} w-1/2 px-4 py-2 align-top")>{premise_set}</td>
                                                <td class=ARROW_CELL><Arrow/></td>
                                                <td class=format!("{SET_TEXT} w-1/2 px-4 py-2 align-top")>{conclusion_set}</td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
                                </tbody>
                            </table>
                        </Panel>
                    }.into_any())
                } else {
                    Either::Right(view! {
                        <div class="rounded-lg border border-dhbw-gray-25 bg-white p-8 text-center">
                            <p class="text-sm text-dhbw-gray-50">"Computing canonical basis…"</p>
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

    let concepts = RwSignal::new(None);

    let calc_concepts = {
        move || {
            context.with_untracked(|ctx| {
                let mut result: Vec<(BitSet, BitSet)> = ctx.index_fcbo_concepts().collect();
                ctx.index_sort_lectic_order(&mut result);
                concepts.set(Some(result));
            });
        }
    };

    calc_concepts();

    let num_concepts = concepts.get_untracked().map(|c| c.len()).unwrap_or(0);
    let layout_algorithm = RwSignal::new(if num_concepts > 100 {
        LayoutAlgorithm::Sugiyama
    } else {
        LayoutAlgorithm::DimDraw
    });

    view! {
        <div class="h-full">
            {move || {
                if let Some(concepts_data) = concepts.get() {
                    let ctx = context.get();
                    view! {
                        <GraphComp
                            concepts=concepts_data
                            context=ctx
                            layout_algorithm=layout_algorithm
                        />
                    }.into_any()
                } else {
                    view! {
                        <p class="text-sm text-dhbw-gray-50">No concepts computed yet</p>
                    }.into_any()
                }
            }}
        </div>
    }
}

#[component]
pub fn ExplorationViewWrapper() -> impl IntoView {
    view! {
        <ExplorationComp />
    }
}

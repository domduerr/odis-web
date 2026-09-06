use bit_set::BitSet;
use leptos::{either::Either, prelude::*};
use leptos::wasm_bindgen::JsCast;
use odis::{
    algorithms::{dimdraw::DimDraw, dimflux::DimFlux, sugiyama::Sugiyama, titanic::Titanic, SearchBudget},
    traits::{ConceptDrawingAlgorithm, DrawingAlgorithm, IcebergConceptEnumerator},
    Drawing, FormalContext, IcebergLattice,
};

use crate::components::graph::{Dimensions, LayoutAlgorithm, Node};
use crate::components::svg::{edge::EdgeComp, node::NodeComp};
use crate::components::svg_download::SvgDownloadComp;
use crate::components::ui::{Panel, CONTROL_LABEL, INPUT};
use crate::core::formatters::count;

// ── Label mode for the text shown below each node ─────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LabelMode {
    /// Show absolute support count below the node.
    SupportAbsolute,
    /// Show relative support as a percentage below the node.
    SupportRelative,
    /// Show object names (extent) below the node.
    Objects,
}

// ── Internal node descriptor (reduced labels) ─────────────────────────────────

#[derive(Clone, Debug)]
struct IcebergNode {
    id: usize,
    attr_label: Option<String>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn pct_to_abs(pct: f64, total: u32) -> u32 {
    (pct / 100.0 * total as f64).ceil() as u32
}

fn abs_to_pct_str(abs: u32, total: u32) -> String {
    if total == 0 {
        return "100".to_string();
    }
    format!("{:.0}", abs as f64 * 100.0 / total as f64)
}

fn obj_label_str(extent: &BitSet, objects: &[String]) -> String {
    let items: Vec<&str> = extent
        .iter()
        .filter(|&i| i < objects.len())
        .map(|i| objects[i].as_str())
        .collect();
    items.join(", ")
}

const ICEBERG_BUDGET: SearchBudget = SearchBudget::Milliseconds(5000);

fn compute_layout(
    ice: &IcebergLattice,
    algo: LayoutAlgorithm,
    attribute_count: usize,
) -> Vec<(f64, f64)> {
    let drawing = match algo {
        // Iceberg posets run larger than the concept lattices the default
        // budget is set for, so this view pays for a longer search.
        LayoutAlgorithm::DimDraw => DimDraw {
            budget: ICEBERG_BUDGET,
        }
        .draw_poset(&ice.poset),
        LayoutAlgorithm::DimFlux => DimFlux {
            budget: ICEBERG_BUDGET,
            ..DimFlux::default()
        }
        .draw_iceberg(ice, attribute_count),
        LayoutAlgorithm::Sugiyama => Sugiyama { vertex_spacing: 1 }.draw_poset(&ice.poset),
    };
    drawing.map(|d| d.coordinates).unwrap_or_default()
}

/// Reduced per-node labels: each attribute/object placed at the concept it generates.
/// Attributes/objects whose concept is below the support threshold are silently skipped.
fn build_iceberg_graph_nodes(
    ice: &IcebergLattice,
    context: &FormalContext<String>,
) -> Vec<IcebergNode> {
    let n = ice.poset.nodes.len();
    let mut attr_labels: Vec<Vec<String>> = vec![Vec::new(); n];
    let mut obj_labels: Vec<Vec<String>> = vec![Vec::new(); n];

    for attr_idx in 0..context.attributes.len() {
        let mut m = BitSet::new();
        m.insert(attr_idx);
        let extent = context.index_extent(&m);
        if let Some(node_idx) = ice.poset.nodes.iter().position(|(ext, _)| *ext == extent) {
            attr_labels[node_idx].push(context.attributes[attr_idx].clone());
        }
    }

    for obj_idx in 0..context.objects.len() {
        let mut g = BitSet::new();
        g.insert(obj_idx);
        let hull = context.index_object_hull(&g);
        if let Some(node_idx) = ice.poset.nodes.iter().position(|(ext, _)| *ext == hull) {
            obj_labels[node_idx].push(context.objects[obj_idx].clone());
        }
    }

    (0..n)
        .map(|id| IcebergNode {
            id,
            attr_label: if attr_labels[id].is_empty() { None } else { Some(attr_labels[id].join(", ")) },
        })
        .collect()
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn IcebergView() -> impl IntoView {
    let context = use_context::<RwSignal<FormalContext<String>>>().expect("Context not provided");
    let context_version = use_context::<RwSignal<u64>>().unwrap_or_else(|| RwSignal::new(0));
    let last_ctx_version: RwSignal<u64> = RwSignal::new(0);

    // ── Threshold signal (drives iceberg computation directly) ────────────────
    let initial_total = context.with_untracked(|ctx| ctx.objects.len() as u32);
    let threshold_abs: RwSignal<u32> = RwSignal::new(initial_total);

    // ── Display options ───────────────────────────────────────────────────────
    let label_mode: RwSignal<LabelMode> = RwSignal::new(LabelMode::SupportAbsolute);
    let layout_algorithm: RwSignal<LayoutAlgorithm> = RwSignal::new(LayoutAlgorithm::DimDraw);

    // ── Canvas dimensions (same defaults as GraphComp) ────────────────────────
    let dimensions = RwSignal::new(Dimensions {
        width: 600.0,
        height: 600.0,
        margin: 70.0,
        radius: 8.0,
        font_size: 16,
    });
    let width_input_ref = NodeRef::<leptos::html::Input>::new();
    let height_input_ref = NodeRef::<leptos::html::Input>::new();
    let svg_ref: NodeRef<leptos::svg::Svg> = NodeRef::new();

    // ── Resize state ──────────────────────────────────────────────────────────
    let is_resizing = RwSignal::new(false);
    let resize_type = RwSignal::new(0i32);
    let resize_start_x = RwSignal::new(0.0f64);
    let resize_start_y = RwSignal::new(0.0f64);
    let resize_start_width = RwSignal::new(0.0f64);
    let resize_start_height = RwSignal::new(0.0f64);

    // ── SVG bounding rect offset for drag coordinate calculation ──────────────
    let offset: RwSignal<Option<(f64, f64)>> = RwSignal::new(None);
    Effect::new(move || {
        if let Some(element) = svg_ref.get() {
            let rect = element.get_bounding_client_rect();
            let scroll_x = window().scroll_x().unwrap_or(0.0);
            let scroll_y = window().scroll_y().unwrap_or(0.0);
            offset.set(Some((rect.x() + scroll_x, rect.y() + scroll_y)));
        }
    });

    // Reset threshold when the context changes.
    Effect::new(move |_| {
        let cv = context_version.get();
        let last = last_ctx_version.get_untracked();
        if cv != last {
            last_ctx_version.set(cv);
            let total = context.with_untracked(|ctx| ctx.objects.len() as u32);
            threshold_abs.set(total);
        }
    });

    // ── Iceberg lattice — recomputed on every threshold / context change ───────
    let iceberg: RwSignal<IcebergLattice> = RwSignal::new(
        context.with_untracked(|ctx| Titanic.enumerate(ctx, initial_total.max(1))),
    );

    Effect::new(move |_| {
        let threshold = threshold_abs.get();
        let _cv = context_version.get(); // also re-run on context change
        let ice = context.with(|ctx| Titanic.enumerate(ctx, threshold));
        iceberg.set(ice);
    });

    // ── Reduced graph nodes — rebuilt when iceberg changes ────────────────────
    let iceberg_graph_nodes: RwSignal<Vec<IcebergNode>> = RwSignal::new({
        iceberg.with_untracked(|ice| context.with_untracked(|ctx| build_iceberg_graph_nodes(ice, ctx)))
    });

    Effect::new(move |_| {
        let gn = iceberg.with(|ice| context.with(|ctx| build_iceberg_graph_nodes(ice, ctx)));
        iceberg_graph_nodes.set(gn);
    });

    // ── Raw layout coordinates — rebuilt when iceberg or algorithm changes ─────
    let raw_coords: RwSignal<Vec<(f64, f64)>> = RwSignal::new(Vec::new());

    Effect::new(move |_| {
        let algo = layout_algorithm.get();
        let attribute_count = context.with(|ctx| ctx.attributes.len());
        let coords = iceberg.with(|ice| compute_layout(ice, algo, attribute_count));
        raw_coords.set(coords);
    });

    // ── Stable iceberg-derived signals (level-1 deps on iceberg) ─────────────
    // These are updated in a direct iceberg subscriber, so they are always
    // consistent with `iceberg` BEFORE `nodes` is rebuilt.  The view closure
    // must only read these signals — never `iceberg` directly — to avoid
    // accessing disposed RwSignals during Leptos reconciliation.
    let edges_signal: RwSignal<Vec<(u32, u32)>> = RwSignal::new(
        iceberg.with_untracked(|ice| ice.poset.covering_edges.clone()),
    );
    let support_signal: RwSignal<Vec<u32>> = RwSignal::new(
        iceberg.with_untracked(|ice| ice.support.clone()),
    );
    let total_signal: RwSignal<u32> = RwSignal::new(initial_total);
    let extents_signal: RwSignal<Vec<BitSet>> = RwSignal::new(
        iceberg.with_untracked(|ice| {
            ice.poset.nodes.iter().map(|(ext, _)| ext.clone()).collect()
        }),
    );

    Effect::new(move |_| {
        iceberg.with(|ice| {
            edges_signal.set(ice.poset.covering_edges.clone());
            support_signal.set(ice.support.clone());
            total_signal.set(ice.total_objects);
            extents_signal.set(ice.poset.nodes.iter().map(|(ext, _)| ext.clone()).collect());
        });
    });

    // ── Node objects (with drag signals) — rebuilt when coords / dims change ───
    // label.0 = None (below text is a separate reactive layer)
    // label.1 = reduced attribute label (italic above, via NodeComp)
    let nodes: RwSignal<Vec<Node>> = RwSignal::new(Vec::new());
    let layout_epoch: RwSignal<u64> = RwSignal::new(0);

    Effect::new(move |_| {
        let raw = raw_coords.get();
        let dims = dimensions.get();
        let scaled = Drawing::new(raw)
            .scale_to_viewport(dims.width, dims.height, dims.margin);
        let graph_nodes = iceberg_graph_nodes.get_untracked();
        let n = graph_nodes.len();
        let mut pos = vec![(dims.width / 2.0, dims.height / 2.0); n];
        for (id, &(x, y)) in scaled.iter().enumerate() {
            if id < n { pos[id] = (x, y); }
        }
        let new_nodes = graph_nodes
            .iter()
            .map(|gn| {
                let (x, y) = pos.get(gn.id).copied().unwrap_or((dims.width / 2.0, dims.height / 2.0));
                Node::new(gn.id, (None, gn.attr_label.clone()), x, y)
            })
            .collect::<Vec<_>>();
        nodes.set(new_nodes);
        layout_epoch.update(|e| *e = e.wrapping_add(1));
    });

    // ── Event handlers: threshold inputs (live update on on:change) ───────────
    let on_abs_change = move |ev: leptos::ev::Event| {
        let target = ev.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>();
        let total = context.with_untracked(|ctx| ctx.objects.len() as u32);
        if let Ok(val) = target.value().parse::<u32>() {
            threshold_abs.set(val.clamp(0, total));
        }
    };

    let on_pct_change = move |ev: leptos::ev::Event| {
        let target = ev.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>();
        let total = context.with_untracked(|ctx| ctx.objects.len() as u32);
        if let Ok(pct) = target.value().parse::<f64>() {
            let abs = pct_to_abs(pct.clamp(0.0, 100.0), total).clamp(0, total);
            threshold_abs.set(abs);
        }
    };

    // ── Event handlers: canvas resize ─────────────────────────────────────────
    let on_width_change = move |ev: leptos::ev::Event| {
        let target: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into();
        if let Ok(val) = target.value().parse::<f64>() {
            dimensions.update(|d| d.width = val.clamp(200.0, 2000.0));
        }
    };

    let on_height_change = move |ev: leptos::ev::Event| {
        let target: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into();
        if let Ok(val) = target.value().parse::<f64>() {
            dimensions.update(|d| d.height = val.clamp(200.0, 2000.0));
        }
    };

    let on_mouse_down_width = move |ev: leptos::ev::MouseEvent| {
        is_resizing.set(true);
        resize_type.set(1);
        resize_start_x.set(ev.client_x() as f64);
        resize_start_width.set(dimensions.get_untracked().width);
        ev.prevent_default();
    };

    let on_mouse_down_height = move |ev: leptos::ev::MouseEvent| {
        is_resizing.set(true);
        resize_type.set(2);
        resize_start_y.set(ev.client_y() as f64);
        resize_start_height.set(dimensions.get_untracked().height);
        ev.prevent_default();
    };

    let on_mouse_down_corner = move |ev: leptos::ev::MouseEvent| {
        is_resizing.set(true);
        resize_type.set(3);
        resize_start_x.set(ev.client_x() as f64);
        resize_start_y.set(ev.client_y() as f64);
        resize_start_width.set(dimensions.get_untracked().width);
        resize_start_height.set(dimensions.get_untracked().height);
        ev.prevent_default();
    };

    let move_handle = window_event_listener(leptos::ev::mousemove, move |ev: leptos::ev::MouseEvent| {
        if !is_resizing.get_untracked() { return; }
        let rtype = resize_type.get_untracked();
        if rtype == 1 {
            let delta = ev.client_x() as f64 - resize_start_x.get_untracked();
            let new_w = (resize_start_width.get_untracked() + delta).clamp(200.0, 2000.0);
            dimensions.update(|d| d.width = new_w);
            if let Some(inp) = width_input_ref.get_untracked() { inp.set_value(&new_w.to_string()); }
        } else if rtype == 2 {
            let delta = ev.client_y() as f64 - resize_start_y.get_untracked();
            let new_h = (resize_start_height.get_untracked() + delta).clamp(200.0, 2000.0);
            dimensions.update(|d| d.height = new_h);
            if let Some(inp) = height_input_ref.get_untracked() { inp.set_value(&new_h.to_string()); }
        } else if rtype == 3 {
            let dx = ev.client_x() as f64 - resize_start_x.get_untracked();
            let dy = ev.client_y() as f64 - resize_start_y.get_untracked();
            let avg = (dx + dy) / 2.0;
            let new_w = (resize_start_width.get_untracked() + avg).clamp(200.0, 2000.0);
            let new_h = (resize_start_height.get_untracked() + avg).clamp(200.0, 2000.0);
            dimensions.update(|d| { d.width = new_w; d.height = new_h; });
            if let Some(inp) = width_input_ref.get_untracked() { inp.set_value(&new_w.to_string()); }
            if let Some(inp) = height_input_ref.get_untracked() { inp.set_value(&new_h.to_string()); }
        }
    });

    let up_handle = window_event_listener(leptos::ev::mouseup, move |_| {
        is_resizing.set(false);
        resize_type.set(0);
    });

    on_cleanup(move || {
        move_handle.remove();
        up_handle.remove();
    });

    let concept_count =
        Signal::derive(move || count(iceberg.with(|ice| ice.poset.nodes.len()), "concept"));

    view! {
        <div class="flex items-start gap-6">
            <Panel title=|| "Iceberg Lattice" meta=concept_count class="min-w-0 flex-1">
                <div class="overflow-auto p-4">
                // ── Canvas + drag handles ─────────────────────────────────────────
                    <div class="relative w-fit">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            style:width=move || format!("{}px", dimensions.get().width)
                            style:height=move || format!("{}px", dimensions.get().height)
                            node_ref=svg_ref
                            class="bg-white"
                        >
                            <rect width="100%" height="100%" x="0" y="0"
                                fill="white" stroke="#5C697133" stroke-width="2"/>

                            {move || {
                                let current_nodes = nodes.get();
                                let dims = dimensions.get();

                                if let Some(off) = offset.get() {
                                    // Read edges UNTRACKED: edges_signal is always updated before
                                    // nodes (level-1 vs level-2 from iceberg), so by the time this
                                    // closure fires (due to `nodes` change) the edges are already
                                    // current.  Subscribing to edges_signal here would cause the
                                    // closure to run a second time with *old* current_nodes, creating
                                    // EdgeComps that hold soon-to-be-disposed x_signal/y_signal
                                    // references → disposed-signal panic when NodeComp init fires
                                    // node.x_signal.set() and the queued render effect reads stale
                                    // signals.
                                    let edges = edges_signal.get_untracked();
                                    let n = current_nodes.len();

                                    // Edges — reactive to x_signal/y_signal via EdgeComp
                                    let edges_view = edges.iter().map(|&(u, v)| {
                                        let u = u as usize;
                                        let v = v as usize;
                                        if u < n && v < n {
                                            view! {
                                                <EdgeComp
                                                    start=(current_nodes[u].x_signal, current_nodes[u].y_signal)
                                                    end=(current_nodes[v].x_signal, current_nodes[v].y_signal)
                                                />
                                            }
                                        } else {
                                            view! {
                                                <EdgeComp
                                                    start=(RwSignal::new(0.0), RwSignal::new(0.0))
                                                    end=(RwSignal::new(0.0), RwSignal::new(0.0))
                                                />
                                            }
                                        }
                                    }).collect::<Vec<_>>();

                                    // Draggable nodes (attr label above; no below label in NodeComp)
                                    // Keyed by (layout_epoch, id) so remounting on layout change.
                                    let epoch_snap = layout_epoch.get();
                                    let node_list = current_nodes.clone();

                                    // Below labels — separate reactive layer.
                                    // Uses x_signal/y_signal for position (follows drag).
                                    // Inner closures read label_mode + iceberg reactively,
                                    // so they update on label_mode change without remounting nodes.
                                    let radius = dims.radius;
                                    let font_size = dims.font_size;
                                    let below_labels_view = current_nodes.iter().enumerate().map(|(i, node)| {
                                        let x_sig = node.x_signal;
                                        let y_sig = node.y_signal;
                                        view! {
                                            <g>
                                                // White outline pass
                                                <text
                                                    font-size=font_size dy=".35em" text-anchor="middle"
                                                    stroke="white" stroke-width="0.3em" font-family="monospace"
                                                    x=move || x_sig.get()
                                                    y=move || y_sig.get() + radius * 2.8
                                                >
                                                    {move || {
                                                        let mode = label_mode.get();
                                                        let t = match mode {
                                                            LabelMode::SupportAbsolute => {
                                                                support_signal.with(|s| s.get(i).map(|v| v.to_string()).unwrap_or_default())
                                                            }
                                                            LabelMode::SupportRelative => {
                                                                let s = support_signal.with(|v| v.get(i).copied().unwrap_or(0));
                                                                let total = total_signal.get();
                                                                if total > 0 { format!("{:.0}%", s as f64 * 100.0 / total as f64) } else { "0%".to_string() }
                                                            }
                                                            LabelMode::Objects => {
                                                                let extents = extents_signal.get();
                                                                let edge_list = edges_signal.get_untracked();
                                                                let objs = context.with_untracked(|ctx| ctx.objects.clone());
                                                                // Reduced labeling: only show objects not in any
                                                                // directly lower (more specific) node's extent.
                                                                // (u, v) means u ≺ v, so children of i are all u where (u, i) is an edge.
                                                                if let Some(ext) = extents.get(i) {
                                                                    let mut reduced = ext.clone();
                                                                    for &(u, v) in &edge_list {
                                                                        if v as usize == i
                                                                            && let Some(child_ext) =
                                                                                extents.get(u as usize)
                                                                        {
                                                                            reduced.difference_with(child_ext);
                                                                        }
                                                                    }

                                                                    obj_label_str(&reduced, &objs)
                                                                } else {
                                                                    String::new()
                                                                }
                                                            }
                                                        };
                                                        "0".repeat(t.len())
                                                    }}
                                                </text>
                                                // Fill pass
                                                <text
                                                    font-size=font_size dy=".35em" text-anchor="middle"
                                                    fill="black" font-family="monospace"
                                                    x=move || x_sig.get()
                                                    y=move || y_sig.get() + radius * 2.8
                                                >
                                                    {move || {
                                                        let mode = label_mode.get();
                                                        match mode {
                                                            LabelMode::SupportAbsolute => {
                                                                support_signal.with(|s| s.get(i).map(|v| v.to_string()).unwrap_or_default())
                                                            }
                                                            LabelMode::SupportRelative => {
                                                                let s = support_signal.with(|v| v.get(i).copied().unwrap_or(0));
                                                                let total = total_signal.get();
                                                                if total > 0 { format!("{:.0}%", s as f64 * 100.0 / total as f64) } else { "0%".to_string() }
                                                            }
                                                            LabelMode::Objects => {
                                                                let extents = extents_signal.get();
                                                                let edge_list = edges_signal.get_untracked();
                                                                let objs = context.with_untracked(|ctx| ctx.objects.clone());
                                                                if let Some(ext) = extents.get(i) {
                                                                    let mut reduced = ext.clone();
                                                                    for &(u, v) in &edge_list {
                                                                        if v as usize == i
                                                                            && let Some(child_ext) =
                                                                                extents.get(u as usize)
                                                                        {
                                                                            reduced.difference_with(child_ext);
                                                                        }
                                                                    }
                                                                    obj_label_str(&reduced, &objs)
                                                                } else {
                                                                    String::new()
                                                                }
                                                            }
                                                        }
                                                    }}
                                                </text>
                                            </g>
                                        }
                                    }).collect::<Vec<_>>();

                                    Either::Left(view! {
                                        {edges_view}
                                        <For
                                            each=move || {
                                                node_list.clone().into_iter()
                                                    .map(move |n| (epoch_snap, n))
                                                    .collect::<Vec<_>>()
                                            }
                                            key=|(epoch, n)| (*epoch, n.id)
                                            children=move |(_epoch, node)| {
                                                view! {
                                                    <NodeComp node=node offset=off dimensions=dims.clone()/>
                                                }
                                            }
                                        />
                                        {below_labels_view}
                                    })
                                } else {
                                    Either::Right(())
                                }
                            }}
                        </svg>

                        // Right drag handle (resize width)
                        <div
                            class="absolute top-0 right-0 w-6 h-full cursor-ew-resize \
                                   hover:bg-dhbw-red/10 transition-colors flex items-center justify-center"
                            on:mousedown=on_mouse_down_width
                        >
                            <div class="w-1 h-8 bg-dhbw-gray-50 rounded"></div>
                        </div>
                        // Bottom drag handle (resize height)
                        <div
                            class="absolute bottom-0 left-0 w-full h-6 cursor-ns-resize \
                                   hover:bg-dhbw-red/10 transition-colors flex items-center justify-center"
                            on:mousedown=on_mouse_down_height
                        >
                            <div class="w-8 h-1 bg-dhbw-gray-50 rounded"></div>
                        </div>
                        // Corner drag handle
                        <div
                            class="absolute bottom-0 right-0 w-8 h-8 cursor-nwse-resize"
                            on:mousedown=on_mouse_down_corner
                        />
                    </div>
                </div>
            </Panel>

            // ── Controls, at the right edge ───────────────────────────────────
            <Panel title=|| "Options" class="w-56 shrink-0">
                <div class="flex flex-col gap-4 p-4">

                    // Min. Support inputs (live update on change — no Apply button)
                    <div class="flex flex-col gap-2">
                        <label class=CONTROL_LABEL>
                            "Min. Support"
                        </label>
                        <div class="flex items-center gap-1">
                            <input
                                type="number"
                                min="0"
                                prop:max=move || context.with(|ctx| ctx.objects.len().to_string())
                                prop:value=move || threshold_abs.get().to_string()
                                on:change=on_abs_change
                                class=INPUT
                            />
                            <span class="text-dhbw-gray-50 text-xs shrink-0">"obj"</span>
                        </div>
                        <div class="flex items-center gap-1">
                            <input
                                type="number"
                                min="0"
                                max="100"
                                prop:value=move || {
                                    let total = context.with(|ctx| ctx.objects.len() as u32);
                                    abs_to_pct_str(threshold_abs.get(), total)
                                }
                                on:change=on_pct_change
                                class=INPUT
                            />
                            <span class="text-dhbw-gray-50 text-xs shrink-0">"%"</span>
                        </div>
                        <span class="text-dhbw-gray-50 text-xs">
                            {move || {
                                let applied = threshold_abs.get();
                                let total = context.with(|ctx| ctx.objects.len());
                                format!("{applied} of {}", count(total, "object"))
                            }}
                        </span>
                    </div>

                    // Layout selector
                    <div class="flex flex-col gap-1">
                        <label class=CONTROL_LABEL>
                            "Layout"
                        </label>
                        <select
                            on:change=move |ev| {
                                let sel = ev.target().unwrap().unchecked_into::<web_sys::HtmlSelectElement>();
                                layout_algorithm.set(match sel.value().as_str() {
                                    "Sugiyama" => LayoutAlgorithm::Sugiyama,
                                    "DimFlux" => LayoutAlgorithm::DimFlux,
                                    _ => LayoutAlgorithm::DimDraw,
                                });
                            }
                            class=INPUT
                        >
                            <option value="DimDraw"
                                selected=move || layout_algorithm.get() == LayoutAlgorithm::DimDraw>
                                "DimDraw"
                            </option>
                            <option value="DimFlux"
                                selected=move || layout_algorithm.get() == LayoutAlgorithm::DimFlux>
                                "DimFlux"
                            </option>
                            <option value="Sugiyama"
                                selected=move || layout_algorithm.get() == LayoutAlgorithm::Sugiyama>
                                "Sugiyama"
                            </option>
                        </select>
                    </div>

                    // Below-label selector
                    <div class="flex flex-col gap-1">
                        <label class=CONTROL_LABEL>
                            "Show below"
                        </label>
                        <select
                            on:change=move |ev| {
                                let sel = ev.target().unwrap().unchecked_into::<web_sys::HtmlSelectElement>();
                                label_mode.set(match sel.value().as_str() {
                                    "pct"     => LabelMode::SupportRelative,
                                    "objects" => LabelMode::Objects,
                                    _         => LabelMode::SupportAbsolute,
                                });
                            }
                            class=INPUT
                        >
                            <option value="abs"
                                selected=move || label_mode.get() == LabelMode::SupportAbsolute>
                                "Support (n)"
                            </option>
                            <option value="pct"
                                selected=move || label_mode.get() == LabelMode::SupportRelative>
                                "Support (%)"
                            </option>
                            <option value="objects"
                                selected=move || label_mode.get() == LabelMode::Objects>
                                "Objects"
                            </option>
                        </select>
                    </div>

                    // Canvas Width
                    <div class="flex flex-col gap-1">
                        <label class=CONTROL_LABEL>
                            "Width"
                        </label>
                        <input
                            type="number" min="200" max="2000" value="600"
                            node_ref=width_input_ref
                            on:change=on_width_change
                            class=INPUT
                        />
                    </div>

                    // Canvas Height
                    <div class="flex flex-col gap-1">
                        <label class=CONTROL_LABEL>
                            "Height"
                        </label>
                        <input
                            type="number" min="200" max="2000" value="600"
                            node_ref=height_input_ref
                            on:change=on_height_change
                            class=INPUT
                        />
                    </div>

                    // Save SVG
                    <div class="mt-2">
                        <SvgDownloadComp node_ref=svg_ref/>
                    </div>
                </div>
            </Panel>
        </div>
    }
}

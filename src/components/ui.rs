//! Shared chrome: one panel frame, one set of control styles and one way of
//! typesetting attribute sets and implications, so the views stay consistent.

use leptos::prelude::*;

/// Solid accent button for the primary action of a view.
pub const BTN_PRIMARY: &str = "inline-flex items-center justify-center rounded-md bg-dhbw-red \
    px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-red-700 \
    disabled:cursor-not-allowed disabled:bg-dhbw-gray-25 disabled:text-dhbw-gray-50 \
    disabled:hover:bg-dhbw-gray-25";

/// Outlined button for secondary actions next to a [`BTN_PRIMARY`].
pub const BTN_SECONDARY: &str = "inline-flex items-center justify-center rounded-md \
    border border-dhbw-gray-25 bg-white px-4 py-2 text-sm font-medium text-dhbw-gray \
    transition-colors hover:bg-dhbw-gray/5";

/// Borderless glyph button used for the add/remove controls inside tables.
pub const BTN_ICON: &str = "inline-flex h-5 w-5 items-center justify-center rounded text-dhbw-gray-50 \
    leading-none transition-colors hover:bg-dhbw-red/10 hover:text-dhbw-red";

/// Sidebar entry — shared by the view switcher and the file actions, so the
/// two kinds of entry cannot drift apart.
pub const NAV_ITEM: &str = "flex w-full items-center gap-2 rounded-md px-3 py-2.5 text-left \
    text-sm transition-colors";

/// Added to [`NAV_ITEM`] for the entry of the view currently on screen.
pub const NAV_ITEM_ACTIVE: &str = "bg-dhbw-red font-medium text-white shadow-sm";

/// Added to [`NAV_ITEM`] for every other entry.
pub const NAV_ITEM_IDLE: &str = "text-dhbw-gray hover:bg-dhbw-gray/5";

/// Text and number inputs, and select boxes.
pub const INPUT: &str = "w-full rounded-md border border-dhbw-gray-25 px-3 py-2 text-sm \
    text-dhbw-gray transition-colors focus:border-dhbw-red focus:outline-none";

/// Caption above an input in a control panel.
pub const CONTROL_LABEL: &str = "text-xs font-medium uppercase tracking-wide text-dhbw-gray-50";

/// Column caption inside a data table.
pub const TH: &str = "px-4 py-2 text-left text-xs font-medium uppercase tracking-wide \
    text-dhbw-gray-50";

/// Table cell holding an [`Arrow`]. It carries the same monospaced font as the
/// set text beside it: `align-middle` resolves against the parent's x-height,
/// so a sans-serif cell would place the arrow a pixel off from its own row.
pub const ARROW_CELL: &str = "px-2 py-2 text-center align-top font-mono";

/// Attribute and object sets — always monospaced. Wraps between words, and
/// only breaks inside one when a single name is too long for the column.
pub const SET_TEXT: &str = "font-mono text-dhbw-gray [overflow-wrap:anywhere]";

/// The implication arrow — drawn as an inline icon rather than typed as a
/// glyph, so it has the same size and weight in every font. It takes the
/// colour of the text around it.
#[component]
pub fn Arrow() -> impl IntoView {
    view! {
        <svg
            class="inline-block h-4 w-7 shrink-0 align-middle text-dhbw-gray"
            viewBox="0 0 28 16"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            role="img"
            aria-label="implies"
        >
            <path d="M2 8h23M18 2l7 6-7 6"/>
        </svg>
    }
}

/// Font stack for text inside the lattice drawings. Spelled out rather than
/// inherited so that a downloaded SVG looks the same outside the browser.
pub const SVG_FONT: &str = "ui-sans-serif, system-ui, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif";

/// Text field that doubles as a panel title, e.g. the name of the context.
/// Font and colour come from the surrounding heading; only the underline
/// tells it apart, and only when hovered or focused.
pub const TITLE_INPUT: &str = "w-full min-w-0 border-b border-transparent bg-transparent \
    transition-colors placeholder:font-normal placeholder:text-dhbw-gray-50 \
    hover:border-dhbw-gray-25 focus:border-dhbw-red focus:outline-none";

/// A titled card. Every box on screen is one of these, so the views share a
/// single frame. The caller supplies the body, so it can scroll, stretch or
/// carry a `node_ref` as the view needs.
#[component]
pub fn Panel(
    /// Heading; usually plain text, but any view works (see [`TITLE_INPUT`]).
    #[prop(into)]
    title: ViewFnOnce,
    /// Secondary text at the right end of the heading, e.g. `"13 concepts"`.
    #[prop(optional, into)]
    meta: Option<Signal<String>>,
    /// Extra classes for the outer frame, e.g. flex sizing.
    #[prop(optional, into)]
    class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <section class=format!(
            "flex flex-col overflow-hidden rounded-lg border border-dhbw-gray-25 bg-white {class}"
        )>
            <header class="flex flex-shrink-0 items-baseline gap-3 border-b border-dhbw-gray-25 bg-gray-50 px-4 py-2.5">
                <h2 class="min-w-0 flex-1 text-sm font-semibold text-dhbw-gray">{title.run()}</h2>
                {meta.map(|meta| view! {
                    <span class="shrink-0 text-xs text-dhbw-gray-50">{move || meta.get()}</span>
                })}
            </header>
            {children()}
        </section>
    }
}

/// An implication typeset on one wrapping line: `{premise} → {conclusion}`.
#[component]
pub fn Implication(
    #[prop(into)] premise: Signal<String>,
    #[prop(into)] conclusion: Signal<String>,
) -> impl IntoView {
    view! {
        <p class=format!("{SET_TEXT} text-sm leading-relaxed")>
            {move || premise.get()}
            <span class="mx-2"><Arrow/></span>
            {move || conclusion.get()}
        </p>
    }
}

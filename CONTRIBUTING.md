# Contributing to odis-web

`odis-web` is a Leptos/WASM front-end for the odis FCA library. It compiles to
WebAssembly via Cargo and is bundled with [Trunk](https://trunkrs.dev/).

---

## Prerequisites

| Tool | Install |
|------|---------|
| Rust (stable) | `rustup update` |
| WASM target | `rustup target add wasm32-unknown-unknown` |
| Trunk | `cargo install trunk` |

> **No Node.js or npm needed.** Trunk fetches and manages Tailwind CSS 3.4
> internally via the `[tools]` section in `Trunk.toml`.

---

## Development

Start a local dev server with live reload from the `odis-web/` directory:

```sh
cd odis-web
trunk serve
```

Trunk serves the app at `http://localhost:8080` by default and rebuilds on
source changes.

---

## Production Build

```sh
cd odis-web
trunk build --release
```

Output lands in `odis-web/dist/`. Copy the contents to `odis.github.io/` to
publish a new version of the hosted app.

---

## Repository Layout

```
odis-web/
├── index.html               # HTML entry point (Trunk reads link/script tags)
├── Trunk.toml               # Trunk configuration (Tailwind version, etc.)
└── src/
    ├── main.rs              # App root: signal setup, provide_context
    ├── js_fn.rs             # JS interop helpers (clipboard, SVG download)
    ├── components/
    │   ├── layout.rs        # View enum, Sidebar, Header
    │   ├── views.rs         # Top-level view components
    │   ├── context.rs       # FormalContext editor / file loader
    │   ├── graph.rs         # Concept-lattice SVG graph
    │   ├── iceberg.rs       # Iceberg-lattice view
    │   ├── exploration.rs   # Attribute exploration dialog
    │   ├── table.rs         # Cross-table editor
    │   ├── svg_download.rs  # SVG download button
    │   └── svg/             # Low-level SVG primitives
    ├── core/
    │   ├── export.rs        # Context serialisation helpers
    │   ├── formatters.rs    # Human-readable label formatters
    │   └── layout_math.rs   # Coordinate helpers
    └── utils/
        └── browser.rs      # Browser API wrappers
```

---

## Reactive Data Flow

The app root (`main.rs`) creates two shared signals and makes them available
to the entire component tree:

```rust
// main.rs — App()
let context: RwSignal<FormalContext<String>> = RwSignal::new(create_default_context());
provide_context(context);

let context_version: RwSignal<u64> = RwSignal::new(0);
provide_context(context_version);
```

Any component that needs the context reads it with:

```rust
let context = use_context::<RwSignal<FormalContext<String>>>().unwrap();
```

When a component mutates the context it increments `context_version` to trigger
reactive recomputation in views that depend on derived data:

```rust
let version = use_context::<RwSignal<u64>>().unwrap();
context.update(|ctx| { /* mutation */ });
version.update(|v| *v += 1);
```

---

## Adding a New View

1. **Create the component** — add `my_view.rs` inside `src/components/` and
   export a single `#[component] pub fn MyView() -> impl IntoView` function.
   Read the shared context at the top:

   ```rust
   let context = use_context::<RwSignal<FormalContext<String>>>().unwrap();
   ```

2. **Add a `View` variant** — open `src/components/layout.rs` and append your
   variant to the `View` enum:

   ```rust
   pub enum View {
       FormalContext,
       Concepts,
       ConceptLattice,
       CanonicalBasis,
       Exploration,
       IcebergLattice,
       MyView,        // ← add here
   }
   ```

3. **Wire the Sidebar** — still in `layout.rs`, add a sidebar button inside
   the `Sidebar` component that sets the current view:

   ```rust
   <SidebarButton
       label="My View"
       active=Signal::derive(move || current_view.get() == View::MyView)
       on_click=move || current_view.set(View::MyView)
   />
   ```

4. **Render the view** — open `src/components/views.rs` and add a branch in
   the view-switching block:

   ```rust
   View::MyView => view! { <MyView /> },
   ```

   Import the new module at the top of `views.rs`:

   ```rust
   use super::my_view::MyView;
   ```

---

## Component Inventory

| File | Purpose |
|------|---------|
| `layout.rs` | `View` enum, `Sidebar`, `Header` |
| `views.rs` | Dispatches rendering by current `View` |
| `context.rs` | Context editor and `.cxt` file import |
| `graph.rs` | Concept-lattice SVG with pan/zoom |
| `iceberg.rs` | Iceberg-lattice (min-support filter) view |
| `exploration.rs` | Attribute exploration |
| `table.rs` | Cross-table incidence editor |
| `svg_download.rs` | SVG export / clipboard button |
| `svg/` | Low-level SVG node/edge rendering primitives |

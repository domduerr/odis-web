use leptos::prelude::*;

#[component]
pub fn EdgeComp(
    start: (RwSignal<f64>, RwSignal<f64>),
    end: (RwSignal<f64>, RwSignal<f64>),
) -> impl IntoView {
    view! {
        <g>
            <path
                d=move || {
                    let a = format!("M{} {} L{} {}", start.0.get(), start.1.get(), end.0.get(), end.1.get());
                    a
                }
                stroke="black"
                stroke-width="2"
            />
        </g>
    }
}

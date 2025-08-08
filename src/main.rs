use console_error_panic_hook;
use leptos::{either::Either, html::Input, logging, prelude::*, task::spawn_local};
use web_sys::SubmitEvent;

use odis::{self, FormalContext};

use crate::components::table::TableComp;

mod components {
    pub mod checkbox;
    pub mod download;
    pub mod exploration;
    pub mod graph;
    pub mod svg_download;
    pub mod table;
    pub mod svg {
        pub mod edge;
        pub mod node;
    }
}

mod js_fn;

#[component]
pub fn App() -> impl IntoView {
    let context = RwSignal::new(None::<FormalContext<String>>);
    let input_element: NodeRef<Input> = NodeRef::new();

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        if "".to_string() == input_element.get().unwrap().value() {
            logging::log!("No file selected yet");
            return;
        }

        let fileList = input_element
            .get()
            .expect("<input> should be mounted")
            .files();
        let file = fileList.unwrap().item(0).unwrap();

        spawn_local(async move {
            let contents = js_fn::file_contents(file.clone()).await;
            context.set(Some(
                FormalContext::<String>::from(contents.as_bytes()).expect("parsing error"),
            ));
        });
    };

    view! {
        <h1>"Concept explorer"</h1>

        <form on:submit=on_submit style:display="inline" style:padding-right="20px">
            <input type="file" node_ref=input_element/>
            <input type="submit" value="Submit"/>
        </form>

        {move || {
            if let None = context.get() {
                Either::Left(view! {<TableComp context=context/>})
            } else {
                Either::Right(view! {<TableComp context=context/>})
            }
        }}
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

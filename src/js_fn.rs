use wasm_bindgen_futures::JsFuture;
use web_sys::File;

pub async fn file_contents(file: File) -> String {
    JsFuture::from(file.text())
        .await
        .unwrap_or("default".into())
        .as_string()
        .unwrap()
}

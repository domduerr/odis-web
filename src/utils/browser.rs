use leptos::wasm_bindgen::JsCast;
use web_sys::{wasm_bindgen::JsValue, Blob, BlobPropertyBag, File, Url};

pub fn trigger_text_download(
    content: &str,
    filename: &str,
    mime_type: &str,
) -> Result<(), JsValue> {
    let content_vec = vec![content.to_string()];

    let file = File::new_with_u8_slice_sequence(&JsValue::from(content_vec), filename)?;
    let url = Url::create_object_url_with_blob(&file)?;

    trigger_download_from_url(&url, filename, mime_type)
}

pub fn trigger_blob_download(blob: &Blob, filename: &str, mime_type: &str) -> Result<(), JsValue> {
    let url = Url::create_object_url_with_blob(blob)?;
    trigger_download_from_url(&url, filename, mime_type)
}

fn trigger_download_from_url(url: &str, filename: &str, _mime_type: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::NULL)?;
    let document = window.document().ok_or_else(|| JsValue::NULL)?;

    let link = document.create_element("a")?;
    link.set_attribute("download", filename)?;
    link.set_attribute("href", url)?;

    let html_link: &web_sys::HtmlElement = link.dyn_ref().ok_or_else(|| JsValue::NULL)?;

    html_link.click();

    Ok(())
}

pub fn serialize_and_download_svg(
    svg_element: &web_sys::Element,
    filename: &str,
) -> Result<(), JsValue> {
    let serializer = web_sys::XmlSerializer::new().map_err(|_| JsValue::NULL)?;

    let xml_string = serializer.serialize_to_string(svg_element)?;
    let xml_vec = vec![xml_string];

    let property_bag = BlobPropertyBag::new();
    property_bag.set_type("image/svg+xml;charset=utf-8");

    let blob =
        Blob::new_with_u8_array_sequence_and_options(&JsValue::from(xml_vec), &property_bag)?;

    trigger_blob_download(&blob, filename, "image/svg+xml")
}

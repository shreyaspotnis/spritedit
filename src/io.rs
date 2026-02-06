use crate::sprite::Sprite;

pub fn sprite_to_png(sprite: &Sprite) -> Vec<u8> {
    let img =
        image::RgbaImage::from_raw(sprite.width, sprite.height, sprite.pixels.clone())
            .expect("Invalid sprite dimensions");
    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    image::ImageEncoder::write_image(
        encoder,
        &img,
        sprite.width,
        sprite.height,
        image::ExtendedColorType::Rgba8,
    )
    .expect("Failed to encode PNG");
    buf
}

pub fn png_to_sprite(data: &[u8]) -> Option<Sprite> {
    let img = image::load_from_memory(data).ok()?.to_rgba8();
    Some(Sprite {
        width: img.width(),
        height: img.height(),
        pixels: img.into_raw(),
    })
}

// --- Native file dialogs ---

#[cfg(not(target_arch = "wasm32"))]
pub mod native {
    use std::io::Read;

    pub fn open_file_dialog() -> Option<Vec<u8>> {
        let path = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "gif", "bmp"])
            .pick_file()?;
        std::fs::read(path).ok()
    }

    pub fn fetch_url(url: &str) -> Result<Vec<u8>, String> {
        let response = ureq::get(url)
            .call()
            .map_err(|e| format!("Fetch failed: {e}"))?;
        let mut buf = Vec::new();
        response
            .into_reader()
            .read_to_end(&mut buf)
            .map_err(|e| format!("Read failed: {e}"))?;
        Ok(buf)
    }

    pub fn save_file_dialog(data: &[u8]) -> bool {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG Image", &["png"])
            .set_file_name("sprite.png")
            .save_file()
        {
            std::fs::write(path, data).is_ok()
        } else {
            false
        }
    }
}

// --- WASM file I/O ---

#[cfg(target_arch = "wasm32")]
pub mod web {
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    thread_local! {
        pub static PENDING_FILE: RefCell<Option<Vec<u8>>> = RefCell::new(None);
    }

    pub fn open_file_dialog() {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let input: web_sys::HtmlInputElement = document
            .create_element("input")
            .unwrap()
            .dyn_into()
            .unwrap();
        input.set_type("file");
        input.set_accept("image/*");

        let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
            let input: web_sys::HtmlInputElement =
                e.target().unwrap().dyn_into().unwrap();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let reader = web_sys::FileReader::new().unwrap();
                    let reader_clone = reader.clone();
                    let onload =
                        Closure::wrap(Box::new(move |_: web_sys::Event| {
                            if let Ok(result) = reader_clone.result() {
                                let array = js_sys::Uint8Array::new(&result);
                                let bytes = array.to_vec();
                                PENDING_FILE
                                    .with(|f| *f.borrow_mut() = Some(bytes));
                            }
                        })
                            as Box<dyn FnMut(_)>);
                    reader
                        .set_onload(Some(onload.as_ref().unchecked_ref()));
                    onload.forget();
                    reader.read_as_array_buffer(&file).unwrap();
                }
            }
        }) as Box<dyn FnMut(_)>);

        input
            .add_event_listener_with_callback(
                "change",
                closure.as_ref().unchecked_ref(),
            )
            .unwrap();
        closure.forget();
        input.click();
    }

    pub fn save_file(data: &[u8], filename: &str) {
        let array = js_sys::Uint8Array::from(data);
        let blob_parts = js_sys::Array::new();
        blob_parts.push(&array.buffer());

        let options = web_sys::BlobPropertyBag::new();
        options.set_type("image/png");

        let blob = web_sys::Blob::new_with_buffer_source_sequence_and_options(
            &blob_parts,
            &options,
        )
        .unwrap();
        let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let anchor: web_sys::HtmlAnchorElement = document
            .create_element("a")
            .unwrap()
            .dyn_into()
            .unwrap();
        anchor.set_href(&url);
        anchor.set_download(filename);
        anchor.click();

        let _ = web_sys::Url::revoke_object_url(&url);
    }

    pub fn fetch_url(url: &str) {
        let url = url.to_string();
        wasm_bindgen_futures::spawn_local(async move {
            let opts = web_sys::RequestInit::new();
            opts.set_method("GET");
            opts.set_mode(web_sys::RequestMode::Cors);

            let request = web_sys::Request::new_with_str_and_init(&url, &opts)
                .expect("Failed to create request");

            let window = web_sys::window().unwrap();
            let resp_value =
                wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
                    .await;
            let resp_value = match resp_value {
                Ok(v) => v,
                Err(_) => return,
            };
            let resp: web_sys::Response = resp_value.dyn_into().unwrap();
            if !resp.ok() {
                return;
            }
            let array_buffer = match resp.array_buffer() {
                Ok(promise) => {
                    match wasm_bindgen_futures::JsFuture::from(promise).await {
                        Ok(buf) => buf,
                        Err(_) => return,
                    }
                }
                Err(_) => return,
            };
            let uint8_array = js_sys::Uint8Array::new(&array_buffer);
            let bytes = uint8_array.to_vec();
            PENDING_FILE.with(|f| *f.borrow_mut() = Some(bytes));
        });
    }

    pub fn check_pending_file() -> Option<Vec<u8>> {
        PENDING_FILE.with(|f| f.borrow_mut().take())
    }
}

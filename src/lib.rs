mod app;
mod storage;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn wasm_start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Info);

    repose_platform::web::run_web_app(
        app::app as fn(&mut repose_core::Scheduler) -> repose_core::View,
        repose_platform::web::WebOptions::new(None),
    )
}

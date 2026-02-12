mod app;
mod storage;

use repose_ui::overlay::SnackbarController;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn wasm_start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Info);

    let mut opts = repose_platform::web::WebOptions::new(None);
    opts.set_fullscreen(true);
    opts.set_auto_root_scroll(false);
    opts.set_continuous_redraw(true);

    repose_platform::web::run_web_app_with_snackbar(
        |s, _rc| app::app(s),
        opts,
        Some(Rc::new(|ms| SnackbarController::tick_for_frame(ms))),
    )
}

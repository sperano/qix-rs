//! The web entry point: the game on the page's canvas, built with `trunk`
//! from `index.html`.

use eframe::wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, HtmlCanvasElement};

use crate::app;

/// The `id` of the canvas the game paints on, in `index.html`.
const CANVAS_ID: &str = "qix_canvas";

/// The `id` of the element `index.html` shows until the game starts.
const LOADING_ID: &str = "loading_text";

/// Shown in place of the loading text when the game fails to start.
const START_FAILED_HTML: &str =
    "<p>The game failed to start. See the developer console for details.</p>";

/// Starts the game on the canvas once the browser runs the future.
pub fn start() {
    wasm_bindgen_futures::spawn_local(async {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            web_sys::console::error_1(&"qix: no document to start in".into());
            return;
        };
        let started = run(&document).await;
        report(&document, started);
    });
}

/// Runs the game on the canvas of `document`.
async fn run(document: &Document) -> Result<(), JsValue> {
    let canvas = document
        .get_element_by_id(CANVAS_ID)
        .ok_or_else(|| JsValue::from_str(&format!("no element with id {CANVAS_ID}")))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| JsValue::from_str(&format!("element {CANVAS_ID} is not a canvas")))?;
    eframe::WebRunner::new()
        .start(
            canvas,
            eframe::WebOptions::default(),
            Box::new(|cc| Ok(Box::new(app::QixApp::new(cc.storage)))),
        )
        .await
}

/// Removes the loading text once the game runs, replaces it with an error
/// and logs the cause when it failed to start.
fn report(document: &Document, started: Result<(), JsValue>) {
    let loading = document.get_element_by_id(LOADING_ID);
    match started {
        Ok(()) => {
            if let Some(loading) = loading {
                loading.remove();
            }
        }
        Err(err) => {
            if let Some(loading) = loading {
                loading.set_inner_html(START_FAILED_HTML);
            }
            web_sys::console::error_2(&"qix: failed to start:".into(), &err);
        }
    }
}

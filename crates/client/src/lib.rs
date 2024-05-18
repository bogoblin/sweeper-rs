use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext;
use limelight::{Program, Renderer, DummyBuffer, DrawMode};

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let canvas = document.get_element_by_id("game_canvas").expect("no canvas");
    let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    let gl = canvas.get_context("webgl2")?
        .expect("Can't get webgl2 context")
        .dyn_into::<WebGl2RenderingContext>()?;

    let mut program = Program::new(
        include_str!("../shaders/shader.vert"),
        include_str!("../shaders/shader.frag"),
        DrawMode::Triangles
    );

    let mut renderer = Renderer::new(gl);

    renderer.render(&mut program, &DummyBuffer::new(3)).expect("couldn't render");

    Ok(())
}
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement};
use winit::dpi::{PhysicalPosition, PhysicalSize};

pub struct OverlayCanvas {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
}

pub enum CapturePointerEvents {
    Capture,
    PassThrough
}

impl OverlayCanvas {
    pub fn new(document: Document) -> Self {
        let canvas: HtmlCanvasElement = document.create_element("canvas").unwrap()
            .dyn_into().unwrap();
        let context: CanvasRenderingContext2d = canvas.get_context("2d").unwrap().unwrap()
            .dyn_into().unwrap();
        document.body().unwrap().append_child(&web_sys::Element::from(canvas.clone())).unwrap();

        let mut result = Self {
            canvas,
            context,
        };

        result.capture_pointer_events(CapturePointerEvents::PassThrough);

        result
    }

    pub fn capture_pointer_events(&self, capture: CapturePointerEvents) {
        match capture {
            CapturePointerEvents::Capture => {
                self.canvas.style().remove_property("cursor").unwrap();
                self.canvas.style().remove_property("pointer-events").unwrap();
            }
            CapturePointerEvents::PassThrough => {
                self.canvas.style().set_property("cursor", "not-allowed").unwrap();
                self.canvas.style().set_property("pointer-events", "none").unwrap();
            }
        }
    }

    pub fn set_size(&self, size: PhysicalSize<u32>) {
        self.canvas.set_width(size.width);
        self.canvas.set_height(size.height);
    }

    pub fn text(&self, text: &str, size: f64, anchor: Anchor, position: PhysicalPosition<f64>) {
        self.context.fill_text(text, position.x, position.y).unwrap();
    }
}

pub enum Anchor {
    Left, Middle, Right
}

pub enum CanvasColor {
    Rgba(f64, f64, f64, f64),
    Oklcha(f64, f64, f64, f64),
}
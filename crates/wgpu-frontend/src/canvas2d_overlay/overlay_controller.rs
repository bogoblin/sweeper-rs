use winit::dpi::PhysicalSize;

#[cfg(target_arch = "wasm32")]
use crate::canvas2d_overlay::overlay_canvas::OverlayCanvas;

pub struct OverlayController {
    #[cfg(target_arch = "wasm32")]
    overlay_canvas: OverlayCanvas,
}

impl OverlayController {
    pub fn new() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            overlay_canvas: OverlayCanvas::new(web_sys::window().unwrap().document().unwrap())
        }
    }

    pub fn set_size(&self, size: PhysicalSize<u32>) {
        #[cfg(target_arch = "wasm32")]
        self.overlay_canvas.set_size(size)
    }
}

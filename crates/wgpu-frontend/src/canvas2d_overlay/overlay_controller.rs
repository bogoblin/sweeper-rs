use std::cmp::min;
use chrono::{DateTime, Duration, TimeDelta, Utc};
use winit::dpi::PhysicalSize;
use world::{Event, PublicTile};
#[cfg(target_arch = "wasm32")]
use crate::canvas2d_overlay::overlay_canvas::OverlayCanvas;

#[derive(Default)]
pub struct OverlayController {
    #[cfg(target_arch = "wasm32")]
    overlay_canvas: OverlayCanvas,
    dead: Option<DateTime<Utc>>
}

struct TimeSpan {
    start: DateTime<Utc>,
    duration: TimeDelta
}

impl TimeSpan {
    pub fn end(&self) -> DateTime<Utc> {
        self.start + self.duration
    }
    pub fn time_from_end(&self) -> TimeDelta {
        self.end() - Utc::now()
    }

    pub fn time_from_start(&self) -> TimeDelta {
        Utc::now() - self.start
    }

    pub fn t_elapsed(&self) -> f64 {
        self.time_from_start().num_milliseconds() as f64 / self.duration.num_milliseconds() as f64
    }
}

impl OverlayController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        return;

        let context = &self.overlay_canvas.context;
        context.clear_rect(0.0, 0.0, 2000.0, 2000.0);

        if let Some(dead) = &self.dead {
            let mut alpha = (Utc::now() - dead).num_milliseconds() as f64 / 500.0;
            if alpha > 1.0 { alpha = 1.0 };
            alpha *= 0.3;
            context.set_fill_style_str(&format!("Rgba(255.0, 0.0, 0.0, {alpha})"));
            context.fill_rect(0.0, 0.0, 2000.0, 2000.0);
        }
    }

    pub fn set_size(&self, size: PhysicalSize<u32>) {
        #[cfg(target_arch = "wasm32")]
        self.overlay_canvas.set_size(size)
    }

    pub fn process_event(&mut self, event: Event, is_you: bool) {
        if is_you {
            for tile in event.updated_rect().public_tiles() {
                if matches!(tile, PublicTile::Exploded) {
                    self.dead = Some(Utc::now())
                }
            }
        }
    }
}

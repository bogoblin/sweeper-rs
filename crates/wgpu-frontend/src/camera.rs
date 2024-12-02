use winit::dpi::PhysicalSize;

pub type ClipSpaceVec = [f32; 2];
pub type WorldSpaceVec = [f32; 2];

pub struct Camera {
    center: WorldSpaceVec,
    tile_size: f32,
    view_size: PhysicalSize<u32>,
}

impl Camera {
    pub fn new(tile_size: f32, view_size: &PhysicalSize<u32>) -> Self {
        Self {
            center: Default::default(),
            tile_size,
            view_size: view_size.clone(),
        }
    }

    pub fn tile_size_clip_space(&self) -> ClipSpaceVec {
        [self.tile_size*2.0 / self.view_size.width as f32, self.tile_size*2.0 / self.view_size.height as f32]
    }

    pub fn center(&self) -> WorldSpaceVec {
        self.center
    }

    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        self.view_size = new_size.clone();
    }
}
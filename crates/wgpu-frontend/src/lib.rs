mod texture;
mod camera;
mod tilerender_texture;
mod shader;

use std::collections::{HashSet};
use std::default::Default;
use std::thread::sleep;
use cfg_if::cfg_if;
use cgmath::Vector2;
use chrono::prelude::*;
use chrono::TimeDelta;
use log::info;
use rand::{thread_rng, RngCore};
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowBuilder};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::{CompositeAlphaMode, PresentMode};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use world::{Chunk, ChunkPosition, Position, Tile, World};
use world::events::Event;
use crate::camera::Camera;
use crate::shader::HasBindGroup;
use crate::texture::Texture;
use crate::tilerender_texture::TilerenderTexture;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init().unwrap();
        } else {
            env_logger::init();
        }
    }
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::dpi::PhysicalSize;
        let _ = window.request_inner_size(PhysicalSize::new(1024, 1024));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas()?);
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.")
    }
    
    let mut state = State::new(&window).await;
    let mut surface_configured = false;

    event_loop.run(move |event, control_flow| match event {
        winit::event::Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window.id() => if !state.input(event) {
            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                    ..
                } => control_flow.exit(),
                WindowEvent::Resized(physical_size) => {
                    surface_configured = true;
                    let mut physical_size = physical_size.clone();
                    #[cfg(target_arch = "wasm32")]
                    {
                        let win = web_sys::window().unwrap();
                        physical_size.width = win.inner_width().unwrap().as_f64().unwrap() as u32;
                        physical_size.height = win.inner_height().unwrap().as_f64().unwrap() as u32;
                    };
                    state.resize(physical_size);
                    #[cfg(target_arch = "wasm32")]
                    state.set_scale_factor(state.window.scale_factor());
                },
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    #[cfg(target_arch = "wasm32")]
                    state.set_scale_factor(*scale_factor);
                }
                WindowEvent::RedrawRequested => {
                    state.window().request_redraw();
                    
                    if !surface_configured {
                        return;
                    }

                    state.update();
                    match state.render() {
                        Ok(_) => {},
                        Err(
                            wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                        ) => state.resize(state.size),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            log::error!("OutOfMemory");
                            control_flow.exit();
                        }

                        // This happens when a frame takes too long to present
                        Err(wgpu::SurfaceError::Timeout) => {
                            log::warn!("Surface timeout")
                        }
                    }
                },
                _ => {}
            }
        },
        _ => {}
    }).unwrap();
}

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    scale_factor: f64,
    window: &'a Window,
    render_pipeline: wgpu::RenderPipeline,
    mouse: MouseState,
    keyboard: KeyState,
    camera: Camera,
    last_frame_time: DateTime<Utc>,
    // tilemap: Tilemap,
    background_texture: Texture,
    tilerender_texture: TilerenderTexture,
    world: World,
    next_event: usize,
}

impl<'a> State<'a> {
    const MAX_SIZE: u32 = 8192;
    
    // Creating some of the wgpu types requires async code
    async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch="wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch="wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let mut required_limits = wgpu::Limits::downlevel_webgl2_defaults();
        required_limits.max_texture_dimension_2d = Self::MAX_SIZE;
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits,
                label: None,
                memory_hints: Default::default(),
            },
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::AutoVsync,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };

        let background_texture = Texture::from_bytes(&device, &queue, include_bytes!("backgrounddark.png"), "background").unwrap();
        let camera = Camera::new(&device, &size);
        let tilerender_texture = TilerenderTexture::new(&device);

        let shader = device.create_shader_module(wgpu::include_wgsl!("tile.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &background_texture.bind_group_layout(),
                    &camera.bind_group_layout(),
                    &tilerender_texture.bind_group_layout(),
                ],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            scale_factor: 1.0,
            render_pipeline,
            background_texture,
            camera,
            tilerender_texture,
            mouse: MouseState::new(),
            keyboard: KeyState::new(),
            last_frame_time: Utc::now(),
            world: World::new(),
            next_event: 0,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 && new_size.width < Self::MAX_SIZE && new_size.height < Self::MAX_SIZE {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera.resize(&new_size);
        }
    }
    
    fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
        self.mouse.scale_factor = scale_factor;
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.keyboard.handle(event.clone())
        || self.mouse.handle(event.clone())
    }

    fn update(&mut self) {
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let dt = Utc::now() - self.last_frame_time;
        self.last_frame_time = Utc::now();
        let pan_speed = 200.0 * dt.num_milliseconds() as f32 / 1000.0;
        let zoom_speed = 16.0 * dt.num_milliseconds() as f32 / 1000.0;
        if self.keyboard.key_is_down(PhysicalKey::Code(KeyCode::ArrowRight)) {
            self.camera.pan_pixels(pan_speed, 0.0);
        }
        if self.keyboard.key_is_down(PhysicalKey::Code(KeyCode::ArrowLeft)) {
            self.camera.pan_pixels(-pan_speed, 0.0);
        }
        if self.keyboard.key_is_down(PhysicalKey::Code(KeyCode::ArrowDown)) {
            self.camera.pan_pixels(0.0, pan_speed);
        }
        if self.keyboard.key_is_down(PhysicalKey::Code(KeyCode::ArrowUp)) {
            self.camera.pan_pixels(0.0, -pan_speed);
        }

        if self.keyboard.key_is_down(PhysicalKey::Code(KeyCode::Equal)) {
            self.camera.zoom_level += zoom_speed;
        }
        if self.keyboard.key_is_down(PhysicalKey::Code(KeyCode::Minus)) {
            self.camera.zoom_level -= zoom_speed;
        }

        if self.mouse.button_is_down(MouseButton::Left) {
            self.camera.start_drag(&self.mouse.position);
        } else {
            self.camera.end_drag();
        }
        self.camera.update_drag(&self.mouse);

        let wheel = self.mouse.wheel();
        if let Some(MouseScrollDelta::LineDelta(_x, y)) = wheel {
            self.camera.zoom_around(y, &self.mouse.position);
        }
        if let Some(MouseScrollDelta::PixelDelta(position)) = wheel {
            self.camera.zoom_around((position.y / 100.0) as f32, &self.mouse.position);
        }
        
        self.tilerender_texture.update_view_from_camera(&self.queue, &self.camera, &self.world);

        let clicked = self.mouse.clicked();
        let released = self.mouse.released();

        let position_at_mouse = self.camera.screen_to_world(&self.mouse.position);
        let position = as_world_position(position_at_mouse);
        if released.contains(&MouseButton::Left) {
            if self.mouse.button_is_down(MouseButton::Right) {
                self.world.double_click(position, "bobby");
            } else {
                self.world.click(position, "bobby");
            }
        }
        if clicked.contains(&MouseButton::Right) {
            self.world.flag(position, "bobby");
        }

        let new_events = &self.world.events[self.next_event..];
        self.next_event = self.world.events.len();
        for event in new_events {
            match event {
                Event::Clicked { updated, .. }
                | Event::DoubleClicked { updated, .. }=> {
                    self.tilerender_texture.write_updated_rect(&self.queue, updated);
                }
                Event::Flag { at, .. } => {
                    self.tilerender_texture.write_tile(&self.queue, Tile::empty().with_flag(), at.clone());
                }
                Event::Unflag { at, .. } => {
                    self.tilerender_texture.write_tile(&self.queue, Tile::empty(), at.clone());
                }
            }
        }

        // self.tilerender_texture.render_zoom_texture(&self.device, &self.queue);

        self.camera.write_to_queue(&self.queue, 0);
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    }
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, self.background_texture.bind_group(), &[]);
            render_pass.set_bind_group(1, self.camera.bind_group(), &[]);
            render_pass.set_bind_group(2, self.tilerender_texture.bind_group(), &[]);
            render_pass.draw(0..6, 0..1);

            cfg_if::cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                } else {
                    sleep(TimeDelta::milliseconds(16).to_std().unwrap());
                }
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[derive(Default)]
struct MouseState {
    position: PhysicalPosition<f64>,
    buttons_down: HashSet<MouseButton>,
    buttons_clicked: HashSet<MouseButton>,
    buttons_released: HashSet<MouseButton>,
    delta: Option<MouseScrollDelta>,
    scale_factor: f64,
}

impl MouseState {
    fn new() -> Self {
        Self {
            scale_factor: 1.0,
            ..Default::default()
        }
    }
    
    pub fn handle(&mut self, event: WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let scale = self.scale_factor;
                self.position = PhysicalPosition::new(position.x / scale, position.y / scale);
                true
            }
            WindowEvent::CursorEntered { .. } => false,
            WindowEvent::CursorLeft { .. } => false,
            WindowEvent::MouseWheel { delta, .. } => {
                self.delta = Some(delta);
                true
            }
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    ElementState::Pressed => {
                        self.buttons_down.insert(button);
                        self.buttons_clicked.insert(button)
                    }
                    ElementState::Released => {
                        self.buttons_down.remove(&button);
                        self.buttons_released.insert(button)
                    }
                }
            }
            _ => false
        }
    }
    
    pub fn clicked(&mut self) -> HashSet<MouseButton> {
        let result = self.buttons_clicked.clone();
        self.buttons_clicked.clear();
        result
    }
    
    pub fn released(&mut self) -> HashSet<MouseButton> {
        let result = self.buttons_released.clone();
        self.buttons_released.clear();
        result
    }
    
    pub fn wheel(&mut self) -> Option<MouseScrollDelta> {
        match self.delta {
            None => None,
            Some(delta) => {
                self.delta = None;
                Some(delta)
            }
        }
    }
    
    pub fn button_is_down(&self, button: MouseButton) -> bool {
        self.buttons_down.contains(&button)
    }
}

struct KeyState {
    keys_down: HashSet<PhysicalKey>
}

impl KeyState {
    fn new() -> KeyState {
        Self {
            keys_down: Default::default()
        }
    }
    
    pub fn handle(&mut self, event: WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                match event.state {
                    ElementState::Pressed => {
                        self.keys_down.insert(event.physical_key);
                        false
                    }
                    ElementState::Released => {
                        self.keys_down.remove(&event.physical_key);
                        false
                    }
                }
            }
            WindowEvent::ModifiersChanged(_) => false,
            _ => false
        }
    }
    pub fn key_is_down(&self, key: PhysicalKey) -> bool {
        self.keys_down.contains(&key)
    }
}

fn as_world_position(vector: Vector2<f32>) -> Position {
    Position(vector.x.floor() as i32, vector.y.floor() as i32)
}
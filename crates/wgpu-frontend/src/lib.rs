mod texture;
mod camera;
pub mod tilemap;

use std::collections::{HashSet};
use std::default::Default;
use std::thread::sleep;
use std::time::{Duration, Instant};
use winit::event::{ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowBuilder};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use world::{Chunk, ChunkPosition, ChunkTiles};
use crate::camera::Camera;
use crate::tilemap::Tilemap;
use rand;
use rand::{thread_rng, RngCore};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).unwrap();
        } else {
            env_logger::init();
        }
    }
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::dpi::PhysicalSize;
        let _ = window.request_inner_size(PhysicalSize::new(800, 600));

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

    event_loop.run(move |event, control_flow| match event {
        Event::WindowEvent {
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
                    state.resize(*physical_size);
                },
                WindowEvent::RedrawRequested => {
                    state.window().request_redraw();

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
    window: &'a Window,
    render_pipeline: wgpu::RenderPipeline,
    mouse: MouseState,
    keyboard: KeyState,
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: texture::Texture,
    camera: Camera,
    last_frame_time: Instant,
    tilemap: Tilemap,
}

impl<'a> State<'a> {
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
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::BUFFER_BINDING_ARRAY | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY,
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
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
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let diffuse_bytes = include_bytes!("tiles.png");
        let diffuse_texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "tiles.png").unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float {filterable: true},
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    },
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        let camera = Camera::new(&device, &size);
        let tilemap = Tilemap::new(&device);
        let shader = device.create_shader_module(wgpu::include_wgsl!("tile.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera.bind_group_layout,
                    &tilemap.bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
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

        for x in -10..10 {
            for y in -10..10 {
                let mut chunk = Chunk::generate(ChunkPosition::new(x * 16, y * 16), 40, 0);
                let mut bytes: [u8; 256] = [0; 256];
                thread_rng().fill_bytes(&mut bytes);
                chunk.tiles = ChunkTiles::from(bytes);
                tilemap.update_chunk(&chunk, &queue);
            }
        }

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            diffuse_bind_group,
            diffuse_texture,
            camera,
            tilemap,
            mouse: MouseState::new(),
            keyboard: KeyState::new(),
            last_frame_time: Instant::now()
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera.resize(&new_size);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.keyboard.handle(event.clone())
        || self.mouse.handle(event.clone())
    }

    fn update(&mut self) {
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let dt = self.last_frame_time.elapsed();
        self.last_frame_time = Instant::now();
        let pan_speed = 200.0 * dt.as_secs_f32();
        let zoom_speed = 16.0 * dt.as_secs_f32();
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
        }
        if !self.mouse.button_is_down(MouseButton::Left) {
            self.camera.end_drag();
        }
        self.camera.update_drag(&self.mouse);
        if let Some(MouseScrollDelta::LineDelta(_x, y)) = self.mouse.wheel() {
            self.camera.zoom_around(y, &self.mouse.position);
        }

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
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.mouse.position.x / self.size.width as f64,
                            g: self.mouse.position.y / self.size.height as f64,
                            b: 0.3,
                            a: 1.0
                        }),
                        store: wgpu::StoreOp::Store,
                    }
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera.bind_group, &[]);
            render_pass.set_bind_group(2, &self.tilemap.bind_group, &[]);
            render_pass.draw(0..3, 0..1);

            sleep(Duration::from_millis(16));
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
    delta: Option<MouseScrollDelta>,
}

impl MouseState {
    fn new() -> Self {
        Self::default()
    }
    
    pub fn handle(&mut self, event: WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.position = position;
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
                        self.buttons_down.insert(button)
                    }
                    ElementState::Released => {
                        self.buttons_down.remove(&button)
                    }
                }
            }
            _ => false
        }
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
                        self.keys_down.insert(event.physical_key)
                    }
                    ElementState::Released => {
                        self.keys_down.remove(&event.physical_key)
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
mod texture;
mod camera;
mod tilerender_texture;
mod shader;
mod tile_sprites;
mod sweeper_socket;
mod cursors;

use std::collections::{HashSet};
use std::default::Default;
use cgmath::Vector2;
use chrono::prelude::*;
use log::info;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowBuilder};
use wgpu::{CompositeAlphaMode, PresentMode, ShaderSource};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use world::{Position, Tile};
use world::events::Event;
use crate::camera::Camera;
use crate::shader::HasBindGroup;
use crate::sweeper_socket::{SweeperSocket};
use crate::tilerender_texture::{TileMapTexture};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use world::client_messages::ClientMessage;
use world::server_messages::ServerMessage;
use crate::cursors::Cursors;

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
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(1280, 720))
        .build(&event_loop).unwrap();

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
                WindowEvent::Resized(..) => {
                    surface_configured = true;
                    cfg_if::cfg_if! {
                        if #[cfg(target_arch = "wasm32")] {
                            let win = web_sys::window().unwrap();
                            let width = win.inner_width().unwrap().as_f64().unwrap() as u32;
                            let height = win.inner_height().unwrap().as_f64().unwrap() as u32;
                            state.set_scale_factor(state.window.scale_factor());
                            state.resize(PhysicalSize::new(width, height));
                        } else {
                            state.resize(state.window.inner_size());
                        }
                    }
                },
                #[cfg(target_arch = "wasm32")]
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    state.set_scale_factor(*scale_factor);
                }
                WindowEvent::RedrawRequested => {
                    state.window().request_redraw();
                    
                    if !surface_configured {
                        return;
                    }

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
    tile_map_texture: TileMapTexture,
    world: Box<dyn SweeperSocket>,
    cursors: Cursors
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

        let camera = Camera::new(&device, &size);

        let tile_map_texture = TileMapTexture::new(&device, &queue, &camera);

        let common_shader = include_str!("common.wgsl");
        let mut wgsl_source = String::from(common_shader);
        wgsl_source.push_str(include_str!("tile.wgsl"));
        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: None,
                source: ShaderSource::Wgsl(wgsl_source.into()),
            }
        );
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera.bind_group_layout(),
                    &tile_map_texture.bind_group_layout(),
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

        let world;
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                use crate::sweeper_socket::socketio::IoWorld;
                world = Box::new(IoWorld::new("/"))
            } else {
                use crate::sweeper_socket::local::LocalWorld;
                world = Box::new(LocalWorld::new())
            }
        }
        let cursors = Cursors::new(&device, &queue, surface_format, &camera);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            scale_factor: 1.0,
            render_pipeline,
            camera,
            mouse: MouseState::new(),
            keyboard: KeyState::new(),
            last_frame_time: Utc::now(),
            world,
            tile_map_texture,
            cursors,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        info!("rendering at {} x {}, scaling is {}%", new_size.width, new_size.height, self.scale_factor*100.0);
        if new_size.width > 0 && new_size.height > 0 && new_size.width < Self::MAX_SIZE && new_size.height < Self::MAX_SIZE {
            self.size = PhysicalSize::new(
                (new_size.width as f64 * self.scale_factor) as u32,
                (new_size.height as f64 * self.scale_factor) as u32,
            );
            self.config.width = self.size.width;
            self.config.height = self.size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera.resize(&self.size, self.scale_factor);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
        self.mouse.scale_factor = scale_factor;
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.keyboard.handle(event.clone())
        || self.mouse.handle(event.clone())
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
        
        let pressed = self.keyboard.pressed();
        if pressed.contains(&PhysicalKey::Code(KeyCode::KeyL)) {
            self.tile_map_texture.sprites.toggle_dark_mode(&self.queue);
        }
        if pressed.contains(&PhysicalKey::Code(KeyCode::KeyF)) {
            self.tile_map_texture.sprites.change_filter(&self.queue);
        }

        if self.mouse.button_is_down(MouseButton::Left) {
            self.camera.start_drag(&self.mouse.position);
        }
        self.camera.update_drag(&self.mouse);

        let wheel = self.mouse.wheel();
        if let Some(MouseScrollDelta::LineDelta(_x, y)) = wheel {
            self.camera.zoom_around(y, &self.mouse.position);
        }
        if let Some(MouseScrollDelta::PixelDelta(position)) = wheel {
            self.camera.zoom_around((position.y / 100.0) as f32, &self.mouse.position);
        }
        
        self.camera.write_to_queue(&self.queue, 0);
        
        // Load in any new chunks
        let rects = self.tile_map_texture.update_draw_area(&self.camera);
        for rect in rects {
            if rect.area() == 0 { continue }
            self.tile_map_texture.blank_rect(&self.device, &self.queue, &self.camera, rect.clone());
            let chunks = self.world.get_chunks(rect);
            for chunk in chunks {
                self.tile_map_texture.write_chunk(&self.queue, chunk);
            }
        }
        
        let clicked = self.mouse.clicked();
        let released = self.mouse.released();

        let position_at_mouse = self.camera.screen_to_world(&self.mouse.position);
        let position = as_world_position(position_at_mouse);
        if released.contains(&MouseButton::Left) {
            let drag_length = self.camera.end_drag(&self.mouse.position);
            if drag_length < 4.0 {
                if self.mouse.button_is_down(MouseButton::Right) {
                    self.world.send(ClientMessage::DoubleClick(position));
                } else {
                    self.world.send(ClientMessage::Click(position));
                }
            }
        }
        if clicked.contains(&MouseButton::Right) {
            let tile = self.world.world().get_tile(&position);
            if tile.is_flag() {
                info!("unflagging");
                self.tile_map_texture.write_tile(&self.queue, tile.without_flag(), position);
            } else {
                info!("flagging");
                self.tile_map_texture.write_tile(&self.queue, tile.with_flag(), position);
            }
            self.world.send(ClientMessage::Flag(position));
        }

        while let Some(message) = self.world.next_message() {
            self.world.world().apply_server_message(&message);
            match message {
                ServerMessage::Event(event) => {
                    let player_id = self.world.world().create_or_update_player(&event);
                    if let Some(player) = self.world.world().players.get(&player_id) {
                        self.cursors.update_player(player, &self.queue);
                    }
                    match event {
                        Event::Clicked { updated, .. }
                        | Event::DoubleClicked { updated, .. } => {
                            self.tile_map_texture.write_updated_rect(&self.queue, &updated);
                        }
                        Event::Flag { at, .. } => {
                            self.tile_map_texture.write_tile(&self.queue, Tile::empty().with_flag(), at.clone());
                        }
                        Event::Unflag { at, .. } => {
                            self.tile_map_texture.write_tile(&self.queue, Tile::empty(), at.clone());
                        }
                    }
                }
                ServerMessage::Chunk(chunk) => {
                    self.tile_map_texture.write_chunk(&self.queue, &chunk);
                    self.world.world().insert_chunk(chunk);
                }
                ServerMessage::Player(player) => {
                    self.cursors.update_player(&player, &self.queue);
                    self.world.world().players.insert(player.player_id.clone(), player);
                }
                ServerMessage::Welcome(player) => {
                    self.cursors.set_you(player.player_id.clone(), &player, &self.queue);
                }
                ServerMessage::Disconnected(player_id) => {
                    self.cursors.delete_player(&player_id, &self.queue);
                    self.world.world().players.remove(&player_id);
                }
            }
        }

        self.tile_map_texture.render(&self.camera, &self.device, &self.queue);

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        if true {
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
                render_pass.set_bind_group(0, self.camera.bind_group(), &[]);
                render_pass.set_bind_group(1, self.tile_map_texture.bind_group(), &[]);
                render_pass.draw(0..6, 0..1);

                cfg_if::cfg_if! {
                    if #[cfg(target_arch = "wasm32")] {
                    } else {
                        use std::thread::sleep;
                        use chrono::TimeDelta;
                        sleep(TimeDelta::milliseconds(16).to_std().unwrap());
                    }
                }
            }
            self.queue.submit(std::iter::once(encoder.finish()));
        }

        self.cursors.render(&self.device, &self.queue, &view, &self.camera);
        
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

#[derive(Default)]
struct KeyState {
    keys_down: HashSet<PhysicalKey>,
    keys_pressed: HashSet<PhysicalKey>,
}

impl KeyState {
    fn new() -> KeyState {
        Self {
            ..Default::default()
        }
    }
    
    pub fn handle(&mut self, event: WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                match event.state {
                    ElementState::Pressed => {
                        self.keys_down.insert(event.physical_key);
                        self.keys_pressed.insert(event.physical_key);
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
    
    pub fn pressed(&mut self) -> HashSet<PhysicalKey> {
        let result = self.keys_pressed.clone();
        self.keys_pressed.clear();
        result
    }
    
}

fn as_world_position(vector: Vector2<f32>) -> Position {
    Position(vector.x.floor() as i32, vector.y.floor() as i32)
}
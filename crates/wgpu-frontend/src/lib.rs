mod texture;
pub mod camera;
mod tilerender_texture;
mod shader;
mod tile_sprites;
mod sweeper_socket;
mod cursors;
mod fingers;

use std::default::Default;
use std::future::Future;
use std::sync::Arc;
use cgmath::Vector2;
use chrono::prelude::*;
use log::info;
use winit::event::{ButtonSource, ElementState, KeyEvent, MouseButton, MouseScrollDelta, PointerSource, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key};
use winit::window::{Window, WindowAttributes, WindowId};
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
use winit::application::ApplicationHandler;
use world::client_messages::ClientMessage;
use world::server_messages::ServerMessage;
use crate::cursors::Cursors;
use crate::fingers::Fingers;

#[derive(Default)]
struct App {
    window: Option<Arc<Box<dyn Window>>>,
    state: Option<State>,
    receiver: Option<futures::channel::oneshot::Receiver<State>>,
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if let Ok(window) = event_loop.create_window(
            WindowAttributes::default()
                .with_surface_size(PhysicalSize::new(1280, 720))
        ) {
            let window_handle = Arc::new(window);
            self.window = Some(window_handle.clone());
            let (sender, receiver) = futures::channel::oneshot::channel();
            self.receiver = Some(receiver);
            
            #[cfg(target_arch = "wasm32")]
            wasm_bindgen_futures::spawn_local(async move {
                let state = State::new(window_handle.clone()).await;
                let _ = sender.send(state);
            });
            
            #[cfg(not(target_arch="wasm32"))]
            {
                let state = pollster::block_on(State::new(window_handle.clone()));
                let _ = sender.send(state);
            }
        }
    }

    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        if self.state.is_none() {
            if let Some(receiver) = &mut self.receiver {
                if let Ok(Some(mut state)) = receiver.try_recv() {
                    state.resize();
                    info!("resized");
                    self.state = Some(state);
                }
            }
        }
        if let Some(state) = &mut self.state {
            state.handle_window_event(event_loop, event);
        }
    }
}

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
    let app = App::default();
    event_loop.run_app(app).unwrap();
}

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    scale_factor: f64,
    window: Arc<Box<dyn Window>>,
    render_pipeline: wgpu::RenderPipeline,
    camera: Camera,
    last_frame_time: DateTime<Utc>,
    tile_map_texture: TileMapTexture,
    world: Box<dyn SweeperSocket>,
    cursors: Cursors,
    surface_configured: bool,
    right_mouse_button_down: bool,
    fingers: Fingers,
}

impl State {
    const MAX_SIZE: u32 = TileMapTexture::SIZE as u32;

    // Creating some of the wgpu types requires async code
    fn new(window: Arc<Box<dyn Window>>) -> impl Future<Output = Self> + 'static {
        let size = window.surface_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch="wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch="wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).unwrap();
        async move {
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

            #[cfg(target_arch = "wasm32")]
            {
                let _ = window.request_surface_size(PhysicalSize::new(1280, 720).into());

                use winit::platform::web::WindowExtWeb;
                web_sys::window()
                    .and_then(|win| win.document())
                    .and_then(|doc| {
                        let dst = doc.get_element_by_id("wasm-example").unwrap();
                        let canvas = web_sys::Element::from(window.canvas().unwrap().clone());
                        dst.append_child(&canvas).unwrap();
                        Some(())
                    })
                    .expect("Couldn't append canvas to document body.")
            }

            let view_matrix = camera.view_matrix();

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
                last_frame_time: Utc::now(),
                world,
                tile_map_texture,
                cursors,
                surface_configured: false,
                right_mouse_button_down: false,
                fingers: Fingers::new(view_matrix),
            }
        }
    }

    fn resize(&mut self) {
        #[cfg(target_arch = "wasm32")]
        let new_size = {
            let win = web_sys::window().unwrap();
            let width = win.inner_width().unwrap().as_f64().unwrap() as u32;
            let height = win.inner_height().unwrap().as_f64().unwrap() as u32;
            PhysicalSize::new(width, height)
        };

        #[cfg(not(target_arch="wasm32"))]
        let new_size = self.size;
        
        self.scale_factor = self.window.scale_factor();
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
            self.surface_configured = true;
        }
    }

    fn handle_window_event(&mut self, event_loop: &dyn ActiveEventLoop, event: WindowEvent) {
        let window_size = PhysicalSize::new(self.size.width as f64, self.size.height as f64);
        let new_view_matrix = self.fingers.handle_event(&event, window_size, self.camera.view_matrix());
        self.camera.set_view_matrix(new_view_matrix);
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::SurfaceResized(size) => {
                self.size = size;
                self.resize();
            },
            #[cfg(target_arch = "wasm32")]
            WindowEvent::ScaleFactorChanged { .. } => {
                self.resize();
            }
            WindowEvent::RedrawRequested => {
                self.window.request_redraw();

                if !self.surface_configured {
                    return;
                }

                match self.render() {
                    Ok(_) => {},
                    Err(_) => self.resize(),
                }
            }
            WindowEvent::PointerButton {
                state: ElementState::Pressed,
                button: ButtonSource::Mouse(MouseButton::Left),
                position,
                ..
            } => {
                self.camera.start_drag(&position);
            }
            WindowEvent::PointerButton {
                state: ElementState::Released,
                button: ButtonSource::Mouse(MouseButton::Left),
                position,
                ..
            } => {
                let drag_distance = self.camera.end_drag(&position);
                if drag_distance < 4.0 {
                    if self.right_mouse_button_down {
                        self.double_click_at(&position);
                    } else {
                        self.click_at(&position);
                    }
                }
            }
            WindowEvent::PointerMoved {
                position,
                source: PointerSource::Mouse,
                ..
            } => {
                self.camera.update_mouse_position(&position);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(_x, y) => {
                        self.camera.zoom_around_mouse_position(y);
                    }
                    MouseScrollDelta::PixelDelta(position) => {
                        self.camera.zoom_around_mouse_position((position.y / 100.0) as f32);
                    }
                }
            }
            WindowEvent::PointerButton {
                state: ElementState::Pressed,
                button: ButtonSource::Mouse(MouseButton::Right),
                position,
                ..
            } => {
                self.right_mouse_button_down = true;
                self.toggle_flag_at(&position);
            }
            WindowEvent::PointerButton {
                state: ElementState::Released,
                button: ButtonSource::Mouse(MouseButton::Right),
                ..
            } => {
                self.right_mouse_button_down = false;
            }
            WindowEvent::KeyboardInput { event: KeyEvent {
                state: ElementState::Pressed,
                logical_key,
                ..
            }, .. } => {
                match logical_key {
                    Key::Named(_) => {}
                    Key::Character(character) => {
                        if character == "l" {
                            self.toggle_dark_mode();
                        }
                    }
                    Key::Unidentified(_) => {}
                    Key::Dead(_) => {}
                }
            }
            _ => {}
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let _dt = Utc::now() - self.last_frame_time;
        self.last_frame_time = Utc::now();

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

        self.cursors.render(&self.device, &self.queue, &view, &self.camera);

        output.present();

        Ok(())
    }

    pub fn click_at(&mut self, mouse_position: &PhysicalPosition<f64>) {
        let position_at_mouse = self.camera.screen_to_world(mouse_position);
        let position = as_world_position(position_at_mouse);

        if !self.world.world().get_tile(&position).is_revealed() {
            self.tile_map_texture.write_tile(&self.queue, Tile::empty().with_revealed(), position);
            self.world.send(ClientMessage::Click(position));
        }
    }

    pub fn double_click_at(&mut self, mouse_position: &PhysicalPosition<f64>) {
        let position_at_mouse = self.camera.screen_to_world(mouse_position);
        let position = as_world_position(position_at_mouse);

        if let Some(to_reveal) = self.world.world().check_double_click(&position) {
            for position_to_reveal in &to_reveal {
                self.tile_map_texture.write_tile(&self.queue, Tile::empty().with_revealed(), *position_to_reveal);
            }
            if to_reveal.len() > 0 {
                self.world.send(ClientMessage::DoubleClick(position));
            }
        }
    }

    pub fn toggle_flag_at(&mut self, mouse_position: &PhysicalPosition<f64>) {
        let position_at_mouse = self.camera.screen_to_world(mouse_position);
        let position = as_world_position(position_at_mouse);

        // TODO: On local, you can't unflag
        let tile = self.world.world().get_tile(&position);
        if tile.is_flag() {
            info!("unflagging");
            self.tile_map_texture.write_tile(&self.queue, tile.without_flag(), position);
        } else {
            info!("flagging");
            self.tile_map_texture.write_tile(&self.queue, tile.with_flag(), position);
        }
        self.world.world().flag(position, "");
        self.world.send(ClientMessage::Flag(position));
    }

    pub fn toggle_dark_mode(&mut self) {
        self.tile_map_texture.sprites.toggle_dark_mode(&self.queue);
    }
}

fn as_world_position(vector: Vector2<f32>) -> Position {
    Position(vector.x.floor() as i32, vector.y.floor() as i32)
}
use std::collections::{HashMap, VecDeque};
use std::io::Read;
use std::num::NonZeroU32;
use std::sync::Arc;

use app::MainApp;

pub use config::Config;
use image::{EncodableLayout, RgbaImage};
use smithay_client_toolkit::shell::wlr_layer::{Anchor, Layer};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, MouseScrollDelta, TouchPhase};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::platform::wayland::{ActiveEventLoopExtWayland, MonitorHandleExtWayland};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    monitor::VideoModeHandle,
    platform::wayland::WindowAttributesExtWayland,
    window::{CursorIcon, Window, WindowAttributes, WindowId},
};

mod app;
mod config;
mod shot;
mod utils;

fn create_window(
    ev: &dyn ActiveEventLoop,
    w: WindowAttributes,
    scale_factor: f64,
    monitor_mode: VideoModeHandle,
) -> WindowState {
    let size = monitor_mode.size().to_logical::<u32>(scale_factor);
    let w = w.with_surface_size(size);
    let w = if ev.is_wayland() {
        w.with_anchor(Anchor::all())
            .with_layer(Layer::Overlay)
            .with_margin(0 as i32, 0 as i32, 0, 0 as i32)
            // .with_region((0, 0).into(), size)
            .with_output(monitor_mode.monitor().native_id())
    } else {
        w.with_position(LogicalPosition::new(0, 0))
            .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
            .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
    };
    WindowState::new(
        ev.create_window(w.with_cursor(CursorIcon::Crosshair))
            .unwrap(),
    )
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.run_app(&mut WindowApp::default()).unwrap();
}

#[derive(Default)]
struct WindowApp {
    windows: HashMap<WindowId, WindowState>,
    messages: VecDeque<app::MainMessage>,
}

impl ApplicationHandler for WindowApp {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let window_attributes = WindowAttributes::default().with_decorations(false);
        // .with_transparent(true)

        if let Some((i, screen)) = event_loop
            .available_monitors()
            .into_iter()
            .enumerate()
            .next()
        {
            let Some(mode) = screen.current_video_mode() else {
                return;
            };
            let window_attributes = window_attributes.clone();
            let window_state = create_window(
                event_loop,
                window_attributes.with_title(format!(
                    "__zoomer_{}",
                    screen.name().unwrap_or(i.to_string())
                )),
                screen.scale_factor(),
                mode,
            );
            let window_id = window_state.window.id();
            println!("Created new window with id={window_id:?}");
            self.windows.insert(window_id, window_state);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if !self.windows.contains_key(&window_id) {
            return;
        }
        let state = self.windows.get_mut(&window_id).unwrap();

        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::AltLeft | KeyCode::AltRight),
                        state,
                        ..
                    },
                ..
            } => self
                .messages
                .push_back(app::MainMessage::AltKey(state.is_pressed())),
            WindowEvent::PointerMoved {
                position: PhysicalPosition { x, y },
                ..
            } => {
                self.messages
                    .push_back(app::MainMessage::Move(x as i32, y as i32));
                // state.window.request_redraw();
            }
            WindowEvent::SurfaceResized(PhysicalSize { width, height }) => {
                state.resize(width, height);
                self.messages
                    .push_back(app::MainMessage::Resize((width, height)));
                // state.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                // send messages
                while let Some(msg) = self.messages.pop_front().as_ref() {
                    process_cmd(state.window.clone(), &state.app.update(msg));
                }
                let PhysicalSize { width, .. } = state.window.surface_size();
                // Render
                if let Some(img) = state.app.render() {
                    state.draw(width, img);
                }
                state.window.request_redraw();
                // close_requested = true;
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, y),
                phase: TouchPhase::Moved,
                ..
            } => {
                if y < 0. {
                    self.messages.push_back(app::MainMessage::ZoomIn);
                } else {
                    self.messages.push_back(app::MainMessage::ZoomOut);
                }
                // state.window.request_redraw();
            }
            _ => (),
        }
    }
}

fn process_cmd(w: Arc<dyn Window>, cmd: &app::Command) {
    match cmd {
        app::Command::Resize(width, height) => {
            w.set_min_surface_size(Some(LogicalSize::new(*width, *height).into()))
        }
        _ => {}
    }
}

struct WindowState {
    app: MainApp,
    pub surface: softbuffer::Surface<Arc<dyn Window>, Arc<dyn Window>>,
    /// The actual winit Window.
    pub window: Arc<dyn Window>,
}

impl WindowState {
    fn new(window: Box<dyn Window>) -> Self {
        let settings = config::get_config();
        let window: Arc<dyn Window> = Arc::from(window);
        let app = MainApp::new(settings, window.current_monitor().unwrap().name());
        let context = softbuffer::Context::new(window.clone()).unwrap();
        let surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

        Self {
            app,
            surface,
            window,
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.surface
            .resize(
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            )
            .unwrap();
    }

    fn draw(&mut self, width: u32, img: RgbaImage) {
        self.window.pre_present_notify();
        let mut buffer = self.surface.buffer_mut().unwrap();

        for (x, y, p) in img.enumerate_pixels() {
            let [r, g, b, a] = p.0;
            buffer[y as usize * width as usize + x as usize] =
                (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | b as u32;
        }
        // img.save("out.png").unwrap();

        // img.save(format!("{}.png", self.window.title())).unwrap();
        buffer.present().unwrap();
    }
}

use std::collections::VecDeque;
use std::num::NonZeroU32;
use std::rc::Rc;

use app::MainApp;
use mouse_position::{Mouse, MouseExt};

// use app::MainApp;
pub use config::Config;
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
use winit::event::{ElementState, KeyEvent, MouseScrollDelta, TouchPhase};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

mod app;
mod config;

fn create_window((x, y): (i32, i32), (w, h): (u32, u32), ev: &ActiveEventLoop) -> Window {
    let attrs = Window::default_attributes()
        .with_title("Zoomer")
        .with_resizable(false)
        .with_decorations(false)
        .with_active(true)
        .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
        .with_inner_size(LogicalSize::new(w, h))
        .with_position(LogicalPosition::new(x, y));
    ev.create_window(attrs).unwrap()
}

// fn main() -> iced::Result {
fn main() -> Result<(), winit::error::EventLoopError> {
    let settings = config::get_config();
    let (w, h) = (
        settings.width.unwrap_or(400),
        settings.height.unwrap_or(200),
    );
    let mut mouse = Mouse::default();
    let mut messages = VecDeque::new();
    let mut app = MainApp::new(settings);

    let event_loop = EventLoop::new().unwrap();
    let mut close_requested = false;

    let mut window: Option<Rc<Window>> = None;
    let mut context = None;
    let mut surface = None;

    event_loop.run(move |event, event_loop| {
        let (x, y) = mouse.get_pos().unwrap();
        messages.push_back(app::MainMessage::Move(x, y));
        match event {
            Event::Resumed => {
                window = Some(Rc::new(create_window(
                    mouse.get_pos().unwrap(),
                    (w, h),
                    &event_loop,
                )));
                ()
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => close_requested = true,
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: winit::keyboard::PhysicalKey::Code(KeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => close_requested = true,
                WindowEvent::CursorMoved { .. } => {
                    if let Some(window) = window.as_ref() {
                        messages.push_back(app::MainMessage::Move(x, y));
                        window.set_outer_position(LogicalPosition::new(x, y));
                    }
                }
                WindowEvent::RedrawRequested => {
                    if let Some(window) = window.clone() {
                        let context = context
                            .get_or_insert(softbuffer::Context::new(window.clone()).unwrap());
                        let surface = surface.get_or_insert(
                            softbuffer::Surface::new(&context, window.clone()).unwrap(),
                        );
                        let PhysicalSize { width, height } = window.inner_size();
                        surface
                            .resize(
                                NonZeroU32::new(width).unwrap(),
                                NonZeroU32::new(height).unwrap(),
                            )
                            .unwrap();
                        // send messages
                        while let Some(msg) = messages.pop_front().as_ref() {
                            app.update(msg);
                        }
                        // Render
                        println!("Pre render");
                        if let Some(img) = app.render() {
                            let mut buffer = surface.buffer_mut().unwrap();

                            for y in 0..height {
                                for x in 0..width {
                                    let index = y * width + x;
                                    let [r, g, b, _] = img.get_pixel(x, y).0;
                                    buffer[index as usize] =
                                        (b as u32) | ((g as u32) << 8) | ((r as u32) << 16);
                                }
                            }
                            println!("render");
                            buffer.present().unwrap();
                        }
                    }
                }
                // WindowEvent::KeyboardInput { event, .. } => (),
                // WindowEvent::ModifiersChanged(_) => (),
                WindowEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(_, y),
                    phase: TouchPhase::Moved,
                    ..
                } => {
                    println!("Wheel: {y:?}");
                    if y < 0. {
                        messages.push_back(app::MainMessage::ZoomOut((y * 1000.) as i32));
                    } else {
                        messages.push_back(app::MainMessage::ZoomIn((y * 1000.) as i32));
                    }
                }
                // WindowEvent::TouchpadMagnify { delta, phase, .. } => {
                //     println!("Touch: {delta:?} - {phase:?}");
                // }
                _ => (),
            },
            Event::AboutToWait => {
                if close_requested {
                    event_loop.exit();
                } else {
                    if let Some(window) = window.as_ref() {
                        window.request_redraw();
                    }
                }
            }
            _ => (),
        }
    })
}

use std::collections::VecDeque;
use std::num::NonZeroU32;
use std::rc::Rc;

use app::MainApp;
use mouse_position::{Mouse, MouseExt};

pub use config::Config;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, KeyEvent, MouseScrollDelta, TouchPhase};
use winit::event_loop::EventLoopWindowTarget;
// use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

mod app;
mod config;
mod shot;
mod utils;

// fn create_window((x, y): (i32, i32), (w, h): (u32, u32), ev: &ActiveEventLoop) -> Window {
fn create_window(ev: &EventLoopWindowTarget<()>) -> Window {
    let w = winit::window::WindowBuilder::new()
        // let attrs = Window::default_attributes()
        .with_title("Zoomer")
        .with_active(true)
        .with_resizable(false)
        .with_decorations(false)
        .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
        // .with_cursor(winit::window::Cursor::Icon(
        //     winit::window::CursorIcon::Crosshair,
        // ))
        .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
        .build(ev)
        .unwrap();
    w.set_cursor_icon(winit::window::CursorIcon::Crosshair);
    w
    // ev.create_window(attrs).unwrap()
}

fn main() -> Result<(), winit::error::EventLoopError> {
    let settings = config::get_config();
    let mut mouse = Mouse::default();
    let mut messages = VecDeque::new();
    let mut app = MainApp::new(settings, mouse.get_pos().unwrap());

    let event_loop = EventLoop::new().unwrap();
    let mut close_requested = false;

    let mut window: Option<Rc<Window>> = None;
    let mut context = None;
    let mut surface = None;

    event_loop.run(move |event, event_loop| {
        // let (x, y) = mouse.get_pos().unwrap();
        // messages.push_back(app::MainMessage::Move(x, y));
        match event {
            Event::Resumed => {
                window = Some(Rc::new(create_window(&event_loop)));
                ()
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CursorLeft { .. }
                | WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => close_requested = true,
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(KeyCode::AltLeft | KeyCode::AltRight),
                            state,
                            ..
                        },
                    ..
                } => messages.push_back(app::MainMessage::AltKey(state.is_pressed())),
                WindowEvent::CursorMoved {
                    position: PhysicalPosition { x, y },
                    ..
                } => {
                    // if let Some(window) = window.as_ref() {
                    messages.push_back(app::MainMessage::Move(x as i32, y as i32));
                    // window.set_outer_position(LogicalPosition::new(x, y));
                    // }
                }
                WindowEvent::Resized(PhysicalSize { width, height }) => {
                    if let Some(window) = window.clone() {
                        let context = context
                            .get_or_insert(softbuffer::Context::new(window.clone()).unwrap());
                        let surface = surface.get_or_insert(
                            softbuffer::Surface::new(&context, window.clone()).unwrap(),
                        );
                        surface
                            .resize(
                                NonZeroU32::new(width).unwrap(),
                                NonZeroU32::new(height).unwrap(),
                            )
                            .unwrap();
                        messages.push_back(app::MainMessage::Resize((width, height)));
                    }
                }
                WindowEvent::RedrawRequested => {
                    if let Some(window) = window.clone() {
                        // send messages
                        while let Some(msg) = messages.pop_front().as_ref() {
                            process_cmd(&window, &app.update(msg));
                        }
                        let context = context
                            .get_or_insert(softbuffer::Context::new(window.clone()).unwrap());
                        let surface = surface.get_or_insert(
                            softbuffer::Surface::new(&context, window.clone()).unwrap(),
                        );
                        let PhysicalSize { width, .. } = window.inner_size();
                        // Render
                        if let Some(img) = app.render() {
                            window.pre_present_notify();
                            let mut buffer = surface.buffer_mut().unwrap();

                            for (x, y, p) in img.enumerate_pixels() {
                                let [r, g, b, a] = p.0;
                                buffer[y as usize * width as usize + x as usize] = (a as u32) << 24
                                    | (r as u32) << 16
                                    | (g as u32) << 8
                                    | b as u32;
                            }
                            // img.save("out.png").unwrap();
                            buffer.present().unwrap();
                        }
                        // close_requested = true;
                    }
                }
                WindowEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(_, y),
                    phase: TouchPhase::Moved,
                    ..
                } => {
                    if y < 0. {
                        messages.push_back(app::MainMessage::ZoomIn);
                    } else {
                        messages.push_back(app::MainMessage::ZoomOut);
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

fn process_cmd(w: &Window, cmd: &app::Command) {
    match cmd {
        app::Command::Resize(width, height) => {
            w.set_min_inner_size(Some(LogicalSize::new(*width, *height)))
        }
        _ => {}
    }
}

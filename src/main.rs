use iced::{Application, Point, Settings};
use mouse_position::{Mouse, MouseExt};

use app::MainApp;
pub use config::Config;

mod app;
mod config;

fn main() -> iced::Result {
    let settings = config::get_config();
    let (x, y) = Mouse::default().get_pos().unwrap();

    MainApp::run(Settings {
        window: iced::window::Settings {
            position: iced::window::Position::Specific(Point {
                x: x as f32,
                y: y as f32,
            }),
            size: iced::Size::new(400., 200.),
            visible: true,
            resizable: true,
            decorations: false,
            level: iced::window::Level::AlwaysOnTop,
            ..Default::default()
        },
        flags: settings,
        ..Default::default()
    })
}

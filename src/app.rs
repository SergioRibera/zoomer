use iced::{Application, Command};

use crate::Config;

pub struct MainApp {
    config: Config,
}

#[derive(Debug)]
struct Window;

#[derive(Debug, Clone)]
pub enum MainMessage {
    ZoomIn,
    ZoomOut,
}

impl Application for MainApp {
    type Executor = iced::executor::Default;
    type Message = MainMessage;
    type Theme = iced::Theme;
    type Flags = Config;

    fn new(config: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self { config }, Command::none())
    }

    fn title(&self) -> String {
        "Zoomer".to_string()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        iced::widget::Column::new().into()
    }
}

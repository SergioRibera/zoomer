use image::{Rgba, RgbaImage};

use crate::shot::{capture, generate_border, round_image};
use crate::utils::str_to_color;
use crate::Config;

pub struct MainApp {
    alt: bool,
    config: Config,
    pos: (i32, i32),
    scale_zoom: i32,
    area_scale: i32,
    area_size: (u32, u32),
    origin_area_size: (u32, u32),
    border_color: Option<Rgba<u8>>,
    img_origin: RgbaImage,
    img: RgbaImage,
}

#[derive(Debug, Clone)]
pub enum MainMessage {
    Move(i32, i32),
    Resize((u32, u32)),
    ZoomIn,
    ZoomOut,
    AltKey(bool),
}

#[derive(Debug, Clone)]
pub enum Command {
    None,
    Resize(i32, i32),
}

impl MainApp {
    pub fn new(config: Config, (x, y): (i32, i32)) -> Self {
        #[cfg(feature = "wayland")]
        let wayshot = libwayshot::WayshotConnection::new().unwrap();
        let border_color = config.border_color.as_deref().and_then(str_to_color);

        let img = capture(
            #[cfg(feature = "wayland")]
            &wayshot,
            (x, y),
        );
        let origin_area_size = (config.width.unwrap_or(400), config.height.unwrap_or(200));
        Self {
            img_origin: img.clone(),
            img,
            config,
            border_color,
            alt: false,
            pos: (0, 0),
            scale_zoom: 0,
            area_scale: 0,
            area_size: origin_area_size,
            origin_area_size,
        }
    }

    pub fn update(&mut self, msg: &MainMessage) -> Command {
        match msg {
            MainMessage::Move(x, y) => self.pos = (*x, *y),
            MainMessage::Resize(size) => {
                self.img =
                    image::imageops::crop_imm(&self.img_origin, 0, 0, size.0, size.1).to_image();
            }
            MainMessage::AltKey(pressed) => self.alt = *pressed,
            MainMessage::ZoomIn => {
                if self.alt {
                    self.area_scale += 1;
                    self.area_size = (
                        (self.origin_area_size.0 as i32 + self.area_scale) as u32,
                        (self.origin_area_size.1 as i32 + self.area_scale) as u32,
                    );
                } else if self.scale_zoom < 20 {
                    self.scale_zoom += 1;
                }
            }
            MainMessage::ZoomOut => {
                if self.alt {
                    self.area_scale -= 1;
                    self.area_size = (
                        (self.origin_area_size.0 as i32 + self.area_scale) as u32,
                        (self.origin_area_size.1 as i32 + self.area_scale) as u32,
                    );
                } else if self.scale_zoom > -20 {
                    self.scale_zoom -= 1;
                }
            }
        }
        Command::None
    }

    pub fn render(&self) -> Option<RgbaImage> {
        let (x, y) = self.pos;
        let (area_width, area_height) = if self.config.circle {
            let n = self.area_size.0.min(self.area_size.1);
            (n, n)
        } else {
            self.area_size
        };
        let zoom_range = (self.config.zoom_area.unwrap_or(50) as i32 + self.scale_zoom) as u32;
        let mut img = self.img.clone();
        let res = image::imageops::crop_imm(
            &img,
            x as u32 - (zoom_range / 2),
            y as u32 - (zoom_range / 2),
            zoom_range,
            zoom_range,
        )
        .to_image();

        let mut res = image::imageops::resize(
            &res,
            area_width,
            area_height,
            image::imageops::FilterType::Gaussian,
        );
        if self.config.circle {
            res = round_image(
                &res,
                self.config.border_thickness.unwrap_or_default() as f32,
                self.border_color,
            );
        } else if let Some(color) = self.border_color {
            generate_border(
                &mut res,
                self.config.border_thickness.unwrap_or_default(),
                color,
            );
        }
        image::imageops::overlay(
            &mut img,
            &res,
            (x - (area_width / 2) as i32).into(),
            (y - (area_height / 2) as i32).into(),
        );
        Some(img)
    }
}

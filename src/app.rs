use image::{Rgba, RgbaImage};
use libwayshot::WayshotConnection;

use crate::shot::{capture, generate_border, round_image, Area};
use crate::utils::str_to_color;
use crate::Config;

pub struct MainApp {
    pos: (i32, i32),
    size: (u32, u32),
    scale_factor: i32,
    wayshot: WayshotConnection,
    config: Config,
    border_color: Option<Rgba<u8>>,
}

#[derive(Debug, Clone)]
pub enum MainMessage {
    Move(i32, i32),
    ZoomIn(i32),
    ZoomOut(i32),
}

#[derive(Debug, Clone)]
pub enum Command {
    None,
    // Batch(impl IntoIterator<Item = Command>),
}

impl MainApp {
    pub fn new(config: Config) -> Self {
        let wayshot = WayshotConnection::new().unwrap();
        let border_color = config.border_color.as_deref().and_then(str_to_color);
        Self {
            pos: (0, 0),
            size: (config.width.unwrap_or(400), config.height.unwrap_or(200)),
            scale_factor: 0,
            config,
            wayshot,
            border_color,
        }
    }

    pub fn update(&mut self, msg: &MainMessage) -> Command {
        match msg {
            MainMessage::Move(x, y) => self.pos = (*x, *y),
            MainMessage::ZoomIn(z) => self.scale_factor += z,
            MainMessage::ZoomOut(z) => self.scale_factor -= z,
        }
        Command::None
    }

    pub fn render(&self) -> Option<RgbaImage> {
        let (x, y) = self.pos;
        let (width, height) = self.size;
        let zoom_range = (self.config.zoom_area.unwrap_or(50) as i32 + self.scale_factor) as u32;

        println!("Position: {x},{y}");
        let from_img = capture(
            &self.wayshot,
            Area {
                x,
                y,
                width: zoom_range,
                height: zoom_range,
            },
        );
        // .inspect_err(|e| println!("Problema al capturar: {e:?}"))
        // let (z_w, z_h) =
        //     calculate_ratio_size((zoom_range as u32, zoom_range as u32), (width, height));
        let mut res = image::imageops::resize(
            &from_img,
            width,
            height,
            image::imageops::FilterType::Gaussian,
        );
        image::imageops::crop(&mut res, 0, height / 2 - zoom_range, width, height);
        generate_border(&mut res, self.border_color.clone());
        Some(res)
    }
}

fn calculate_ratio_size((img_w, img_h): (u32, u32), (w_w, w_h): (u32, u32)) -> (u32, u32) {
    let w_ratio = w_w / img_w;
    let h_ratio = w_h / img_h;

    let ratio = if img_w > img_h {
        w_ratio.min(h_ratio)
    } else {
        w_ratio.max(h_ratio)
    };

    (img_w * ratio, img_h * ratio)
}

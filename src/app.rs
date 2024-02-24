use image::RgbaImage;
use libwayshot::WayshotConnection;
use libwayshot::output::OutputPositioning;

use crate::Config;

pub struct MainApp {
    pos: (i32, i32),
    size: (u32, u32),
    scale_factor: i32,
    wayshot: WayshotConnection,
    config: Config,
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
        Self {
            pos: (0, 0),
            size: (config.width.unwrap_or(400), config.height.unwrap_or(200)),
            scale_factor: 0,
            config,
            wayshot,
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
        let zoom_range = 50;

        println!("Position: {x},{y}");
        capture_area(&self.wayshot, (x, y), zoom_range)
            // .inspect_err(|e| println!("Problema al capturar: {e:?}"))
            .map(|from_img| {
                let (z_w, z_h) =
                    calculate_ratio_size((zoom_range as u32, zoom_range as u32), (width, height));
                image::imageops::resize(&from_img, z_w, z_h, image::imageops::FilterType::Triangle)
            })
    }
}

fn capture_area(wayshot: &WayshotConnection, (x, y): (i32, i32), size: i32) -> Option<RgbaImage> {
    wayshot
        .get_all_outputs()
        .iter()
        .find(|o| {
            let OutputPositioning {
                x: ox,
                y: oy,
                width,
                height,
            } = o.dimensions;
            x >= ox && (x - width) < ox + width && y >= oy && (y - height) < oy + height
        })
        .map(|o| {
            let img = wayshot.screenshot_single_output(o, false).unwrap();
            image::imageops::crop_imm(
                &img,
                (x - o.dimensions.width - (size / 2)) as u32,
                (y - o.dimensions.height - (size / 2)) as u32,
                size as u32,
                size as u32,
            )
            .to_image()
        })
}

fn calculate_ratio_size((img_w, img_h): (u32, u32), (w_w, w_h): (u32, u32)) -> (u32, u32) {
    let w_ratio = w_w / img_w;
    let h_ratio = w_h / img_h;

    let ratio = w_ratio.max(h_ratio);

    (img_w * ratio, img_h * ratio)
}

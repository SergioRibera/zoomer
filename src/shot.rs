use image::{Rgba, RgbaImage};
#[cfg(feature = "wayland")]
use libwayshot::output::OutputPositioning;
#[cfg(feature = "wayland")]
use libwayshot::reexport::Transform;
#[cfg(feature = "wayland")]
use libwayshot::WayshotConnection;

#[derive(Clone, Copy, Debug, Default)]
pub struct Area {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[cfg(feature = "wayland")]
fn rotate(screen: &RgbaImage, t: Transform) -> RgbaImage {
    use image::imageops::{rotate180, rotate270, rotate90};

    match t {
        Transform::_90 => rotate90(screen),
        Transform::_180 => rotate180(screen),
        Transform::_270 => rotate270(screen),
        _ => screen.clone(),
    }
}

pub fn capture(
    #[cfg(feature = "wayland")] wayshot: &WayshotConnection,
    name: Option<String>,
) -> RgbaImage {
    let mut xc = None;

    #[cfg(feature = "x11")]
    {
        xc = xcap::Monitor::from_point(x, y)
            .expect(&format!("Not monitor from point ({}, {})", x, y))
            .capture_image()
            .map(|out| out.into())
            .ok();
    }

    #[cfg(feature = "wayland")]
    wayshot
        .get_all_outputs()
        .iter()
        .find(|o| {
            if let Some(n) = name.clone() {
                return o.name == n;
            }
            false
        })
        .map(|o| {
            rotate(
                &wayshot.screenshot_single_output(o, false).unwrap(),
                o.transform,
            )
        })
        .or(xc)
        .unwrap()
}

pub fn generate_border(img: &mut RgbaImage, border_thickness: u32, color: Rgba<u8>) {
    let (width, height) = img.dimensions();
    // TODO: extract from image
    let color = color;

    for x in 0..width {
        for y in 0..border_thickness {
            img.put_pixel(x, y, color);
            img.put_pixel(x, height - y - 1, color);
        }
    }

    for y in 0..height {
        for x in 0..border_thickness {
            img.put_pixel(x, y, color);
            img.put_pixel(width - x - 1, y, color);
        }
    }
}

pub fn round_image(img: &RgbaImage, border_thickness: f32, color: Option<Rgba<u8>>) -> RgbaImage {
    let (width, height) = img.dimensions();
    let border_radius = (width.max(height) as f32 / 2.) - border_thickness;
    let mut new_img = RgbaImage::new(width, height);

    for x in 0..width {
        for y in 0..height {
            let dx = (x as f32 - width as f32 / 2.0).abs();
            let dy = (y as f32 - height as f32 / 2.0).abs();
            let distance = (dx.powi(2) + dy.powi(2)).sqrt();

            if distance < border_radius {
                let pixel = img.get_pixel(x, y);
                new_img.put_pixel(x, y, pixel.clone());
            }
            if let Some(color) = color {
                if distance >= border_radius - border_thickness
                    && distance < border_radius + border_thickness
                {
                    new_img.put_pixel(x, y, color);
                }
            }
        }
    }

    new_img
}

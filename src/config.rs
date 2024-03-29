use clap::Parser;
use merge2::{Merge, bool::overwrite_true};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Deserialize, Merge, Parser, Serialize)]
#[clap(version, author)]
/// Application to zoom to certain areas of the screen.
/// Use the Mouse Wheel to zoom in and zoom out.
/// Use Alt + Mouse Whell to increase the area size
pub struct Config {
    #[clap(long, short, default_value = "false")]
    #[merge(strategy = overwrite_true)]
    /// Make circle zoom area
    pub circle: bool,
    #[clap(long, default_value = "400")]
    #[merge(strategy = swap_option)]
    /// Initial width of tool
    pub width: Option<u32>,
    #[clap(long, default_value = "200")]
    #[merge(strategy = swap_option)]
    /// Initial height of tool
    pub height: Option<u32>,
    #[clap(long, short, default_value = "50")]
    #[merge(strategy = swap_option)]
    /// Initial height of tool
    pub zoom_area: Option<u32>,
    #[clap(long, short = 'b', default_value = "auto")]
    #[merge(strategy = swap_option)]
    /// Initial height of tool
    pub border_color: Option<String>,
    #[clap(long, short = 't', default_value = "5")]
    #[merge(strategy = swap_option)]
    /// Border thickness
    pub border_thickness: Option<u32>
}

#[inline]
fn swap_option<T>(left: &mut Option<T>, right: &mut Option<T>) {
    if left.is_none() || right.is_some() {
        core::mem::swap(left, right);
    }
}

pub fn get_config() -> Config {
    let config_path = directories::BaseDirs::new()
        .unwrap()
        .config_dir()
        .join("zoomer");

    let _ = std::fs::create_dir_all(config_path.clone());

    let config_path = config_path.join("config.toml");
    let mut args = Config::parse();

    std::fs::read_to_string(config_path)
        .map(|cfg_content| {
            let mut config: Config = toml::from_str(&cfg_content).unwrap();
            config.merge(&mut args);
            config
        })
        .unwrap_or(args)
}

use clap::{Parser, ValueEnum};
use merge2::Merge;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
pub enum Behaviour {
    #[default]
    Default,
    MacOS,
}

#[derive(Clone, Default, Debug, Deserialize, Merge, Parser, Serialize)]
#[clap(version, author)]
pub struct Config {
    #[clap(long, short)]
    #[merge(strategy = swap_option)]
    /// Location for the launcher panel
    pub behaviour: Option<Behaviour>,
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

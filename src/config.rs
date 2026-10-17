use anyhow::{Context, Result};
use directories::BaseDirs;
use serde::Deserialize;
use std::{env, fs, io::ErrorKind, path::PathBuf};

use crate::backends::{self, Backend::Fcitx5, RimeMode::Ascii};

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    pub default_backend: backends::Backend,
    pub fcitx5_rime_mode: backends::RimeMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_backend: Fcitx5,
            fcitx5_rime_mode: Ascii,
        }
    }
}

fn path() -> Result<PathBuf> {
    let config_dir = match env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
    {
        Some(path) => path,
        None => {
            let base_dirs = BaseDirs::new().context("Cannot determine home path")?;

            base_dirs.config_dir().to_path_buf()
        }
    };

    Ok(config_dir.join("limes").join("config.toml"))
}

pub fn load() -> Result<Config> {
    let path = path()?;

    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Ok(Config::default());
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("Cannot read config file: {}", path.display()));
        }
    };

    toml::from_str(&contents)
        .with_context(|| format!("Cannot parse config file: {}", path.display()))
}

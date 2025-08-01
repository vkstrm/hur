use serde::{Deserialize, Serialize};

use crate::error;
use crate::error::Error;
use crate::{io::read_file, modes::RedirectMode};
use std::env;
use std::fs::{DirBuilder, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub redirect_mode: Option<RedirectMode>,
}

pub fn load_config() -> Result<Option<Config>, Error> {
    let path = get_config_path()?;
    let file = match read_file(&path) {
        Ok(file) => file,
        Err(_) => return Ok(None),
    };
    let config: Config = match toml::from_str(&file) {
        Ok(c) => c,
        Err(why) => error!(&why.to_string()),
    };

    Ok(Some(config))
}

fn get_config_path() -> Result<PathBuf, Error> {
    let mut dir = match env::home_dir() {
        Some(dir) => dir,
        None => error!("Can't get home directory"),
    };
    dir.push(".config/hur/config.toml");
    Ok(dir)
}

pub fn create_default_config() -> Result<(), Error> {
    let path = get_config_path()?;
    if path.exists() {
        error!("Config file already exists")
    }

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            DirBuilder::new().create(parent)?;
        }
    }

    let config = Config {
        redirect_mode: Some(RedirectMode::NoFollow),
    };
    let config_string = match toml::to_string(&config) {
        Ok(string) => string,
        Err(why) => error!(&why.to_string()),
    };

    println!("Created config at {}", path.to_str().unwrap());
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(config_string.as_bytes())?;
    Ok(())
}

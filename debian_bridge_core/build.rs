#[macro_use]
extern crate dirs;
extern crate xdg;

use std::{error::Error, path::Path};

fn main() {
    if !cfg!(target_os = "linux") {
        panic!("Only linux supported for now.");
    }

    assets::prepare_assets().unwrap_or_else(|err| {
        println!("Can not load assets: {}", err.to_string());
    });
}

mod assets {
    use std::{
        error::Error,
        path::{Path, PathBuf},
    };

    const ICON_NAME_DEFAULT: &str = "debian_bridge_default.ico";

    pub fn prepare_assets() -> Result<(), Box<dyn Error>> {
        let mut path = dirs::home_dir().unwrap();

        prepare_icon_assets(path.as_path())?;

        Ok(())
    }

    fn prepare_icon_assets(path: &Path) -> Result<PathBuf, Box<dyn Error>> {
        let mut path = path.to_owned();
        path.push(".icons");

        if !path.exists() {
            std::fs::create_dir(&path);
        }
        let mut path = default_icon_path(path.as_path());

        if !path.exists() {
            std::fs::write(&path, include_bytes!("./resources/default.ico").to_vec())?;
        }

        Ok(path)
    }

    fn default_icon_path(path: &Path) -> PathBuf {
        let mut path = path.to_owned();
        path.push(ICON_NAME_DEFAULT);
        path
    }
}

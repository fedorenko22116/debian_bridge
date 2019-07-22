pub mod driver;

use std::error::Error;
use driver::*;
use tokio::{prelude::Future, runtime::Runtime};
use shiplift::{Docker, rep::Version};
use std::ffi::OsString;

pub struct System {
    pub wm: Option<WindowManager>,
    pub sd: Option<SoundDriver>,
    pub pd: Option<PrinterDriver>,
    pub wcm: Option<WebCamDriver>,
    pub docker_version: DockerVersion,
}

impl System {
    pub fn try_new(docker: &Docker) -> Result<Self, Box<dyn Error>> {
        Ok(
            Self {
                wm: Self::get_window_manager(),
                sd: Self::get_sound_driver(),
                pd: Self::get_printer_driver(),
                wcm: Self::get_web_cam_driver(),
                docker_version: Self::get_docker(docker)?,
            }
        )
    }

    fn get_docker(docker: &Docker) -> Result<DockerVersion, Box<dyn Error>> {
        let version = docker.version();
        let mut rt = Runtime::new().unwrap();

        let result = match rt.block_on(version) {
            Ok(Version { api_version: v, .. }) => Ok(DockerVersion::Default),
            Err(err) => Err(err.into())
        };

        rt.shutdown_now().wait().unwrap();
        result
    }

    fn get_window_manager() -> Option<WindowManager> {
        match std::env::var_os("XDG_SESSION_TYPE") {
            Some(os_string) =>  {
                match os_string.as_os_str().to_str() {
                    Some("x11") => Some(WindowManager::X11),
                    Some("wayland") => Some(WindowManager::Wayland),
                    _ => None,
                }
            },
            _ => None,
        }
    }

    fn get_sound_driver() -> Option<SoundDriver> {
        None
    }

    fn get_printer_driver() -> Option<PrinterDriver> {
        None
    }

    fn get_web_cam_driver() -> Option<WebCamDriver> {
        None
    }
}

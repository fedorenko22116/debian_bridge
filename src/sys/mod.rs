pub mod driver;
pub mod error;

use std::error::Error;
use driver::*;
use tokio::{prelude::Future, runtime::Runtime};
use shiplift::{Docker, rep::Version};
use std::ffi::OsString;
use std::process::{Command, ExitStatus, Stdio};
use std::fs::File;
use std::fmt::{Display, Formatter};
use error::SystemError;
use colorful::Color;
use colorful::Colorful;

type SystemResult<T> = Result<T, SystemError>;

#[derive(Clone)]
pub struct System {
    pub wm: Option<WindowManager>,
    pub sd: Option<SoundDriver>,
    pub pd: Option<PrinterDriver>,
    pub wcm: Option<WebCamDriver>,
    pub docker_version: DockerVersion,
}

impl System {
    pub fn try_new(docker: &Docker) -> SystemResult<Self> {
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

    fn get_docker(docker: &Docker) -> SystemResult<DockerVersion> {
        let version = docker.version();
        let mut rt = Runtime::new().unwrap();

        let result = match rt.block_on(version) {
            Ok(Version { api_version: v, .. }) => Ok(DockerVersion(v.to_owned())),
            Err(err) => Err(SystemError::DockerConnection)
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
        let pulse = Command::new("pactl")
            .arg("list")
            .stdout(Stdio::null())
            .status();

        let alsa = Command::new("aplay")
            .arg("-l")
            .stdout(Stdio::null())
            .status();

        match pulse {
            Ok(_) => Some(SoundDriver::PulseAudio),
            _ => match alsa {
                Ok(_) => Some(SoundDriver::Alsa),
                _ => None,
            }
        }
    }

    fn get_printer_driver() -> Option<PrinterDriver> {
        let printer_exists = Command::new("lpstat")
            .arg("-d")
            .stdout(Stdio::null())
            .status();

        match printer_exists {
            Ok(_) => Some(PrinterDriver::Default),
            _ => None,
        }
    }

    fn get_web_cam_driver() -> Option<WebCamDriver> {
        let wcm = Command::new("ls")
            .arg("-ld")
            .arg("/sys/class/video4linux/video0/device/driver")
            .stdout(Stdio::null())
            .status();

        match wcm {
            Ok(_) => Some(WebCamDriver::Default),
            _ => None,
        }
    }
}

impl Display for System {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "\n\n\
                \tDocker version ===> {docker_version}\n\
                \tWindow manager ===> {window_manager}\n\
                \tSound driver   ===> {sound_driver}\n\
                \tPrinter driver ===> {printer_driver}\n\
                \tWebcam driver  ===> {webcam_driver}\n\
            ",
                docker_version = DisplayOption(Some(self.docker_version.to_owned())),
                window_manager = DisplayOption(self.wm.to_owned()),
                sound_driver = DisplayOption(self.sd.to_owned()),
                printer_driver = DisplayOption(self.pd.to_owned()),
                webcam_driver = DisplayOption(self.wcm.to_owned())
        )
    }
}

struct DisplayOption<T>(pub Option<T>);

impl<T: Driver> Display for DisplayOption<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.0 {
            Some(ref v) => {
                write!(f, "{}", format!("{}", v).color(Color::Green))
            },
            None => write!(f, "{}", "None".color(Color::Red))
        }
    }
}

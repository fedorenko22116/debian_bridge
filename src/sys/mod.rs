pub mod driver;
pub mod error;

use colorful::{Color, Colorful};
use driver::*;
use error::SystemError;
use shiplift::{rep::Version, Docker};
use std::{
    error::Error,
    ffi::OsString,
    fmt::{Display, Formatter},
    fs::File,
    process::{Command, ExitStatus, Stdio},
};
use tokio::{prelude::Future, runtime::Runtime};

type SystemResult<T> = Result<T, SystemError>;

#[derive(Clone)]
pub struct System {
    pub wm: Option<WindowManager>,
    pub sd: Option<SoundDriver>,
    pub docker_version: DockerVersion,
}

impl System {
    pub fn try_new(docker: &Docker) -> SystemResult<Self> {
        Ok(Self {
            wm: Self::get_window_manager(),
            sd: Self::get_sound_driver(),
            docker_version: Self::get_docker(docker)?,
        })
    }

    fn get_docker(docker: &Docker) -> SystemResult<DockerVersion> {
        let version = docker.version();
        let mut rt = Runtime::new().unwrap();

        let result = match rt.block_on(version) {
            Ok(Version { api_version: v, .. }) => Ok(DockerVersion(v.to_owned())),
            Err(err) => Err(SystemError::DockerConnection),
        };

        rt.shutdown_now()
            .wait()
            .map_err(|err| SystemError::DockerConnection)?;

        result
    }

    fn get_window_manager() -> Option<WindowManager> {
        std::env::var_os("XDG_SESSION_TYPE")
            .map(|os_string| match os_string.as_os_str().to_str() {
                Some("x11") => Some(WindowManager::X11),
                Some("wayland") => Some(WindowManager::Wayland),
                _ => return None,
            })
            .unwrap()
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

        pulse
            .ok()
            .map(|o| SoundDriver::PulseAudio)
            .or(alsa.ok().map(|o| SoundDriver::Alsa))
    }
}

impl Display for System {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(
            f,
            "\n\n\tDocker version ===> {docker_version}\n\tWindow manager ===> \
             {window_manager}\n\tSound driver   ===> {sound_driver}\n",
            docker_version = DisplayOption(Some(self.docker_version.to_owned())),
            window_manager = DisplayOption(self.wm.to_owned()),
            sound_driver = DisplayOption(self.sd.to_owned()),
        )
    }
}

struct DisplayOption<T>(pub Option<T>);

impl<T: Driver> Display for DisplayOption<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.0 {
            Some(ref v) => write!(f, "{}", format!("{}", v).color(Color::Green)),
            None => write!(f, "{}", "None".color(Color::Red)),
        }
    }
}

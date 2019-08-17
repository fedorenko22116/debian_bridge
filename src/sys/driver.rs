use std::fmt::{Display, Formatter, Result};

#[derive(Clone)]
pub enum WindowManager {
    X11,
    Wayland,
}

impl Display for WindowManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let val = match self {
            WindowManager::X11 => "X11",
            WindowManager::Wayland => "Wayland",
        };

        write!(f, "{}", val)
    }
}

impl Driver for WindowManager {}

#[derive(Clone)]
pub enum SoundDriver {
    Alsa,
    PulseAudio,
}

impl Display for SoundDriver {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let val = match self {
            SoundDriver::Alsa => "Alsa",
            SoundDriver::PulseAudio => "PulseAudio",
        };

        write!(f, "{}", val)
    }
}

impl Driver for SoundDriver {}

#[derive(Clone)]
pub enum PrinterDriver {
    Default,
}

impl Display for PrinterDriver {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", "Default driver installed")
    }
}

impl Driver for PrinterDriver {}

#[derive(Clone)]
pub enum WebCamDriver {
    Default,
}

impl Display for WebCamDriver {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", "Default driver installed")
    }
}

impl Driver for WebCamDriver {}

#[derive(Clone)]
pub struct DockerVersion(pub String);

impl Display for DockerVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

impl Driver for DockerVersion {}

pub trait Driver: Display + Clone {}

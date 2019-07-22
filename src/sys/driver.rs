#[derive(Debug)]
pub enum WindowManager {
    X11,
    Wayland
}

#[derive(Debug)]
pub enum SoundDriver {
    Alsa,
    PulseAudio
}

#[derive(Debug)]
pub enum PrinterDriver {
    Default
}

#[derive(Debug)]
pub enum WebCamDriver {
    Default
}

#[derive(Debug)]
pub enum DockerVersion {
    Default
}
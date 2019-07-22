use std::path::PathBuf;
use std::str::FromStr;
use std::error::Error;

#[derive(Clone)]
pub enum Feature {
    Display,
    Sound,
    Notification,
    Webcam,
    Printer,
    HomePersistent,
}

#[derive(Clone)]
pub struct Program {
    pub path: PathBuf,
    pub settings: Vec<Feature>,
}

impl FromStr for Program {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        unimplemented!()
    }
}

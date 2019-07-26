use std::path::Path;
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone)]
pub struct Config {
    programs: Vec<Program>,
}

impl Config {
    pub fn new(programs: &Vec<Program>) -> Self {
        Config { programs: programs.to_owned() }
    }

    pub fn deserialize(path: &Path) -> Result<Self, Box<dyn Error>> {
        unimplemented!()
    }

    pub fn serialize(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }
}

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

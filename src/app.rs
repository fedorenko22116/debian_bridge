extern crate shiplift;

use std::path::Path;

#[derive(Copy, Clone)]
pub struct Config {
    programs: Vec<Program>,
}

impl Config {
    pub fn new(programs: &Vec<Program>) -> Self {
        Config { programs: programs.to_owned() }
    }

    pub fn deserialize(path: &Path) -> Self {

    }

    pub fn serialize(path: &Path) {

    }
}

pub enum Setting {
    Display,
    Sound,
    Notification,
}

#[derive(Copy, Clone)]
pub struct Program {
    path: String,
    settings: Vec<Setting>,
}

pub struct App {
    config: Config
}

impl App {
    pub fn list(&self) {

    }

    pub fn remove(&self, str: &str) {

    }

    pub fn test(&self) {

    }

    pub fn create(&self, path: &Path) {

    }

    pub fn new(config: &Config) -> Self {
        App { config: config.to_owned() }
    }
}

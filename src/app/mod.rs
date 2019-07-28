mod config;
pub mod error;

pub use config::{Config, Program, Feature};
use std::path::Path;
use std::net::IpAddr;
use shiplift::Docker;
use std::error::Error;
use crate::System;

pub struct App {
    config: Config,
    system: System,
    docker: Docker,
}

impl App {
    pub fn list(&self) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }

    pub fn remove(&self, program: &str) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }

    pub fn create(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        self.config.serialize(path)?;
        Ok(())
    }

    pub fn new(config: &Config, system: &System, docker: &Docker) -> Self {
        App {
            config: config.to_owned(),
            system: system.to_owned(),
            docker: docker.to_owned(),
        }
    }
}

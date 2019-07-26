mod config;

pub use config::{Config, Program, Feature};
use std::path::Path;
use std::net::IpAddr;
use shiplift::Docker;
use std::error::Error;

pub struct App {
    pub config: Config,
    docker: Docker,
}

impl App {
    pub fn list(&self) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }

    pub fn remove(&self, program: &Program) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }

    pub fn create(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }

    pub fn rpc(&self, host: &IpAddr, port: &u16) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }

    pub fn new(config: &Config, docker: &Docker) -> Self {
        App {
            config: config.to_owned(),
            docker: docker.to_owned(),
        }
    }
}

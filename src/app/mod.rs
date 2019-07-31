mod config;
pub mod error;

pub use config::{Config, Program, Feature};
use crate::System;
use std::path::Path;
use std::net::IpAddr;
use shiplift::Docker;
use std::error::Error;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use colorful::Color;
use colorful::Colorful;
use crate::sys::driver::Driver;

pub struct FeaturesList {
    list: HashMap<Feature, bool>,
}

impl FeaturesList {
    pub fn new(system: &System) -> Self {
        let mut list = HashMap::new();

        FeaturesList::add_feature_if_driver_exists(&mut list, Feature::Display, &system.wm);
        FeaturesList::add_feature_if_driver_exists(&mut list, Feature::Sound, &system.sd);
        FeaturesList::add_feature_if_driver_exists(&mut list, Feature::Webcam, &system.wcm);
        FeaturesList::add_feature_if_driver_exists(&mut list, Feature::Printer, &system.pd);

        list.insert(Feature::HomePersistent, true);
        list.insert(Feature::Notification, true);
        list.insert(Feature::Shortcut, true);

        Self {list}
    }

    fn add_feature_if_driver_exists<T: Driver>(list: &mut HashMap<Feature, bool>, f: Feature, d: &Option<T>) {
        if let Some(_feature) = d {
            list.insert(f, true);
        } else {
            list.insert(f, false);
        }
    }
}

pub struct App {
    config: Config,
    docker: Docker,
    pub features: FeaturesList,
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

    pub fn run(&self, program: &str) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        self.config.serialize(path)?;
        Ok(())
    }

    pub fn new(config: &Config, system: &System, docker: &Docker) -> Self {
        App {
            config: config.to_owned(),
            docker: docker.to_owned(),
            features: FeaturesList::new(&system)
        }
    }
}

impl Display for FeaturesList {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        writeln!(f, "\n");

        for (feature, available) in &self.list {
            writeln!(f, "\t{:<15} ===> {}", format!("{}", feature), match available {
                true => "available".color(Color::Green),
                false => "unavailable".color(Color::Red),
            });
        }

        Ok(())
    }
}

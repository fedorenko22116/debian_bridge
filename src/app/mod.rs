pub mod error;
mod config;
mod deb;
mod util;
mod docker;

pub use config::{Config, Program, Feature, Icon};
use std::path::{Path, PathBuf};
use std::net::IpAddr;
use std::error::Error;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ffi::OsStr;
use colorful::Color;
use colorful::Colorful;
use colorful::core::StrMarker;
use shiplift::Docker;
use crate::sys::driver::Driver;
use crate::System;
use crate::app::deb::Deb;
use crate::app::error::AppError;
use crate::app::docker::DockerFacade;

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

        Self {list}
    }

    fn add_feature_if_driver_exists<T: Driver>(list: &mut HashMap<Feature, bool>, f: Feature, d: &Option<T>) {
        list.insert(f, d.is_some());
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

pub struct App<'a> {
    prefix: String,
    cache_path: PathBuf,
    config: Config,
    docker: DockerFacade<'a>,
    pub features: FeaturesList,
}

impl<'a> App<'a> {
    pub fn list(&self) -> Vec<String> {
        self.config.programs.iter()
            .map(|program| (&program).get_name_short().to_owned())
            .collect::<Vec<String>>()
            .to_vec()
    }

    pub fn remove<T: Into<String>>(&mut self, program: T) -> Result<&Self, Box<dyn Error>> {
        let program = match self.config.find(program.into()) {
            Some(p) => p,
            None => return Err(AppError::Program("Input program doesn't exist".to_str()).into()),
        };

        self.docker.delete(&program.0);

        if let Some(_) = program.0.icon {
            let mut path = dirs::desktop_dir().unwrap();
            let name = format!("{}.desktop", program.0.get_name_short());

            path.push(name);

            std::fs::remove_file(path).unwrap_or_else(|err| {
                error!("Can't remove an entry file: '{}'", err.to_string());
                ()
            });
        }

        self.config.remove(&program.0)?;

        Ok(self)
    }

    pub fn create(&mut self, app_path: &Path, settings: Vec<Feature>, icon: &Option<Icon>) -> Result<&Self, Box<dyn Error>> {
        let deb = Deb::try_new(app_path)?;
        let mut app_tmp_path = self.cache_path.to_owned();
        app_tmp_path.push(Path::new("tmp.deb"));

        std::fs::copy(app_path, &app_tmp_path);

        let mut dockerfile = util::gen_dockerfile(&deb);

        debug!("Generated dockerfile:\n{}", dockerfile);

        let mut dockerfile_path = self.cache_path.to_owned();
        dockerfile_path.push(Path::new("Dockerfile"));

        std::fs::write(&dockerfile_path, dockerfile)?;

        self.config.push(&Program::new(&deb.package, &app_path, &settings, &icon))?;
        self.docker.create(&deb);

        std::fs::remove_file(&dockerfile_path)?;
        std::fs::remove_file(&app_tmp_path)?;

        if let Some(icon) = &icon {
            let entry = util::gen_desktop_entry(
                &deb.package,
                &deb.description.unwrap_or("Application".to_string()),
                &icon.path
            );
            let mut path = dirs::desktop_dir().unwrap();

            debug!("Generated new entry in '{}':\n{}", path.to_str().unwrap(), entry);

            if !path.exists() {
                std::fs::create_dir(&path)?;
            }

            path.push(format!("{}.desktop", deb.package));

            std::fs::write(path, entry)?;
        }

        Ok(self)
    }

    pub fn run<T: Into<String>>(&self, program: T) -> Result<&Self, Box<dyn Error>> {
        let program = self.config.find(program)
            .ok_or(AppError::Program("Program not found".to_string()))?.0;

        self.docker.run(&program)?;
        Ok(self)
    }

    pub fn save(&self, path: &Path) -> Result<&Self, Box<dyn Error>> {
        self.config.serialize(path)?;
        debug!("Config updated");
        Ok(self)
    }

    pub fn new<T: Into<String>>(prefix: T, cache_path: &Path, config: &Config, system: &'a System, docker: &'a Docker) -> Self {
        let prefix = prefix.into();

        App {
            prefix: prefix.to_owned(),
            config: config.to_owned(),
            docker: DockerFacade::new(docker, system, prefix, cache_path),
            cache_path: cache_path.to_owned(),
            features: FeaturesList::new(&system)
        }
    }
}

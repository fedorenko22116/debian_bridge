pub mod error;
mod config;
mod deb;
mod util;

pub use config::{Config, Program, Feature, Icon};
use tokio::{prelude::Future, runtime::Runtime};
use tokio::prelude::{Stream};
use std::path::{Path, PathBuf};
use std::net::IpAddr;
use shiplift::{Docker, BuildOptions};
use std::error::Error;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ffi::OsStr;
use colorful::Color;
use colorful::Colorful;
use colorful::core::StrMarker;
use crate::sys::driver::Driver;
use crate::System;
use crate::app::deb::Deb;
use crate::app::error::AppError;

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
        list.insert(f, if let Some(_feature) = d { true } else { false });
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

pub struct App {
    prefix: String,
    cache_path: PathBuf,
    config: Config,
    docker: Docker,
    pub features: FeaturesList,
}

impl App {
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
        let fut = self.docker
            .images()
            .get(&program.0.get_name(&self.prefix))
            .delete();
        let mut rt = Runtime::new().unwrap();

        rt.block_on(fut)?;
        rt.shutdown_now().wait();

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

        let fut = self.docker
            .images()
            .build(
                &BuildOptions::builder(
                    self.cache_path.as_os_str().to_str().unwrap()
                ).tag(&format!("{}_{}", self.prefix, deb.package)).build()
            )
            .for_each(|output| {
                debug!("{}", output);
                Ok(())
            })
            .map_err(|e| return e);
        let mut rt = Runtime::new().unwrap();

        rt.block_on(fut)?;
        rt.shutdown_now().wait();

        std::fs::remove_file(&dockerfile_path)?;
        std::fs::remove_file(&app_tmp_path)?;

        if let Some(icon) = icon {
            let entry = util::gen_desktop_entry(&deb.package, &deb.description.unwrap_or("Application".to_string()));
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
        unimplemented!()
    }

    pub fn save(&self, path: &Path) -> Result<&Self, Box<dyn Error>> {
        self.config.serialize(path)?;
        debug!("Config updated");
        Ok(self)
    }

    pub fn new<T: Into<String>>(prefix: T, cache_path: &Path, config: &Config, system: &System, docker: &Docker) -> Self {
        App {
            prefix: prefix.into(),
            config: config.to_owned(),
            docker: docker.to_owned(),
            cache_path: cache_path.to_owned(),
            features: FeaturesList::new(&system)
        }
    }
}

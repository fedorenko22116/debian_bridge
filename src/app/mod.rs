mod config;
mod deb;
pub mod error;

pub use config::{Config, Program, Feature};
use tokio::{prelude::Future, runtime::Runtime};
use tokio::prelude::{Stream};
use std::path::{Path, PathBuf};
use std::net::IpAddr;
use shiplift::{Docker, BuildOptions};
use std::error::Error;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use colorful::Color;
use colorful::Colorful;
use dockerfile::{Dockerfile, Arg, Copy, Cmd, Run, User, Env, Workdir};
use crate::sys::driver::Driver;
use crate::System;
use std::ffi::OsStr;
use crate::app::deb::Deb;

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
        list.insert(f, if let Some(_feature) = d { true } else { false });
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
            .map(|program| (&program).get_name(&self.prefix).to_owned())
            .collect::<Vec<String>>()
            .to_vec()
    }

    pub fn remove(&mut self, program: &str) -> Result<&Self, Box<dyn Error>> {
        let fut = self.docker
            .images()
            .get(&program)
            .delete();
        let mut rt = Runtime::new().unwrap();

        rt.block_on(fut)?;
        rt.shutdown_now().wait();

        self.config.remove(
            &program.to_string()
        )?;

        Ok(self)
    }

    pub fn create(&self, app_path: &Path) -> Result<&Self, Box<dyn Error>> {
        let deb = Deb::try_new(app_path)?;
        let mut app_tmp_path = self.cache_path.to_owned();
        app_tmp_path.push(Path::new("tmp.deb"));

        std::fs::copy(app_path, &app_tmp_path);

        let mut dockerfile = Dockerfile::base("debian:9-slim")
            .push_initial_arg(Arg::new("informuser"))
            .push(Workdir::new("/data"))
            .push(Copy::new("tmp.deb /data/application.deb"))
            .push(Run::new("apt-get update"));

        if let Some(d) = deb.dependencies {
            dockerfile = dockerfile.push(Run::new(
                format!("apt-get install -y {}", d.replace(&[','][..], "")))
            );
        }

        let dockerfile = dockerfile
            .push(Run::new("dpkg -i /data/application.deb || true"))
            .push(Run::new("apt-get install -y -f --no-install-recommends && rm -rf /var/lib/apt/lists/* && useradd $informuser"))
            .push(User::new("$informuser"))
            .push(Env::new("HOME /home/$informuser"))
            .push(Cmd::new(deb.package.to_owned()))
            .finish();

        let mut dockerfile_path = self.cache_path.to_owned();
        dockerfile_path.push(Path::new("Dockerfile"));

        std::fs::write(&dockerfile_path, dockerfile.to_string())?;

        let fut = self.docker
            .images()
            .build(
                &BuildOptions::builder(
                    self.cache_path.as_os_str().to_str().unwrap()
                ).tag(format!("{}_{}", self.prefix, deb.package)).build()
            )
            .for_each(|output| {
                println!("{}", output);
                Ok(())
            })
            .map_err(|e| return e);
        let mut rt = Runtime::new().unwrap();

        rt.block_on(fut)?;
        rt.shutdown_now().wait();

//        std::fs::remove_file(&dockerfile_path)?;
//        std::fs::remove_file(&app_tmp_path)?;

        Ok(self)
    }

    pub fn run(&self, program: &str) -> Result<&Self, Box<dyn Error>> {
        unimplemented!()
    }

    pub fn save(&self, path: &Path) -> Result<&Self, Box<dyn Error>> {
        self.config.serialize(path)?;
        Ok(self)
    }

    pub fn new(prefix: &String, cache_path: &Path, config: &Config, system: &System, docker: &Docker) -> Self {
        App {
            prefix: prefix.to_owned(),
            config: config.to_owned(),
            docker: docker.to_owned(),
            cache_path: cache_path.to_owned(),
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

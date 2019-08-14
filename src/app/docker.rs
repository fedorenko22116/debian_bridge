use shiplift::{Docker, BuildOptions};
use tokio::{prelude::Future, runtime::Runtime};
use tokio::prelude::{Stream};
use std::error::Error;
use std::path::{PathBuf, Path};
use crate::{Program, Feature, System};
use crate::app::deb::Deb;
use crate::app::error::AppError;
use shiplift::rep::ContainerCreateInfo;
use shiplift::tty::StreamType;
use std::process::{Command, Stdio};
use colorful::core::StrMarker;

type AppResult<T> = Result<T, AppError>;

pub struct DockerFacade<'a> {
    docker: &'a Docker,
    system: &'a System,
    prefix: String,
    cache_path: PathBuf,
}

impl<'a> DockerFacade<'a> {
    pub fn new<T: Into<String>>(docker: &'a Docker, system: &'a System, prefix: T, cache_path: &Path) -> Self {
        DockerFacade {
            docker,
            system,
            prefix: prefix.into(),
            cache_path: cache_path.into(),
        }
    }

    pub fn delete(&mut self, program: &Program) -> AppResult<&Self> {
        let fut = self.docker
            .images()
            .get(&program.get_name(&self.prefix))
            .delete();
        let mut rt = Runtime::new().unwrap();

        rt.block_on(fut).map_err(|err| {
                error!("{}", err.to_string());
                AppError::Docker
            })?;
        rt.shutdown_now().wait();

        Ok(self)
    }

    pub fn create<T: Into<String>>(&mut self, name: T) -> AppResult<&Self> {
        let tag = format!("{}_{}", self.prefix, name.into());

        info!("Image name: {}", tag);

        let fut = self.docker
            .images()
            .build(
                &BuildOptions::builder(
                    self.cache_path.as_os_str().to_str().unwrap()
                ).tag(&tag).build()
            )
            .for_each(|output| {
                debug!("{}", output);
                Ok(())
            });
        let mut rt = Runtime::new().unwrap();

        rt.block_on(fut).map_err(|err| {
                error!("{}", err.to_string());
                AppError::Docker
            })?;
        rt.shutdown_now().wait();

        Ok(self)
    }

    //TODO: add more options and rewrite with docker API if possible
    pub fn run(&self, program: &Program) -> AppResult<&Self> {
        let home = std::env::var_os("HOME")
            .unwrap().to_str().unwrap().to_string();
        let cmd_name = program.get_name(&self.prefix);
        let home_volume = format!("{}:{}", home, home);
        let mut args = vec![
            "run", "-ti", "--net=host", "--rm",
            "-v", "/dev/shm:/dev/shm",
            "-v", "/etc/machine-id:/etc/machine-id",
            "-v", "/var/lib/dbus:/var/lib/dbus",
            "--privileged"
        ];

        if program.settings.contains(&Feature::Display) {
            args.push("-v");
            args.push("/tmp/.X11-unix:/tmp/.X11-unix");
            args.push("--env");
            args.push("DISPLAY");
        }

        if program.settings.contains(&Feature::Sound) {
            args.push("-v");
            args.push("/dev/snd:/dev/snd");
        }

        if program.settings.contains(&Feature::HomePersistent) {
            args.push("-v");
            args.push(&home_volume);
        }

        args.push(&cmd_name);

        let mut cmd = Command::new("docker")
            .args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|err| AppError::Docker)?;

        let status = cmd.wait();

        info!("Exited with status {:?}", status);

        Ok(self)
    }
}

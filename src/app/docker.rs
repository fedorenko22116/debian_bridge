use super::{error::AppError, Feature, Program, System};
use colorful::core::StrMarker;
use serde_json::Value;
use shiplift::{BuildOptions, ContainerListOptions, Docker};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
};
use tokio::{
    prelude::{Future, Stream},
    runtime::Runtime,
};

type AppResult<T> = Result<T, AppError>;

pub struct DockerFacade<'a> {
    docker: &'a Docker,
    system: &'a System,
    prefix: String,
    cache_path: PathBuf,
}

impl<'a> DockerFacade<'a> {
    pub fn new<T: Into<String>>(
        docker: &'a Docker,
        system: &'a System,
        prefix: T,
        cache_path: &Path,
    ) -> Self {
        DockerFacade {
            docker,
            system,
            prefix: prefix.into(),
            cache_path: cache_path.into(),
        }
    }

    fn get_containers(&self, image: &String) -> AppResult<Vec<String>> {
        let program_name = Arc::new(image.to_owned());

        let fut = self
            .docker
            .containers()
            .list(&ContainerListOptions::builder().all().build())
            .map(move |containers| {
                containers
                    .iter()
                    .filter_map(|c| {
                        if c.image.eq(&*program_name) {
                            Some(c.id.to_owned())
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .map_err(|_| ());

        let mut rt = Runtime::new().unwrap();

        let container_ids = match rt.block_on(fut) {
            Ok(res) => Ok(res),
            Err(err) => Err(AppError::Docker),
        };

        rt.shutdown_now().wait().map_err(|err| AppError::Docker)?;

        container_ids
    }

    fn delete_container(&self, id: &String) -> AppResult<&Self> {
        let fut = self
            .docker
            .containers()
            .get(&id)
            .delete()
            .map_err(|_| AppError::Docker);
        let mut rt = Runtime::new().unwrap();

        rt.block_on(fut).map_err(|err| {
            warn!("{}", err.to_string());
            AppError::Docker
        })?;
        rt.shutdown_now().wait().map_err(|err| AppError::Docker)?;

        Ok(self)
    }

    pub fn delete(&mut self, program: &Program) -> AppResult<&Self> {
        let name = program.get_name(&self.prefix);
        let containers_ids = self.get_containers(&name)?;

        containers_ids
            .iter()
            .try_for_each(|id| self.delete_container(&id).map(|_| ()))?;

        let fut = self.docker.images().get(&name).delete();
        let mut rt = Runtime::new().unwrap();

        rt.block_on(fut).map_err(|err| {
            warn!("{}", err.to_string());
            AppError::DockerStatus(404)
        })?;
        rt.shutdown_now().wait().map_err(|err| AppError::Docker)?;

        Ok(self)
    }

    pub fn create<T: Into<String>>(&mut self, name: T) -> AppResult<&Self> {
        let tag = format!("{}_{}", self.prefix, name.into());

        info!("Image name: {}", tag);

        let fut = self
            .docker
            .images()
            .build(
                &BuildOptions::builder(self.cache_path.as_os_str().to_str().unwrap())
                    .tag(&tag)
                    .build(),
            )
            .for_each(|output| {
                let output = output.as_object().unwrap();

                if output.contains_key("error") {
                    error!("Docker output: {}", output.get("error").unwrap());
                    return Err(shiplift::Error::InvalidResponse(
                        "Failed to build an image".to_string(),
                    ));
                }

                if output.contains_key("stream") {
                    info!("Docker output: {}", output.get("stream").unwrap());
                }

                Ok(())
            });
        let mut rt = Runtime::new().unwrap();

        rt.block_on(fut).map_err(|err| {
            error!("{}", err.to_string());
            AppError::Docker
        })?;
        rt.shutdown_now().wait().map_err(|err| AppError::Docker)?;

        Ok(self)
    }

    //TODO: add more options and rewrite with docker API if possible
    pub fn run(&self, program: &Program) -> AppResult<&Self> {
        let home = std::env::var_os("HOME")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let cmd_name = program.get_name(&self.prefix);
        let home_volume = format!("{}:{}", home, home);
        let mut args = vec![
            "run",
            "-ti",
            "--net=host",
            "--rm",
            "-v",
            "/dev/shm:/dev/shm",
            "-v",
            "/etc/machine-id:/etc/machine-id",
            "-v",
            "/var/lib/dbus:/var/lib/dbus",
            "--privileged",
        ];

        if program.settings.contains(&Feature::Display) {
            args.push_volume("/tmp/.X11-unix:/tmp/.X11-unix")
                .push_env("DISPLAY");
        }

        if program.settings.contains(&Feature::Sound) {
            args.push_volume("/dev/snd:/dev/snd");
        }

        if program.settings.contains(&Feature::HomePersistent) {
            args.push_volume(&home_volume);
        }

        if program.settings.contains(&Feature::Time) {
            args.push_volume("/etc/localtime:/etc/localtime");
        }

        if program.settings.contains(&Feature::Notification) {
            args.push_volume("/var/lib/dbus:/var/lib/dbus");
        }

        if program.settings.contains(&Feature::Devices) {
            args.push_volume("/dev:/dev");
        }

        args.push(&cmd_name);

        let mut cmd = Command::new("docker")
            .args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|err| AppError::Docker)?;

        let status = cmd.wait().map_err(|err| {
            error!("{}", err.to_string());
            AppError::Docker
        })?;

        info!("Exited with status {:?}", status);

        Ok(self)
    }
}

trait PushArgument<T: Into<String>> {
    fn push_volume(&mut self, v: T) -> &mut Self;
    fn push_env(&mut self, v: T) -> &mut Self;
}

impl<'a> PushArgument<&'a str> for Vec<&'a str> {
    fn push_volume(&mut self, v: &'a str) -> &mut Self {
        self.push("-v");
        self.push(v);
        self
    }

    fn push_env(&mut self, v: &'a str) -> &mut Self {
        self.push("--env");
        self.push(v);
        self
    }
}

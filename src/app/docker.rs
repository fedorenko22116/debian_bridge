use shiplift::{Docker, BuildOptions, ContainerOptions};
use tokio::{prelude::Future, runtime::Runtime};
use tokio::prelude::{Stream};
use std::error::Error;
use std::path::{PathBuf, Path};
use crate::{Program, Feature, System};
use crate::app::deb::Deb;
use crate::app::error::AppError;

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

        match rt.block_on(fut) {
            Ok(_) => (),
            Err(err) => {
                error!("{}", err.to_string());
                return Err(AppError::Docker)
            }
        };
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
            })
            .map_err(|e| return e);
        let mut rt = Runtime::new().unwrap();

        match rt.block_on(fut) {
            Ok(_) => (),
            Err(err) => {
                error!("{}", err.to_string());
                return Err(AppError::Docker)
            }
        };
        rt.shutdown_now().wait();

        Ok(self)
    }

    //TODO: add more options and add validations
    pub fn run(&self, program: &Program) -> AppResult<&Self> {
        let mut options = ContainerOptions::builder(
            program.get_name(&self.prefix).as_str()
        );

        options
            .attach_stderr(true)
            .attach_stdin(true)
            .attach_stdout(true)
            .network_mode("host")
            .privileged(true);

        if program.settings.contains(&Feature::Display) {
            options.env(vec!["DISPLAY"])
                .volumes(vec![
                    "/tmp/.X11-unix:/tmp/.X11-unix",
                ]);
        }

        if program.settings.contains(&Feature::HomePersistent) {
            options.env(vec!["$HOME:$HOME"]);
        }

        let options = options.build();

        let fut = self.docker
            .containers()
            .create(&options)
            .map(|i| info!("{:?}", i))
            .map_err(|e| error!("{}", e));
        let mut rt = Runtime::new().unwrap();

        match rt.block_on(fut) {
            Ok(_) => (),
            Err(err) => return Err(AppError::Docker)
        };
        rt.shutdown_now().wait();

        Ok(self)
    }
}

use shiplift::{Docker, BuildOptions};
use tokio::{prelude::Future, runtime::Runtime};
use tokio::prelude::{Stream};
use std::error::Error;
use std::path::{PathBuf, Path};
use crate::Program;
use crate::app::deb::Deb;
use crate::app::error::AppError;

type AppResult<T> = Result<T, AppError>;

pub struct DockerFacade<'a> {
    docker: &'a Docker,
    prefix: String,
    cache_path: PathBuf,
}

impl<'a> DockerFacade<'a> {
    pub fn new<T: Into<String>>(docker: &'a Docker, prefix: T, cache_path: &Path) -> Self {
        DockerFacade {
            docker,
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

    pub fn create(&mut self, deb: &Deb) -> AppResult<&Self> {
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
}

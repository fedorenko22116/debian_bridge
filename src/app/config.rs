use super::error::AppError;
use std::path::{Path, PathBuf};
use std::error::Error;
use std::str::FromStr;
use std::fs::File;
use std::io::BufReader;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::fmt::Display;
use std::ops::Deref;

pub type AppResult<T> = Result<T, AppError>;

const ICON_NAME_DEFAULT: &str = "debian_bridge_default.ico";

#[derive(Clone, Serialize, Deserialize)]
pub struct Icon {
    pub path: PathBuf,
}

impl Icon {
    pub fn new(path: &Path) -> Self {
        Icon {
            path: path.to_owned(),
        }
    }

    fn prepare_assets(path: &Path) -> Result<PathBuf, Box<dyn Error>> {
        let mut path = Self::default_icon_path(path);

        if !path.exists() {
            debug!("Icon image path: {:?}", path);
            std::fs::write(&path, include_bytes!("../../resources/default.ico").to_vec())?;
        }

        info!("Icon assets prepared");

        Ok(path)
    }

    fn default_icon_path(path: &Path) -> PathBuf {
        let mut path = path.to_owned();
        path.push(ICON_NAME_DEFAULT);
        path
    }
}

impl Default for Icon {
    fn default() -> Self {
        let mut path = dirs::home_dir().unwrap();
        path.push(".icons");

        if !path.exists() {
            std::fs::create_dir(&path);
        }

        Icon {
            path: Self::prepare_assets(
                path.as_path()
            ).unwrap()
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Feature {
    Display,
    Sound,
    Notification,
    Webcam,
    Printer,
    HomePersistent,
}

impl Display for Feature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match self {
            Feature::Display => "Display",
            Feature::Sound => "Sound",
            Feature::Notification => "Notification",
            Feature::Webcam => "Webcam",
            Feature::Printer => "Printer",
            Feature::HomePersistent => "Home persistent",
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Program {
    name: String,
    pub path: PathBuf,
    pub settings: Vec<Feature>,
    pub icon: Option<Icon>,
    pub command: String,
}

impl Program {
    pub fn get_name<T: Into<String>>(&self, prefix: T) -> String {
        format!("{}_{}", prefix.into(), self.name)
    }

    pub fn get_name_short(&self) -> String {
        self.name.to_owned()
    }

    pub fn new<T>(name: T, path: &Path, settings: &Vec<Feature>, icon: &Option<Icon>, cmd: &Option<String>) -> Self
        where T: Into<String> {
        let name = name.into();

        Program {
            name: name.to_owned(),
            path: path.to_owned(),
            settings: settings.to_vec(),
            icon: icon.to_owned(),
            command: cmd.to_owned()
                .unwrap_or(name),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub programs: Vec<Program>,
}

impl Config {
    pub fn deserialize(path: &Path) -> AppResult<Self> {
        if !path.exists() {
            return match File::create(path) {
                Result::Ok(_) => Ok(Config { programs: vec![] }),
                Result::Err(err) => Err(AppError::File(err.to_string()))
            }
        }

        let mut config_str = String::new();
        let config_file = match File::open(path) {
            Err(err) => return Err(AppError::File(err.to_string())),
            Ok(res) => res,
        };
        let mut br = BufReader::new(config_file);
        match br.read_to_string(&mut config_str) {
            Err(err) => return Err(AppError::File(err.to_string())),
            Ok(res) => res,
        };

        match serde_json::from_str(config_str.as_str()) {
            Ok(res) => Ok(res),
            Err(err) => Err(AppError::File(err.to_string())),
        }
    }

    pub fn serialize(&self, path: &Path) -> AppResult<&Self> {
        let data = match serde_json::to_string(&self) {
            Ok(res) => res,
            Err(err) => return Err(AppError::File(err.to_string())),
        };

        match std::fs::write(path, data.as_bytes()) {
            Ok(res) => Ok(self),
            Err(err) => Err(AppError::File(err.to_string()))
        }
    }

    pub fn push(&mut self, program: &Program) -> AppResult<&Self> {
        match self.programs.iter().find(|&x| x.name == program.name) {
            Some(elem) => return Err(
                AppError::Program(
                    format!("Program with such name already exists '{}'. Remove it first or use a custom tag with -t (--tag) option", program.name).to_string()
                )
            ),
            None => (),
        };

        self.programs.push(program.to_owned());
        Ok(self)
    }

    pub fn find<T: Into<String>>(&self, name: T) -> Option<(Program, usize)> {
        let name = name.into();
        let idx = self.programs.iter().position(|x| x.name == name)?;
        self.programs.get(idx).map(|p| (p.to_owned(), idx))
    }

    pub fn remove(&mut self, program: &Program) -> AppResult<&Self> {
        let program_idx = match self.find(&program.name) {
            Some(elem) => elem.1,
            None => return Err(
                AppError::Program(format!("Can't find a program '{}'", program.name).to_string())
            ),
        };

        self.programs.remove(program_idx);
        Ok(self)
    }

    pub fn clear(&mut self) -> &Self {
        self.programs = vec![];
        self
    }
}

use super::error::AppError;
use std::path::{Path, PathBuf};
use std::error::Error;
use std::str::FromStr;
use std::fs::File;
use std::io::BufReader;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::fmt::Display;

type AppResult<T> = Result<T, AppError>;

#[derive(Clone, Serialize, Deserialize)]
pub struct Icon {
    img: PathBuf,
    mime: String
}

impl Icon {
    pub fn new() -> Self {
        unimplemented!()
    }
}

impl Default for Icon {
    fn default() -> Self {
        unimplemented!()
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
}

impl Program {
    pub fn get_name(&self, prefix: &String) -> String {
        format!("{}_{}", prefix, self.name)
    }

    pub fn get_name_short(&self) -> String {
        self.name.to_owned()
    }

    pub fn new<T: Into<String>>(name: T, path: &Path, settings: &Vec<Feature>, icon: &Option<Icon>) -> Self {
        Program {
            name: name.into(),
            path: path.to_owned(),
            settings: settings.to_vec(),
            icon: icon.to_owned(),
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

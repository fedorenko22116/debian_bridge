use std::path::Path;
use std::process::{Command, Stdio};
use crate::app::error::AppError;
use std::ffi::OsStr;
use colorful::core::StrMarker;
use pipers::Pipe;
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub struct Deb {
    pub package: String,
    pub version: Option<String>,
    pub license: Option<String>,
    pub vendor: Option<String>,
    pub architecture: Option<String>,
    pub maintainer: Option<String>,
    pub installed_size: Option<String>,
    pub dependencies: Option<String>,
    pub section: Option<String>,
    pub priority: Option<String>,
    pub homepage: Option<String>,
    pub description: Option<String>,
}

impl Deb {
    pub fn try_new(path: &Path) -> Result<Self, AppError> {
        if !path.exists() || !path.extension().and_then(OsStr::to_str).eq(&Some("deb")) {
            return Err(AppError::File("Input application doesn't exist or is in incorrect format".to_str()));
        }

        let info = Pipe::new(format!("ar p {} control.tar.gz", path.to_str().unwrap()).as_str())
            .then("tar xzOf - ./control")
            .finally()
            .unwrap()
            .wait_with_output();

        let output = info
            .map(|o| (*String::from_utf8_lossy(&o.stdout)).to_owned())
            .map_err(|e| AppError::File(e.to_string()))?;

        Ok(
            Deb {
                package: Deb::parse_output(&output, "Package").unwrap(),
                version: Deb::parse_output(&output, "Version"),
                license: Deb::parse_output(&output, "License"),
                vendor: Deb::parse_output(&output, "Vendor"),
                architecture: Deb::parse_output(&output, "Architecture"),
                maintainer: Deb::parse_output(&output, "Maintainer"),
                installed_size: Deb::parse_output(&output, "Installed-Size"),
                dependencies: Deb::parse_output(&output, "Depends"),
                section: Deb::parse_output(&output, "Section"),
                priority: Deb::parse_output(&output, "Priority"),
                homepage: Deb::parse_output(&output, "Homepage"),
                description: Deb::parse_output(&output, "Description")
            }
        )
    }

    fn parse_output<T: Into<String>, S: Into<String>>(output: T, param: S) -> Option<String> {
        let pattern = Regex::new(format!(r"{}: (.*)\n", param.into()).as_str()).unwrap();

        for caps in pattern.captures_iter(&output.into()) {
            return Some(caps.get(1).unwrap().as_str().to_str())
        }

        None
    }
}

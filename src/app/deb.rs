use super::error::AppError;
use colorful::core::StrMarker;
#[cfg(test)]
use mocktopus::macros::*;
use pipers::Pipe;
use regex::Regex;
use std::{convert::TryInto, ffi::OsStr, path::Path};

#[derive(Debug, Clone, PartialEq)]
pub struct Dependencies {
    list: Vec<String>,
}

#[cfg_attr(test, mockable)]
impl Dependencies {
    pub fn new<T: Into<String>>(deps: T) -> Self {
        let deps = deps.into();

        Self {
            list: Self::parse(deps),
        }
    }

    fn parse(deps: String) -> Vec<String> {
        let deps: Vec<String> = deps.split(",").map(|dep| dep.to_owned()).collect();

        deps.iter()
            .map(|dep| {
                Self::split_first(
                    Self::split_first(dep.to_owned(), "|".to_string()),
                    "(".to_string(),
                )
            })
            .collect()
    }

    fn split_first(dep: String, pat: String) -> String {
        let parts: Vec<String> = dep.split(pat.as_str()).map(|dep| dep.to_owned()).collect();
        parts.get(0).unwrap().trim().to_owned()
    }

    pub fn extract(&self) -> String {
        self.list.join(" ")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Deb {
    pub package: String,
    pub version: Option<String>,
    pub license: Option<String>,
    pub vendor: Option<String>,
    pub architecture: Option<String>,
    pub maintainer: Option<String>,
    pub installed_size: Option<String>,
    pub dependencies: Option<Dependencies>,
    pub section: Option<String>,
    pub priority: Option<String>,
    pub homepage: Option<String>,
    pub description: Option<String>,
}

impl Deb {
    pub fn try_new(path: &Path) -> Result<Self, AppError> {
        if !path.exists() || !path.extension().and_then(OsStr::to_str).eq(&Some("deb")) {
            return Err(AppError::File(
                "Input application doesn't exist or is in incorrect format".to_str(),
            ));
        }

        let info = Pipe::new(format!("ar p {} control.tar.gz", path.to_str().unwrap()).as_str())
            .then("tar xzOf - ./control")
            .finally()
            .map_err(|err| {
                AppError::Program(format!("Can not parse a package: {}", err.to_string()))
            })?
            .wait_with_output();

        let output = info
            .map(|o| (*String::from_utf8_lossy(&o.stdout)).to_owned())
            .map_err(|e| AppError::File(e.to_string()))?;

        Ok(Deb {
            package: Deb::parse_output(&output, "Package").ok_or(AppError::Program(
                "Can not parse an input package".to_string(),
            ))?,
            version: Deb::parse_output(&output, "Version"),
            license: Deb::parse_output(&output, "License"),
            vendor: Deb::parse_output(&output, "Vendor"),
            architecture: Deb::parse_output(&output, "Architecture"),
            maintainer: Deb::parse_output(&output, "Maintainer"),
            installed_size: Deb::parse_output(&output, "Installed-Size"),
            dependencies: Deb::parse_output(&output, "Depends").map(|o| Dependencies::new(o)),
            section: Deb::parse_output(&output, "Section"),
            priority: Deb::parse_output(&output, "Priority"),
            homepage: Deb::parse_output(&output, "Homepage"),
            description: Deb::parse_output(&output, "Description"),
        })
    }

    fn parse_output<T: Into<String>, S: Into<String>>(output: T, param: S) -> Option<String> {
        let pattern = Regex::new(format!(r"{}: (.*)\n", param.into()).as_str()).unwrap();

        for caps in pattern.captures_iter(&output.into()) {
            return Some(caps.get(1).unwrap().as_str().to_str());
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dep_parses_success() {
        let deps = Dependencies::new(
            "git, libgconf-2-4 (>= 3.2.5) | libgconf2-4, libgtk-3-0 (>= 3.9.10),libgcrypt11 | \
             libgcrypt20, libnotify4, libxtst6, libnss3 (>= 2:3.22), python, gvfs-bin, xdg-utils, \
             libx11-xcb1, libxss1,libasound2 (>= 1.0.16), libxkbfile1, libcurl3 | libcurl4, \
             policykit-1",
        );

        assert_eq!(
            "git libgconf-2-4 libgtk-3-0 libgcrypt11 libnotify4 libxtst6 libnss3 python gvfs-bin \
             xdg-utils libx11-xcb1 libxss1 libasound2 libxkbfile1 libcurl3 policykit-1",
            deps.extract()
        );

        let deps = Dependencies::new("one_dep");

        assert_eq!("one_dep", deps.extract());
    }
}

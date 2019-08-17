use super::{deb::Deb, Program};
use dockerfile::{Cmd, Copy, Dockerfile, Env, Run, User, Workdir};
use freedesktop_desktop_entry::{Application, DesktopEntry, DesktopType};
use std::path::Path;

use crate::{app::error::AppError, Feature};
#[cfg(test)]
use mocktopus::macros::*;
use std::process::{Command, Stdio};

type AppResult<T> = Result<T, AppError>;

#[cfg_attr(test, mockable)]
fn get_user() -> Option<String> {
    std::env::var_os("USER")?
        .as_os_str()
        .to_str()
        .map(|s| s.to_string())
}

#[cfg_attr(test, mockable)]
fn get_package_path(package: &str) -> AppResult<String> {
    Ok(String::from_utf8(
        Command::new("which")
            .arg(package)
            .output()
            .map_err(|err| {
                AppError::Program("Program is not installed or can not be reached".into())
            })?
            .stdout,
    )
    .unwrap()
    .trim()
    .into())
}

#[cfg_attr(test, mockable)]
fn is_gnome_terminal() -> bool {
    Command::new("gnome-terminal")
        .arg("-h")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

pub fn gen_dockerfile(deb: &Deb, program: &Program) -> AppResult<String> {
    let mut dockerfile = Dockerfile::base("debian:9-slim")
        .push(Env::new(format!(
            "informuser={}",
            get_user().ok_or(AppError::Program("Can not find a current user".into()))?
        )))
        .push(Workdir::new("/data"))
        .push(Copy::new("tmp.deb /data/application.deb"))
        .push(Run::new("apt-get update"));

    if let Some(d) = &deb.dependencies {
        dockerfile = dockerfile.push(Run::new(format!(
            "apt-get install -y {}; exit 0",
            d.replace(&[','][..], "")
        )));
    }

    if let Some(d) = &program.deps {
        dockerfile = dockerfile.push(Run::new(format!("apt-get install -y {}", d)));
    }

    Ok(dockerfile
        .push(Run::new("dpkg -i /data/application.deb || true"))
        .push(Run::new(
            "apt-get install -y -f --no-install-recommends && rm -rf /var/lib/apt/lists/* && \
             useradd $informuser",
        ))
        .push(User::new("$informuser"))
        .push(Env::new("HOME /home/$informuser"))
        .push(Cmd::new(program.command.to_owned()))
        .finish()
        .to_string())
}

pub fn gen_desktop_entry<T: Into<String>, S: Into<String>>(
    name: T,
    description: S,
    icon: &Path,
) -> AppResult<String> {
    if !is_gnome_terminal() {
        return Err(AppError::Program(
            "Only gnome-terminal supported for now".into(),
        ));
    }

    let name = name.into();
    let package = env!("CARGO_PKG_NAME");
    let exec = format!(
        "gnome-terminal -e '{} run {}'",
        get_package_path(package)?,
        name
    );
    let description = description.into();

    Ok(DesktopEntry::new(
        &name,
        &icon.to_str().unwrap(),
        DesktopType::Application(
            Application::new(&["GNOME", "GTK"], exec.as_str()).keywords(&[name.as_str()]),
        ),
    )
    .comment(if description.is_empty() {
        "Empty description"
    } else {
        &description
    })
    .generic_name(&name)
    .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mocktopus::mocking::{MockResult, Mockable};

    #[rustfmt::skip::macros(assert_eq)]
    #[test]
    fn test_gen_dockerfile() {
        get_user.mock_safe(|| MockResult::Return(Some("user".to_string())));

        let dockerfile = gen_dockerfile(&get_deb(), &get_program()).unwrap();

        assert_eq!(
            dockerfile,
            "\
             FROM debian:9-slim\n\
             ENV informuser=user\n\
             WORKDIR /data\n\
             COPY tmp.deb /data/application.deb\n\
             RUN apt-get update\n\
             RUN apt-get install -y foo bar; exit 0\n\
             RUN apt-get install -y baz qux\n\
             RUN dpkg -i /data/application.deb || true\n\
             RUN apt-get install -y -f --no-install-recommends && rm -rf /var/lib/apt/lists/* && \
             useradd $informuser\nUSER $informuser\nENV HOME /home/$informuser\nCMD foobar\n"
        )
    }

    #[rustfmt::skip::macros(assert_eq)]
    #[test]
    fn test_gen_entrypoint() {
        get_package_path.mock_safe(|_| MockResult::Return(Ok("/foo".to_string())));
        is_gnome_terminal.mock_safe(|| MockResult::Return(true));

        let entrypoint = gen_desktop_entry("Foo", "bar", Path::new("")).unwrap();

        assert_eq!(
            entrypoint,
            "\
            [Desktop Entry]\n\
            Type=Application\n\
            Name=Foo\n\
            GenericName=Foo\n\
            X-GNOME-FullName=Foo Foo\n\
            Icon=\n\
            Comment=bar\n\
            Categories=GNOME;GTK\n\
            Keywords=\"Foo;\"\n\
            Exec=gnome-terminal -e \'/foo run Foo\'\n"
        )
    }

    fn get_program() -> Program {
        Program::new(
            "foobar".to_string(),
            Path::new(""),
            &vec![],
            &None,
            &None,
            &Some("baz qux".to_string()),
        )
    }

    fn get_deb() -> Deb {
        Deb {
            package: "".to_string(),
            version: None,
            license: None,
            vendor: None,
            architecture: None,
            maintainer: None,
            installed_size: None,
            dependencies: Some("foo, bar".to_string()),
            section: None,
            priority: None,
            homepage: None,
            description: None,
        }
    }
}

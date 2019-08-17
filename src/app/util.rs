use super::{deb::Deb, Program};
use dockerfile::{Cmd, Copy, Dockerfile, Env, Run, User, Workdir};
use freedesktop_desktop_entry::{Application, DesktopEntry, DesktopType};
use std::path::Path;

use crate::Feature;
#[cfg(test)]
use mocktopus::macros::*;

#[cfg_attr(test, mockable)]
fn get_user() -> String {
    match std::env::var_os("USER") {
        Some(os_string) => match os_string.as_os_str().to_str() {
            Some(user) => user.to_string(),
            _ => "default".to_string(),
        },
        _ => "default".to_string(),
    }
}

pub fn gen_dockerfile(deb: &Deb, program: &Program) -> String {
    let mut dockerfile = Dockerfile::base("debian:9-slim")
        .push(Env::new(format!("informuser={}", get_user())))
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

    return dockerfile
        .push(Run::new("dpkg -i /data/application.deb || true"))
        .push(Run::new(
            "apt-get install -y -f --no-install-recommends && rm -rf /var/lib/apt/lists/* && \
             useradd $informuser",
        ))
        .push(User::new("$informuser"))
        .push(Env::new("HOME /home/$informuser"))
        .push(Cmd::new(program.command.to_owned()))
        .finish()
        .to_string();
}

pub fn gen_desktop_entry<T: Into<String>, S: Into<String>>(
    name: T,
    description: S,
    icon: &Path,
) -> String {
    let name = name.into();
    let exec = format!("{} run {}", env!("CARGO_PKG_NAME"), name);

    DesktopEntry::new(
        &name,
        &icon.to_str().unwrap(),
        DesktopType::Application(
            Application::new(&["System", "GTK"], exec.as_str())
                .keywords(&[name.as_str()])
                .startup_notify(),
        ),
    )
    .comment(&description.into())
    .generic_name(&name)
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mocktopus::mocking::{MockResult, Mockable};

    #[test]
    fn test_gen_dockerfile() {
        get_user.mock_safe(|| MockResult::Return("user".to_string()));

        let dockerfile = gen_dockerfile(&get_deb(), &get_program());

        assert_eq!(
            dockerfile,
            "\
             FROM debian:9-slim\nENV informuser=user\nWORKDIR /data\nCOPY tmp.deb \
             /data/application.deb\nRUN apt-get update\nRUN apt-get install -y foo bar; exit \
             0\nRUN apt-get install -y baz qux\nRUN dpkg -i /data/application.deb || true\nRUN \
             apt-get install -y -f --no-install-recommends && rm -rf /var/lib/apt/lists/* && \
             useradd $informuser\nUSER $informuser\nENV HOME /home/$informuser\nCMD foobar\n"
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

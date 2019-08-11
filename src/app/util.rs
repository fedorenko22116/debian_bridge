use crate::app::deb::Deb;
use dockerfile::{Dockerfile, Arg, Copy, Cmd, Run, User, Env, Workdir};
use freedesktop_desktop_entry::{Application, DesktopEntry, DesktopType};
use std::path::Path;

fn get_user() -> String {
    match std::env::var_os("USER") {
        Some(os_string) =>  {
            match os_string.as_os_str().to_str() {
                Some(user) => user.to_string(),
                _ => "default".to_string(),
            }
        },
        _ => "default".to_string(),
    }
}

pub fn gen_dockerfile<T: Into<String>>(deb: &Deb, cmd: T) -> String {
    let cmd = cmd.into();
    let mut dockerfile = Dockerfile::base("debian:9-slim")
        .push(Env::new(format!("informuser={}", get_user())))
        .push(Workdir::new("/data"))
        .push(Copy::new("tmp.deb /data/application.deb"))
        .push(Run::new("apt-get update"));

    if let Some(d) = &deb.dependencies {
        dockerfile = dockerfile.push(Run::new(
            format!("apt-get install -y {}; exit 0", d.replace(&[','][..], "")))
        );
    }

    return dockerfile
        .push(Run::new("dpkg -i /data/application.deb || true"))
        .push(Run::new("apt-get install -y -f --no-install-recommends && rm -rf /var/lib/apt/lists/* && useradd $informuser"))
        .push(User::new("$informuser"))
        .push(Env::new("HOME /home/$informuser"))
        .push(Cmd::new(cmd))
        .finish()
        .to_string();
}

pub fn gen_desktop_entry<T: Into<String>, S: Into<String>>(name: T, description: S, icon: &Path) -> String {
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

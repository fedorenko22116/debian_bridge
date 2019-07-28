#[macro_use] extern crate clap;
#[macro_use] extern crate log;
extern crate pretty_env_logger;
extern crate xdg;

use debian_bridge::{App as Wrapper, Config, Program, Feature, System};
use clap::{App, AppSettings};
use std::path::Path;
use std::net::IpAddr;
use std::str::FromStr;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    if !cfg!(target_os = "linux") {
        return Err("Only linux supported for now.".into());
    }

    let yaml = load_yaml!("../config/cli.yaml");
    let matches = App::from_yaml(yaml)
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .name(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .get_matches();

    pretty_env_logger::init_custom_env(match matches.occurrences_of("v") {
        0 => "error",
        1 => "warn",
        2 => "info",
        3 => "debug",
        4 | _ => "trace",
    });

    let config_path = xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"))?
        .place_config_file("config.json")?;

    let docker = shiplift::Docker::new();
    let config = Config::deserialize(config_path.as_path())?;
    let system = System::try_new(&docker)?;
    let app = Wrapper::new(&config, &system, &docker);

    match matches.subcommand_name() {
        Some("test") => {
            println!("System settings: {}", system);
        }
        Some("create") => {
            let system = System::try_new(&docker);
            app.create(Path::new("./pcg.deb"))?;
        }
        Some("remove") => {
            app.remove("some_program")?
        }
        Some("list") => {
            app.list()?
        }
        _ => {
            unreachable!()
        }
    }

    app.save(&config_path)
}

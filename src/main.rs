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

    println!("{:?}", config_path);

    let docker = shiplift::Docker::new();
    let config = Config::new(&vec![
        Program {
            settings: vec![Feature::Display],
            path: Path::new("./").to_owned(),
        }
    ]);
    let app = Wrapper::new(&config, &docker);

    match matches.subcommand_name() {
        Some("test") => {
            let system = System::try_new(&docker)?;
            println!("System settings: {:?}", system.wm);
            Ok(())
        }
        Some("create") => {
            let system = System::try_new(&docker);
            app.create(Path::new("./pcg.deb"))?;
            app.config.serialize(&config_path)
        }
        Some("remove") => {
            app.remove(&Program::from_str("qwe")?)?;
            app.config.serialize(&config_path)
        }
        Some("list") => {
            app.list()?;
            Ok(())
        }
        Some("rpc") => {
            app.rpc(&IpAddr::from_str("127.0.0.1")?, &8080u16)?;
            Ok(())
        }
        _ => {
            Err("Command doesn't exist".into())
        }
    }
}

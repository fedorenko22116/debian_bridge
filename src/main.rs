#[macro_use] extern crate clap;
#[macro_use] extern crate log;

use debian_bridge::{App as Wrapper, Config, Program, Feature, System, Icon};
use clap::{App, AppSettings};
use std::path::Path;
use std::net::IpAddr;
use std::str::FromStr;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    if !cfg!(target_os = "linux") {
        return Err("Only linux supported for now.".into());
    }

    let package_name = env!("CARGO_PKG_NAME").to_owned();
    let yaml = load_yaml!("../config/cli.yaml");
    let matches = App::from_yaml(yaml)
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .name(&package_name)
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .get_matches();

    let debug_level = match matches.occurrences_of("verbose") {
        0 => "error",
        1 => "warn",
        2 => "info",
        3 => "debug",
        4 | _ => "trace",
    };

    std::env::set_var("RUST_APP_LOG", &debug_level);
    pretty_env_logger::init_custom_env("RUST_APP_LOG");

    info!("Logger configured: debug level: {}", debug_level);

    let config_path = xdg::BaseDirectories::with_prefix(&package_name)?
        .place_config_file("config.json")?;

    info!("Configuration path: {}", config_path.to_str().unwrap());

    let cache_path = xdg::BaseDirectories::with_prefix(&package_name)?
        .place_cache_file("")?;

    info!("Cache path: {}", cache_path.to_str().unwrap());

    let docker = shiplift::Docker::new();
    let config = Config::deserialize(config_path.as_path())?;
    let system = System::try_new(&docker)?;
    let mut app = Wrapper::new(&package_name, &cache_path, &config, &system, &docker);

    info!("Subcommand processing...");

    match matches.subcommand_name() {
        Some("test") => {
            println!("System settings: {}", system);
            println!("Available features: {}", app.features);
        },
        Some("create") => {
            app.create(Path::new(
                matches
                    .subcommand_matches("create").unwrap()
                    .value_of(&"package").unwrap()
            ), vec![Feature::Display], &None)?;
        },
        Some("run") => {
            app.run(
                matches
                    .subcommand_matches("run").unwrap()
                    .value_of(&"name").unwrap()
            )?;
        },
        Some("remove") => {
            app.remove(
                matches
                    .subcommand_matches("remove").unwrap()
                    .value_of(&"name").unwrap()
            )?;
            println!("Program successfuly removed");
        },
        Some("list") => {
            let list = app.list().join(", ");

            match list.as_str() {
                "" => println!("No program added yet"),
                list => println!("Available programs list: {}", list)
            }
        },
        _ => {
            unreachable!()
        },
    }

    info!("Subcommand processing finished");

    app.save(&config_path)?;

    info!("Exit");
    std::env::remove_var("RUST_APP_LOG");

    Ok(())
}

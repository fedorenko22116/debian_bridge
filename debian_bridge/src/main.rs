#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate xdg;

use clap::{App, AppSettings, ArgMatches};
use debian_bridge::CommandMatcher;
use debian_bridge_core::{App as Wrapper, Config, Docker, Feature, Icon, Program, System};
use std::{
    error::Error,
    net::IpAddr,
    path::{Path, PathBuf},
    str::FromStr,
};

fn main() -> Result<(), Box<dyn Error>> {
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
        0 => "info",
        1 => "debug",
        2 | _ => "trace",
    };

    std::env::set_var("RUST_APP_LOG", &debug_level);
    pretty_env_logger::init_custom_env("RUST_APP_LOG");

    debug!("Logger configured: debug level: {}", debug_level);

    let config_path = matches
        .value_of("config")
        .map(|c| std::fs::canonicalize(c).unwrap())
        .unwrap_or(
            xdg::BaseDirectories::with_prefix(&package_name)?.place_config_file("config.json")?,
        );

    debug!("Configuration path: {}", config_path.to_str().unwrap());

    let cache_path = xdg::BaseDirectories::with_prefix(&package_name)?.place_cache_file("")?;

    debug!("Cache path: {}", cache_path.to_str().unwrap());

    let matcher = CommandMatcher::new(&matches);
    let docker = Docker::new();
    let config = Config::deserialize(config_path.as_path())?;
    let system = System::try_new(&docker)?;
    let mut app = Wrapper::new(&package_name, &cache_path, &config, &system, &docker);

    debug!("Subcommand processing...");

    match matches.subcommand_name() {
        Some("test") => {
            println!("System settings: {}", system);
            println!("Available features: {}", app.features);
        }
        Some("create") => {
            app.create(
                get_create_package(&matcher)?.as_path(),
                &get_create_features(&matcher),
                &Some(Icon::default()),
                &get_create_command(&matcher),
                &get_create_deps(&matcher),
            )?;
            info!("Program successfuly created");
        }
        Some("run") => {
            app.run(
                matches
                    .subcommand_matches("run")
                    .unwrap()
                    .value_of(&"name")
                    .unwrap(),
            )?;
        }
        Some("remove") => {
            app.remove(
                matches
                    .subcommand_matches("remove")
                    .unwrap()
                    .value_of(&"name")
                    .unwrap(),
            )?;
            info!("Program successfuly removed");
        }
        Some("list") => {
            let list = app.list().join(", ");

            match list.as_str() {
                "" => println!("No program added yet"),
                list => println!("Available programs list: {}", list),
            }
        }
        _ => unreachable!(),
    }

    debug!("Subcommand processing finished");

    app.save(&config_path)?;

    debug!("Exit");

    std::env::remove_var("RUST_APP_LOG");

    Ok(())
}

fn get_create_features(matcher: &CommandMatcher) -> Vec<Feature> {
    let mut features = vec![];

    if matcher.is_option_present("create", "display") {
        features.push(Feature::Display);
    }

    if matcher.is_option_present("create", "sound") {
        features.push(Feature::Sound);
    }

    if matcher.is_option_present("create", "home") {
        features.push(Feature::HomePersistent);
    }

    if matcher.is_option_present("create", "notifications") {
        features.push(Feature::Notification);
    }

    if matcher.is_option_present("create", "timezone") {
        features.push(Feature::Time);
    }

    if matcher.is_option_present("create", "devices") {
        features.push(Feature::Devices);
    }

    features
}

fn get_create_package(matcher: &CommandMatcher) -> std::io::Result<PathBuf> {
    std::fs::canonicalize(Path::new(
        matcher.get_argument("create", "package").unwrap().as_str(),
    ))
}

fn get_create_command(matcher: &CommandMatcher) -> Option<String> {
    matcher.get_argument("create", "command")
}

fn get_create_deps(matcher: &CommandMatcher) -> Option<String> {
    matcher.get_argument("create", "dependencies")
}

#[macro_use] extern crate clap;
#[macro_use] extern crate log;
extern crate pretty_env_logger;

use debian_bridge::app::App as Wrapper;
use clap::{App, AppSettings};

fn main() {
    let yaml = load_yaml!("../config/cli.yaml");
    let matches = App::from_yaml(yaml)
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();

    pretty_env_logger::init_custom_env(match matches.occurrences_of("v") {
        0 => "error",
        1 => "warn",
        2 => "info",
        3 => "debug",
        4 | _ => "trace",
    });

    match matches.subcommand_name() {
        Some("test") => {

        }
        Some("create") => {

        }
        Some("remove") => {

        }
        Some("list") => {

        }
        _ => {
            error!("Command doesn't exist");
        }
    }
}

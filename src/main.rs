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

//    let config = matches.value_of("config").unwrap_or("default.conf");

//    println!("Value for config: {}", config);
//    println!("Using input file: {}", matches.value_of("INPUT").unwrap());

    let logger = match matches.occurrences_of("v") {
        0 => "error",
        1 => "warn",
        2 => "info",
        3 => "debug",
        4 | _ => "trace",
    };

    pretty_env_logger::init_custom_env(logger);

    if let Some(matches) = matches.subcommand_matches("test") {
        if matches.is_present("debug") {
            println!("Printing debug info...");
        } else {
            println!("Printing normally...");
        }

        return;
    }
}

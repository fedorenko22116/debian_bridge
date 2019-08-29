#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate xdg;

mod matcher;
mod starter;

pub use matcher::*;
pub use starter::start;

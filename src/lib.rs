#![cfg_attr(test, feature(proc_macro_hygiene))]
#[cfg(test)]
extern crate mocktopus;

#[macro_use]
pub extern crate log;

extern crate colorful;
extern crate dirs;
extern crate dockerfile;
extern crate freedesktop_desktop_entry;
extern crate pipers;
extern crate pretty_env_logger;
extern crate regex;
extern crate serde_json;
extern crate shiplift;
extern crate tokio;

mod app;
mod sys;

pub use app::*;
pub use shiplift::Docker;
pub use sys::System;

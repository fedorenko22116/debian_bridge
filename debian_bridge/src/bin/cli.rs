use debian_bridge_cli::start;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    start(env!("CARGO_PKG_NAME"))
}

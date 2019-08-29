use debian_bridge_cli::start;
use std::error::Error;

fn main() {
    start(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_AUTHORS"),
        env!("CARGO_PKG_VERSION"),
    )
}

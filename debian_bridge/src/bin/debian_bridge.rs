extern crate debian_bridge_cli;

use debian_bridge_cli::start;

fn main() {
    start(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_AUTHORS"),
        env!("CARGO_PKG_VERSION"),
    )
}

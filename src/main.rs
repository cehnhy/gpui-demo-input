use clap::Parser;
use gpui_demo_input::{app, bridge, cli};

fn main() {
    match cli::Cli::parse().command {
        Some(command) => bridge::client::run(command),
        None => app::run(),
    }
}

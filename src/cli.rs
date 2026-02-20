use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "gpui-demo-input")]
#[command(about = "GPUI Demo Input with Client/Server support", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Client {
        #[command(subcommand)]
        action: ClientAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum ClientAction {
    OpenLauncher {},
}

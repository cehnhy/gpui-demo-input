use tarpc::{client, context, tokio_serde::formats::Json};

use crate::cli::{ClientAction, Command};

use super::{OpenLauncherRequest, ServiceClient};

pub fn run(command: Command) {
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    rt.block_on(async {
        let socket_path = "/tmp/gpui-demo-input.sock";
        let transport = tarpc::serde_transport::unix::connect(socket_path, Json::default)
            .await
            .expect("failed to connect to server — is the launcher running?");

        let client = ServiceClient::new(client::Config::default(), transport).spawn();

        match command {
            Command::Client { action } => match action {
                ClientAction::OpenLauncher {} => {
                    client
                        .open_launcher(context::current(), OpenLauncherRequest {})
                        .await
                        .expect("RPC call failed")
                        .expect("open_launcher returned an error");
                }
            },
        }
    });
}

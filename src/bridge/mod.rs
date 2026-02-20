pub mod client;
pub mod event;
pub mod server;

use serde::{Deserialize, Serialize};

#[tarpc::service]
pub trait Service {
    async fn open_launcher(request: OpenLauncherRequest) -> Result<OpenLauncherResponse, ()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenLauncherRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenLauncherResponse {}

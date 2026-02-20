use flume::Sender;
use tarpc::context;

use super::{event::Event, OpenLauncherRequest, OpenLauncherResponse, Service};

#[derive(Clone)]
pub struct Server {
    tx: Sender<Event>,
}

impl Server {
    pub fn new(tx: Sender<Event>) -> Self {
        Self { tx }
    }
}

impl Service for Server {
    async fn open_launcher(
        self,
        _: context::Context,
        _request: OpenLauncherRequest,
    ) -> Result<OpenLauncherResponse, ()> {
        self.tx.send(Event::OpenLauncher {}).map_err(|_| ())?;
        Ok(OpenLauncherResponse {})
    }
}

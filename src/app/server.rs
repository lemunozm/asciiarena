use crate::server_manager::{ServerManager};
use crate::logger::{self};

pub fn run(_: Vec<String>) {
    logger::init(logger::Level::Trace); //Default info
    if let Some(mut server_manager) = ServerManager::new(3001, 3001) {
        server_manager.run();
    }
    else {
        log::error!("Could not run server on the specified ports"); //print ports
    }
}

use crate::server_manager::{ServerManager};

pub fn run(_: Vec<String>) {
    simple_logger::init_with_level(log::Level::Trace).unwrap();

    if let Some(mut server_manager) = ServerManager::new("127.0.0.1:3000".parse().unwrap()) {
        println!("Server running...");
        server_manager.run();
    }
    else {
        println!("Could not run server on the specified port");
    }
}

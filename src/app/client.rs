use crate::client_manager::{ClientManager};

pub fn run(_: Vec<String>) {
    simple_logger::init_with_level(log::Level::Trace).unwrap();

    if let Some(mut client_manager) = ClientManager::new("127.0.0.1:3000".parse().unwrap()) {
        println!("Connected to server");
        if let None = client_manager.run() {
            println!("Connection lost with the server");
        }
    }
    else {
        println!("Could not connect to server");
    }
}

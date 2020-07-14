use crate::client_manager::{ClientManager};
use crate::logger::{self};

pub fn run(_: Vec<String>) {
    logger::init(logger::Level::Trace);
    if let Some(mut client_manager) = ClientManager::new("127.0.0.1:3001".parse().unwrap()) {
        println!("Connected to server");
        if let None = client_manager.run() {
            println!("Connection lost with the server");
        }
    }
    else {
        println!("Could not connect to server");
    }
}

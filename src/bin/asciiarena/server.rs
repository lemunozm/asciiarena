use asciiarena::events::{self};
use asciiarena::message::{Message};
use asciiarena::network_manager::{NetworkManager, Endpoint};
use asciiarena::server_manager::{ServerManager, Signal};

pub fn run(_: Vec<String>) {
    let (event_queue, message_handle) = events::new_event_system::<Message, Signal, Endpoint>();

    let address = "0.0.0.0:3000".parse().unwrap();
    match NetworkManager::listen(address, message_handle) {
        Some(_) => {
            println!("Running server...");
            ServerManager::new(event_queue).run();
        },
        None => {
            println!("Can not start the server");
        }
    }
}

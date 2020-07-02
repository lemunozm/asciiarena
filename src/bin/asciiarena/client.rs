use asciiarena::events::{self};
use asciiarena::message::{Message};
use asciiarena::network_manager::{NetworkManager, Endpoint};
use asciiarena::client_manager::{ClientManager};

pub fn run(_: Vec<String>) {
    let (event_queue, message_handle) = events::new_event_system::<Message, (), Endpoint>();

    let address = "127.0.0.1:3000".parse().unwrap();
    match NetworkManager::connect(address, message_handle) {
        Some(network) => {
            println!("Connected to server!");
            ClientManager::new(event_queue, network.endpoint()).run();
        },
        None => {
            println!("Can not connect to the server");
        }
    }
}

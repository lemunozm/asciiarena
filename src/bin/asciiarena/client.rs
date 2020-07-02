use asciiarena::events::{self};
use asciiarena::message::{Message};
use asciiarena::network_manager::{NetworkManager, NetworkRole, Endpoint};
use asciiarena::client_manager::{ClientManager};

pub fn run(_: Vec<String>) {
    let (event_queue, message_input, message_output) = events::new_event_system::<Message, (), Endpoint>();

    let mut network_manager = NetworkManager::new(message_input, message_output);
    match network_manager.run(NetworkRole::Client("127.0.0.1:3000".parse().unwrap())) {
        Some(endpoint) => {
            println!("Connected to server!");
            ClientManager::new(event_queue, endpoint).run();
        },
        None => {
            println!("Can not connect to the server");
        }
    }
}

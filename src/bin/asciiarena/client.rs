use asciiarena::events::{self, Endpoint};
use asciiarena::message::{Message};
use asciiarena::network_manager::{NetworkManager, NetworkRole};
use asciiarena::client_manager::{ClientManager};

pub fn run(args: Vec<String>) {

    println!("Client initialized");

    let (event_queue, message_input, message_output) = events::new_event_system::<Message, ()>();

    let mut network_manager = NetworkManager::new(message_input, message_output);
    network_manager.run(NetworkRole::Client("127.0.0.1:3000".parse().unwrap()));

    let mut client_manager = ClientManager::new(event_queue, Endpoint {});
    client_manager.run();
}

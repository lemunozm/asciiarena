use asciiarena::events::{self};
use asciiarena::message::{Message};
use asciiarena::network_manager::{NetworkManager, NetworkRole, Endpoint};
use asciiarena::server_manager::{ServerManager, Signal};

pub fn run(_: Vec<String>) {
    let (event_queue, message_input, message_output) = events::new_event_system::<Message, Signal, Endpoint>();

    let mut network_manager = NetworkManager::new(message_input, message_output);
    match network_manager.run(NetworkRole::Server("0.0.0.0:3000".parse().unwrap())) {
        Some(_) => {
            println!("Running server...");
            ServerManager::new(event_queue).run();
        },
        None => {
            println!("Can not start the server");
        }
    }
}

use asciiarena::events::{self};
use asciiarena::message::{Message};
use asciiarena::network_manager::{NetworkManager, NetworkRole};
use asciiarena::server_manager::{ServerManager, Signal};

pub fn run(args: Vec<String>) {
    let (event_queue, message_input, message_output) = events::new_event_system::<Message, Signal>();

    let mut network_manager = NetworkManager::new(message_input, message_output);
    network_manager.run(NetworkRole::Server("0.0.0.0:3000".parse().unwrap()));

    let mut server_manager = ServerManager::new(event_queue);
    server_manager.run();

    /*
    let output_thread = thread::spawn(move || {
        loop {
            if let Some((message, endpoints)) = message_output.pop(Duration::from_millis(50)) {
                let data: Vec<u8> = bincode::serialize(&message).unwrap();
                for endpoint in endpoints {
                    if let Some(connection) = network.get(endpoint) {
                        connection.tcp_stream.write(data);
                    }
                }
            }
        }
    });

    let input_thread = thread::spawn(move || {

    });

*/

    /*
    let callbacks = network::Callbacks {
        on_connection: |connection| {
        },
        on_disconnection: |connection| {
            event_io.push(Message::Disconnect, connection);
        },
        on_input_data: |connection| {
            let data = connection.read();
            let message: Message = bincode::deserialize(&data[..]).unwrap();
            event_io.push(message, connection);
        },
        idle: || {
        },
    };

    network::listen("127.0.0.1:3000".parse().unwrap(), callbacks);
    */
}

/*
impl<'a, Event> EventIO<Event>
where Event: Serialize + Deserialize<'a> {
*/


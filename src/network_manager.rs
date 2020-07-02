use crate::events::{MessageHandle};
use crate::network::{self, Connection};

use serde::{Serialize, Deserialize};

use std::net::SocketAddr;
use std::time::Duration;

use std::thread::{self, JoinHandle};

pub type Endpoint = usize;

pub struct NetworkManager {
    input_thread: Option<JoinHandle<()>>,
    output_thread: Option<JoinHandle<()>>,
    endpoint: Endpoint,
}

impl NetworkManager
{
    pub fn listen<'a, M>(addr: SocketAddr, message_handle: MessageHandle<M, Endpoint>) -> Option<NetworkManager>
    where M: Serialize + Deserialize<'a> + Send + 'static {
        let MessageHandle {input_message_handle, mut output_message_handle} = message_handle;

        let (mut input_network, mut output_network, endpoint) = network::listen(addr);

        let input_thread = thread::spawn(move || {
            let mut input_message_handle_data = input_message_handle.clone();
            let mut input_message_handle_disc = input_message_handle.clone();
            let network_callbacks = network::Callbacks {
                on_connection: |connection: Connection| {
                },
                on_data: |connection: Connection, data: &'a[u8], size: usize| {
                    let message: M = bincode::deserialize(&data[..]).unwrap();
                    input_message_handle_data.push(message, connection.id());
                },
                on_disconnection: |connection: Connection| {
                    input_message_handle_disc.notify_lost_endpoint(connection.id());
                },
            };
            input_network.run(network_callbacks);
        });

        let output_thread = thread::spawn(move || {
            loop {
                if let Some((message, connection_ids)) = output_message_handle.pop(Duration::from_millis(50)) {
                    let data: Vec<u8> = bincode::serialize(&message).unwrap();
                    output_network.send_all(connection_ids, &data[0..]);
                }
            }
        });

        Some(NetworkManager {
            input_thread: Some(input_thread),
            output_thread: Some(output_thread),
            endpoint: endpoint.unwrap(),
        })
    }

    pub fn connect<'a, M>(addr: SocketAddr, message_handle: MessageHandle<M, Endpoint>) -> Option<NetworkManager>
    where M: Serialize + Deserialize<'a> + Send + 'static {
        None
    }

    pub fn endpoint(&self) -> Endpoint {
       self.endpoint
    }
}


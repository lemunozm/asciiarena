use crate::events::{InputMessageHandle, OutputMessageHandle};
use crate::message::{Message};
use crate::network::{self};

use serde::{Serialize, Deserialize};

use std::net::SocketAddr;
use std::time::Duration;

use std::thread::{self};


pub enum NetworkRole {
    Server(SocketAddr),
    Client(SocketAddr),
}

pub type Endpoint = usize;

pub struct NetworkManager<M> {
    message_input_handle: InputMessageHandle<M, Endpoint>,
    message_output_handle: OutputMessageHandle<M, Endpoint>,
}

impl<'a, M: Serialize + Deserialize<'a>> NetworkManager<M>
{
    pub fn new(message_input_handle: InputMessageHandle<M, Endpoint>, message_output_handle: OutputMessageHandle<M, Endpoint>) -> NetworkManager<M> {
        NetworkManager {
            message_input_handle,
            message_output_handle,
        }
    }

    pub fn run(&mut self, role: NetworkRole) -> Option<Endpoint> {
        None
        /*
        // Input
        let network_callbacks = network::Callbacks {
            on_connection: |connection| {
            },
            on_disconnection: |connection| {
                self.message_input_handle.notify_lost_endpoint(connection.id());
            },
            on_input_data: |connection| {
                let data = connection.read();
                let message: Message = bincode::deserialize(&data[..]).unwrap();
                self.message_input_handle.push(message, connection);
            },
        };
        let network = match role {
            Client(addr) => network::connect(addr, network_callbacks)
            Server(addr) => network::listen(addr, network_callbacks)
        };

        // Output
        let output_thread = thread::spawn(|| {
            loop {
                if let Some((message, connection_ids)) = self.message_output_handle.pop(Duration::from_millis(50)) {
                    let data: Vec<u8> = bincode::serialize(&message).unwrap();
                    for id in connection_ids {
                        if let Some(connection) = network.connections().get(id) {
                            connection.tcp_stream.write(data);
                        }
                    }
                }
            }
        }
        */
    }
}


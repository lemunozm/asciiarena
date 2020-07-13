use crate::message::{ClientMessage, ServerMessage};

use message_io::events::{EventQueue};
use message_io::network::{NetworkManager, NetEvent, TransportProtocol, Endpoint};

use std::net::{SocketAddr};


pub enum Event {
    Network(NetEvent<ClientMessage>),
}

pub struct ServerManager {
    event_queue: EventQueue<Event>,
    network: NetworkManager,
    //room here
}

impl ServerManager {
    pub fn new(tcp_port: u16, udp_port: u16) -> Option<ServerManager> {
        let mut event_queue = EventQueue::new();

        let network_sender = event_queue.sender().clone();
        let mut network = NetworkManager::new(move |net_event| network_sender.send(Event::Network(net_event)));

        let tcp_address = SocketAddr::from(([0, 0, 0, 0], tcp_port));
        let tcp_listener = network.listen(tcp_address, TransportProtocol::Tcp);

        let udp_address = SocketAddr::from(([0, 0, 0, 0], udp_port));
        let udp_listener = network.listen(udp_address, TransportProtocol::Udp);

        tcp_listener.and(udp_listener).map(|_| {
            ServerManager {
                event_queue,
                network,
            }
        })
    }

    pub fn run(&mut self) {
        loop {
            match self.event_queue.receive() {
                Event::Network(net_event) => match net_event {
                    NetEvent::Message(message, endpoint) => {
                        log::trace!("Client message: {:?}", message);
                        println!("aaaaaa");
                        match message {
                            ClientMessage::Version{tag} => {
                                self.network.send(endpoint, ServerMessage::Version{tag: String::from("0.1.0"), compatible: true});
                            }
                        }
                    },
                    NetEvent::AddedEndpoint(_, _) => (),
                    NetEvent::RemovedEndpoint(_) => {}
                }
            }
        }
    }
}

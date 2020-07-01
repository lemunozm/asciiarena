use crate::events::{InputMessageHandle, OutputMessageHandle};

use mio::net::{TcpListener, TcpStream};
use mio::{Poll, Interest, Token, Events};

use std::net::SocketAddr;
use std::collections::HashMap;
use std::time::Duration;
use std::io::ErrorKind;

pub enum NetworkRole {
    Server(SocketAddr),
    Client(SocketAddr),
}

pub struct NetworkManager<M> {
    message_input: InputMessageHandle<M>,
    message_output: OutputMessageHandle<M>,
}

impl<M> NetworkManager<M> {
    pub fn new(message_input: InputMessageHandle<M>, message_output: OutputMessageHandle<M>) -> NetworkManager<M> {
        NetworkManager { message_input, message_output }
    }

    pub fn run(&mut self, role: NetworkRole) {
        //TODO
    }
}

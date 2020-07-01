use mio::net::{TcpListener, TcpStream};
use mio::{Poll, Interest, Token, Events};

use std::net::SocketAddr;
use std::collections::HashMap;
use std::time::Duration;
use std::io::ErrorKind;

/*
pub struct Callbacks<A, B, C> {
    pub on_connection: A,
    pub on_disconnection: B,
    pub on_data: C,
}

pub struct Connection {
    pub tcp_stream: TcpStream,
    pub input_buffer: Vec<u8>,
    pub id: usize,
}

pub fn run<OnConnection, OnDisconnection, OnData>(addr: SocketAddr, callbacks: Callbacks<OnConnection, OnDisconnection, OnData>)
    where
    OnConnection: Fn(&Connection),
    OnDisconnection: Fn(&Connection),
    OnData: Fn(&Connection),
{
    const EVENTS_SIZE: usize = 128;
    const INITIAL_BUFFER_SIZE: usize = 1024;
    const SERVER: Token = Token(0);

    let mut connections = HashMap::new();
    let mut connections_count = 0;
    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(EVENTS_SIZE);
    let mut listener = TcpListener::bind(addr).unwrap();

    poll.registry().register(&mut listener, SERVER, Interest::READABLE).unwrap();

    loop {
        match poll.poll(&mut events, Some(Duration::from_millis(0))) {
            Ok(size) => {
                for event in events.iter() {
                    match event.token() {
                        SERVER => {
                            connections_count += 1;
                            let connection = Connection {
                                tcp_stream: listener.accept().unwrap().0,
                                input_buffer: Vec::with_capacity(INITIAL_BUFFER_SIZE),
                                id: connections_count
                            };
                            (callbacks.on_connection)(&connection);
                            connections.insert(connection.id, connection);
                        },
                        connection_id => {
                            if let Some(connection) = connections.get(&connection_id.0) {
                                (callbacks.on_data)(connection);
                            }
                        }
                    }
                }
            },
            Err(ref err) if err.kind() == ErrorKind::TimedOut => continue,
            error => error.unwrap()
        }
    }
}
*/

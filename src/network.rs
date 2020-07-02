use mio::net::{TcpListener, TcpStream};
use mio::{Poll, Interest, Token, Events};

use std::thread::{JoinHandle};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::time::Duration;
use std::io::ErrorKind;

pub struct Callbacks<A, B, C> {
    pub on_connection: A,
    pub on_data: B,
    pub on_disconnection: C,
}

pub struct Connection {
    tcp_stream: TcpStream,
    handle: JoinHandle<()>,
    input_buffer: Vec<u8>,
    id: usize,
}

impl Connection {
    pub fn id(&self) -> usize {
        return self.id
    }
}

pub fn connect(addr: SocketAddr) -> (InputNetwork, OutputNetwork, Option<usize>) {
    (InputNetwork::new(), OutputNetwork::new(), Some(0))
}

pub fn listen(addr: SocketAddr) -> (InputNetwork, OutputNetwork, Option<usize>) {
    let mut listener = TcpListener::bind(addr).unwrap();
    (InputNetwork::new(), OutputNetwork::new(), Some(0))
}

pub struct InputNetwork {
    connections: HashMap<usize, Connection>,
    id: usize,
}

impl InputNetwork {
    pub fn new() -> InputNetwork {
        InputNetwork {
            connections: HashMap::new(),
            id: 0,
        }
    }

    pub fn run<A, B, C>(&mut self, callbacks: Callbacks<A, B, C>) {
        loop {
        }
    }
}

pub struct OutputNetwork {
}

impl OutputNetwork {
    pub fn new() -> OutputNetwork {
        OutputNetwork {}
    }

    pub fn send(&mut self, id: usize, data: &[u8]) {
    }

    pub fn send_all(&mut self, ids: Vec<usize>, data: &[u8]) {
    }
}

/*

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

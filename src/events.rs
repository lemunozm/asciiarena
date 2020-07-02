use crossbeam_channel::{self, Sender, Receiver};

use std::time::Duration;
use std::thread::{self, JoinHandle};
use std::hash::{Hash};

pub enum Event<M, S, E> {
    Message(M, E),
    Signal(S),
    LostEndpoint(E),
    Idle,
}

pub struct MessageHandle<M, E> {
    pub input_message_handle: InputMessageHandle<M, E>,
    pub output_message_handle: OutputMessageHandle<M, E>,
}

pub fn new_event_system<M, S: Send + 'static, E: Hash + Copy>() -> (EventQueue<M, S, E>, MessageHandle<M, E>) {
    let (msg_input_sender, msg_input_receiver) = crossbeam_channel::unbounded();
    let (endpoint_input_sender, endpoint_input_receiver) = crossbeam_channel::unbounded();
    let (msg_output_sender, msg_output_receiver) = crossbeam_channel::unbounded();

    (
        EventQueue::new(msg_input_receiver, endpoint_input_receiver, msg_output_sender),
        MessageHandle {
            input_message_handle: InputMessageHandle::new(msg_input_sender, endpoint_input_sender),
            output_message_handle: OutputMessageHandle::new(msg_output_receiver),
        },
    )
}

pub struct EventQueue<M, S, E> {
    signal_input_sender: Sender<S>,
    signal_input_receiver: Receiver<S>,
    timer_input_sender: Sender<(S, usize)>,
    timer_input_receiver: Receiver<(S, usize)>,
    msg_input_receiver: Receiver<(M, E)>,
    endpoint_input_receiver: Receiver<E>,
    msg_output_sender: Sender<(M, Vec<E>)>,
    timers: Vec<JoinHandle<()>>,
}

impl<M, S: Send + 'static, E: Hash + Copy> EventQueue<M, S, E>
{
    fn new(msg_input_receiver: Receiver<(M, E)>,
           endpoint_input_receiver: Receiver<E>,
           msg_output_sender: Sender<(M, Vec<E>)>,
           ) -> EventQueue<M, S, E>
    {
        let (signal_input_sender, signal_input_receiver) = crossbeam_channel::unbounded();
        let (timer_input_sender, timer_input_receiver) = crossbeam_channel::unbounded();
        EventQueue {
            signal_input_sender,
            signal_input_receiver,
            timer_input_sender,
            timer_input_receiver,
            msg_input_receiver,
            endpoint_input_receiver,
            msg_output_sender,
            timers: Vec::new(),
        }
    }

    pub fn emit_message(&mut self, message: M, endpoint: E) {
        self.msg_output_sender.send((message, vec![endpoint])).unwrap();
    }

    pub fn emit_message_all(&mut self, message: M, endpoints: Vec<E>) {
        self.msg_output_sender.send((message, endpoints)).unwrap();
    }

    pub fn push_signal(&mut self, signal: S) {
        self.signal_input_sender.send(signal).unwrap();
    }

    pub fn push_timed_signal(&mut self, signal: S, timeout: Duration) {
        let timer_input_sender = self.timer_input_sender.clone();
        let timer_list_index = self.timers.len();
        let timer = thread::spawn(move || {
            thread::sleep(timeout);
            timer_input_sender.send((signal, timer_list_index)).unwrap();
        });
        self.timers.push(timer);
    }

    pub fn pop_event(&mut self, timeout: Duration) -> Option<Event<M, S, E>> {
        crossbeam_channel::select! {
            recv(self.signal_input_receiver) -> signal => {
                Some(Event::Signal(signal.unwrap()))
            },
            recv(self.timer_input_receiver) -> signal_index => {
                let (signal, index) = signal_index.unwrap();
                self.timers.remove(index);
                Some(Event::Signal(signal))
            },
            recv(self.msg_input_receiver) -> msg_endpoint => {
                let (mensage, endpoint) = msg_endpoint.unwrap();
                Some(Event::Message(mensage, endpoint))
            },
            recv(self.endpoint_input_receiver) -> endpoint => {
                Some(Event::LostEndpoint(endpoint.unwrap()))
            },
            default(timeout) => None
        }
    }
}

impl<M, S, E> Drop for EventQueue<M, S, E> {
    fn drop(&mut self) {
        while self.timers.len() > 0 {
            self.timers.remove(0).join().unwrap();
        }
    }
}


pub struct InputMessageHandle<M, E> {
    msg_input_sender: Sender<(M, E)>,
    endpoint_input_sender: Sender<E>,
}

impl<M, E> InputMessageHandle<M, E> {
    fn new(msg_input_sender: Sender<(M, E)>, endpoint_input_sender: Sender<E>) -> InputMessageHandle<M, E> {
        InputMessageHandle { msg_input_sender, endpoint_input_sender }
    }

    pub fn push(&mut self, message: M, endpoint: E) {
        self.msg_input_sender.send((message, endpoint)).unwrap();
    }

    pub fn notify_lost_endpoint(&mut self, endpoint: E) {
        self.endpoint_input_sender.send(endpoint).unwrap();
    }
}

impl<M, E> Clone for InputMessageHandle<M, E> {
    fn clone(&self) -> Self {
        Self {
            msg_input_sender: self.msg_input_sender.clone(),
            endpoint_input_sender: self.endpoint_input_sender.clone(),
        }
    }
}


pub struct OutputMessageHandle<M, E> {
    output_receiver: Receiver<(M, Vec<E>)>,
}

impl<M, E> OutputMessageHandle<M, E> {
    fn new(output_receiver: Receiver<(M, Vec<E>)>) -> OutputMessageHandle<M, E> {
        OutputMessageHandle { output_receiver }
    }

    pub fn pop(&mut self, timeout: Duration) -> Option<(M, Vec<E>)> {
        self.output_receiver.recv_timeout(timeout).ok()
    }
}

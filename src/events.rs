use crossbeam_channel::{self, Sender, Receiver};

use std::time::Duration;
use std::thread::{self};

pub enum Event<Message, Signal> {
    Message(Message),
    Signal(Signal),
    LostEndpoint,
    Idle,
}


#[derive(Hash, Clone, Copy)]
pub struct Endpoint {

}

pub fn new_event_system<M, S: Send + 'static>() -> (EventQueue<M, S>, InputMessageHandle<M>, OutputMessageHandle<M>) {
    let (msg_input_sender, msg_input_receiver) = crossbeam_channel::unbounded();
    let (endpoint_input_sender, endpoint_input_receiver) = crossbeam_channel::unbounded();
    let (msg_output_sender, msg_output_receiver) = crossbeam_channel::unbounded();

    let event_queue = EventQueue::new(msg_input_receiver, endpoint_input_receiver, msg_output_sender);
    let message_input = InputMessageHandle::new(msg_input_sender, endpoint_input_sender);
    let message_output = OutputMessageHandle::new(msg_output_receiver);

    (event_queue, message_input, message_output)
}

pub struct EventQueue<M, S> {
    signal_input_sender: Sender<S>,
    signal_input_receiver: Receiver<S>,
    timer_input_sender: Sender<(S, usize)>,
    timer_input_receiver: Receiver<(S, usize)>,
    msg_input_receiver: Receiver<(M, Endpoint)>,
    endpoint_input_receiver: Receiver<Endpoint>,
    msg_output_sender: Sender<(M, Vec<Endpoint>)>,
    timers: Vec<thread::JoinHandle<()>>,
    selfpoint: Endpoint,
}

impl<M, S: Send + 'static> EventQueue<M, S>
{
    fn new(msg_input_receiver: Receiver<(M, Endpoint)>,
           endpoint_input_receiver: Receiver<Endpoint>,
           msg_output_sender: Sender<(M, Vec<Endpoint>)>
           ) -> EventQueue<M, S>
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
            selfpoint: Endpoint {} }
    }

    pub fn emit_message(&mut self, message: M, endpoint: Endpoint) {
        self.msg_output_sender.send((message, vec![endpoint])).unwrap();
    }

    pub fn emit_message_all(&mut self, message: M, endpoints: Vec<Endpoint>) {
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

    pub fn pop_event(&mut self, timeout: Duration) -> Option<(Event<M, S>, Endpoint)> {
        crossbeam_channel::select! {
            recv(self.signal_input_receiver) -> signal => {
                Some((Event::Signal(signal.unwrap()), self.selfpoint))
            },
            recv(self.timer_input_receiver) -> signal_index => {
                let (signal, index) = signal_index.unwrap();
                self.timers.remove(index);
                Some((Event::Signal(signal), self.selfpoint))
            },
            recv(self.msg_input_receiver) -> msg_endpoint => {
                let (mensage, endpoint) = msg_endpoint.unwrap();
                Some((Event::Message(mensage), endpoint))
            },
            recv(self.endpoint_input_receiver) -> endpoint => {
                Some((Event::LostEndpoint, endpoint.unwrap()))
            },
            default(timeout) => None
        }
    }
}

impl<M, S> Drop for EventQueue<M, S> {
    fn drop(&mut self) {
        while self.timers.len() > 0 {
            self.timers.remove(0).join().unwrap();
        }
    }
}


pub struct InputMessageHandle<M> {
    msg_input_sender: Sender<(M, Endpoint)>,
    endpoint_input_sender: Sender<Endpoint>,
}

impl<M> InputMessageHandle<M> {
    fn new(msg_input_sender: Sender<(M, Endpoint)>, endpoint_input_sender: Sender<Endpoint>) -> InputMessageHandle<M> {
        InputMessageHandle { msg_input_sender, endpoint_input_sender }
    }

    pub fn push(&mut self, message: M, endpoint: Endpoint) {
        self.msg_input_sender.send((message, endpoint)).unwrap();
    }

    pub fn notify_lost_endpoint(&mut self, endpoint: Endpoint) {
        self.endpoint_input_sender.send(endpoint).unwrap();
    }
}


pub struct OutputMessageHandle<M> {
    output_receiver: Receiver<(M, Vec<Endpoint>)>,
}

impl<M> OutputMessageHandle<M> {
    fn new(output_receiver: Receiver<(M, Vec<Endpoint>)>) -> OutputMessageHandle<M> {
        OutputMessageHandle { output_receiver }
    }

    pub fn pop(&mut self, timeout: Duration) -> Option<(M, Vec<Endpoint>)> {
        self.output_receiver.recv_timeout(timeout).ok()
    }
}

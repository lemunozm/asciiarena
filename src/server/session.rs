use crate::util::{SessionToken};

use rand::prelude::*;

use std::collections::HashMap;

pub enum SessionCreationResult {
    Created(SessionToken),
    Recycled(SessionToken),
    AlreadyLogged,
    Full,
}

pub struct Room<E> {
    sessions: HashMap<SessionToken, PlayerSession<E>>,
    size: usize,
}

impl<E> Room<E>
where E: Eq
{
    pub fn new(size: usize) -> Room<E> {
        Room {
            sessions: HashMap::new(),
            size,
        }
    }

    pub fn clear(&mut self) {
        self.sessions.clear();
    }

    pub fn is_full(&self) -> bool {
        self.sessions.len() == self.size
    }

    pub fn sessions(&self) -> impl Iterator<Item = &PlayerSession<E>> {
        self.sessions.values()
    }

    pub fn session(&self, token: SessionToken) -> Option<&PlayerSession<E>> {
        self.sessions.get(&token)
    }

    pub fn session_mut(&mut self, token: SessionToken) -> Option<&mut PlayerSession<E>> {
        self.sessions.get_mut(&token)
    }

    pub fn remove_session_by_endpoint(&mut self, endpoint: E) -> Option<PlayerSession<E>> {
        if let Some(session) = self.session_by_endpoint(endpoint) {
            let token = session.token();
            self.sessions.remove(&token)
        }
        else { None }
    }

    pub fn session_by_endpoint(&self, safe_endpoint: E) -> Option<&PlayerSession<E>> {
        self.sessions.values().find(|session| {
            match session.safe_endpoint() {
                Some(endpoint) if *endpoint == safe_endpoint => true,
                _ => false,
            }
        })
    }

    pub fn session_by_endpoint_mut(&mut self, safe_endpoint: E) -> Option<&mut PlayerSession<E>> {
        self.sessions.values_mut().find(|session| {
            match session.safe_endpoint() {
                Some(endpoint) if *endpoint == safe_endpoint => true,
                _ => false,
            }
        })
    }

    pub fn create_session(&mut self, name: &str, safe_endpoint: E) -> SessionCreationResult {
        if let Some(session) = self.sessions.values().find(|session| session.name() == name) {
            if session.safe_endpoint().is_some() {
                SessionCreationResult::AlreadyLogged
            }
            else {
                let token = session.token();
                self.sessions.remove(&token);
                let new_token = self.generate_unique_token();
                self.sessions.insert(token, PlayerSession::new(token, name, safe_endpoint));
                SessionCreationResult::Recycled(new_token)
            }
        }
        else if self.is_full() {
            SessionCreationResult::Full
        }
        else {
            let new_token = self.generate_unique_token();
            self.sessions.insert(new_token, PlayerSession::new(new_token, name, safe_endpoint));
            SessionCreationResult::Created(new_token)
        }
    }

    pub fn safe_endpoints(&self) -> impl Iterator<Item = &E> {
        self.sessions()
            .map(|session| session.safe_endpoint().as_ref())
            .filter_map(|e| e) // Only connected endpoints
    }

    /// Try to return the fast endpoint, if this is not possible, the safe endpoint is returned.
    pub fn faster_endpoints(&self) -> impl Iterator<Item = &E> {
        self.sessions()
            .map(|session| {
                match session.trusted_fast_endpoint() {
                    Some(fast_endpoint) => Some(fast_endpoint),
                    None => session.safe_endpoint().as_ref(),
                }
            })
            .filter_map(|e| e) // Only connected endpoints
    }

    fn generate_unique_token(&self) -> SessionToken {
        loop {
            let mut rng = rand::thread_rng();
            let token: SessionToken = rng.gen();
            if !self.sessions.contains_key(&token) {
                break token;
            }
        }
    }
}

pub struct PlayerSession<E> {
    token: SessionToken,
    name: String,
    safe_endpoint: Option<E>,
    fast_endpoint: Option<E>,
    is_fast_endpoint_trusted: bool,
}

impl<E> PlayerSession<E> {
    fn new(token: SessionToken, name: &str, safe_endpoint: E) -> PlayerSession<E> {
        PlayerSession {
            token,
            name: name.into(),
            safe_endpoint: Some(safe_endpoint),
            fast_endpoint: None,
            is_fast_endpoint_trusted: false,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn token(&self) -> SessionToken {
        self.token
    }

    pub fn safe_endpoint(&self) -> &Option<E> {
        &self.safe_endpoint
    }

    pub fn trusted_fast_endpoint(&self) -> &Option<E> {
        match self.is_fast_endpoint_trusted {
            true => &self.fast_endpoint,
            false => &None,
        }
    }

    pub fn set_untrusted_fast_endpoint(&mut self, endpoint: E) {
        self.fast_endpoint = Some(endpoint);
        self.is_fast_endpoint_trusted = false;
    }

    pub fn trust_in_fast_endpoint(&mut self) -> &Option<E> {
        if self.fast_endpoint.is_some() {
            self.is_fast_endpoint_trusted = true;
        }
        &self.fast_endpoint
    }

    pub fn disconnect(&mut self) {
        self.safe_endpoint = None;
        self.fast_endpoint = None;
        self.is_fast_endpoint_trusted = false;
    }
}


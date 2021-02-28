use crate::ids::{SessionToken};

use rand::prelude::*;

use message_io::network::{Endpoint};

use std::collections::HashMap;

pub enum SessionStatus {
    Created(SessionToken),
    Recycled(SessionToken),
    AlreadyLogged,
    Full,
}

pub struct RoomSession<U> {
    sessions: HashMap<SessionToken, Session<U>>,
    size: usize,
}

impl<U: Eq> RoomSession<U> {
    pub fn new(size: usize) -> RoomSession<U> {
        RoomSession {
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

    pub fn sessions(&self) -> impl Iterator<Item = &Session<U>> {
        self.sessions.values()
    }

    pub fn session_mut(&mut self, token: SessionToken) -> Option<&mut Session<U>> {
        self.sessions.get_mut(&token)
    }

    pub fn remove_session_by_endpoint(&mut self, endpoint: Endpoint) -> Option<Session<U>> {
        if let Some(session) = self.session_by_endpoint(endpoint) {
            let token = session.token();
            self.sessions.remove(&token)
        }
        else { None }
    }

    pub fn session_by_endpoint(&self, safe_endpoint: Endpoint) -> Option<&Session<U>> {
        self.sessions.values().find(|session| {
            match session.safe_endpoint() {
                Some(endpoint) if *endpoint == safe_endpoint => true,
                _ => false,
            }
        })
    }

    pub fn session_by_endpoint_mut(&mut self, safe_endpoint: Endpoint) -> Option<&mut Session<U>> {
        self.sessions.values_mut().find(|session| {
            match session.safe_endpoint() {
                Some(endpoint) if *endpoint == safe_endpoint => true,
                _ => false,
            }
        })
    }

    pub fn create_session(&mut self, user: U, safe_endpoint: Endpoint) -> SessionStatus {
        let existing_session = self.sessions
            .values_mut()
            .find(|session| *session.user() == user);

        if let Some(session) = existing_session {
            match session.safe_endpoint() {
                Some(_) => SessionStatus::AlreadyLogged,
                None => {
                    session.set_safe_endpoint(safe_endpoint);
                    SessionStatus::Recycled(session.token())
                }
            }
        }
        else if self.is_full() {
            SessionStatus::Full
        }
        else {
            let new_token = self.generate_unique_token();
            let new_session = Session::new(new_token, user, safe_endpoint);
            self.sessions.insert(new_token, new_session);
            SessionStatus::Created(new_token)
        }
    }

    pub fn safe_endpoints(&self) -> Vec<Endpoint> {
        self.sessions().filter_map(|session| *session.safe_endpoint()).collect()
    }

    /// Tries to return the fast endpoint, if this is not possible, the safe endpoint is returned.
    pub fn faster_endpoints(&self) -> Vec<Endpoint> {
        self.sessions()
            .filter_map(|session| {
                match session.trusted_fast_endpoint() {
                    Some(fast_endpoint) => Some(*fast_endpoint),
                    None => *session.safe_endpoint(),
                }
            })
            .collect()
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

pub struct Session<U> {
    token: SessionToken,
    user: U,
    safe_endpoint: Option<Endpoint>,
    fast_endpoint: Option<Endpoint>,
    is_fast_endpoint_trusted: bool,
}

impl<U> Session<U> {
    fn new(token: SessionToken, user: U, safe_endpoint: Endpoint) -> Session<U> {
        Session {
            token,
            user,
            safe_endpoint: Some(safe_endpoint),
            fast_endpoint: None,
            is_fast_endpoint_trusted: false,
        }
    }

    pub fn user(&self) -> &U {
        &self.user
    }

    pub fn token(&self) -> SessionToken {
        self.token
    }

    pub fn safe_endpoint(&self) -> &Option<Endpoint> {
        &self.safe_endpoint
    }

    pub fn trusted_fast_endpoint(&self) -> &Option<Endpoint> {
        match self.is_fast_endpoint_trusted {
            true => &self.fast_endpoint,
            false => &None,
        }
    }

    fn set_safe_endpoint(&mut self, endpoint: Endpoint) {
        self.safe_endpoint = Some(endpoint);
    }

    pub fn set_untrusted_fast_endpoint(&mut self, endpoint: Endpoint) {
        self.fast_endpoint = Some(endpoint);
        self.is_fast_endpoint_trusted = false;
    }

    pub fn trust_in_fast_endpoint(&mut self) -> &Option<Endpoint> {
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


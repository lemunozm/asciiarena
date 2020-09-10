use crate::util::{SessionToken};

use rand::prelude::*;

use std::collections::HashMap;

pub enum SessionCreationResult {
    Created(SessionToken),
    Recycled(SessionToken),
    AlreadyLogged,
    Full,
}

pub enum HintEndpoint {
    OnlySafe,
    PreferedFast,
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

    pub fn size(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
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

    pub fn remove_session(&mut self, token: SessionToken) -> Option<PlayerSession<E>> {
        self.sessions.remove(&token)
    }

    pub fn remove_session_by_endpoint(&mut self, endpoint: E) -> Option<PlayerSession<E>> {
        if let Some(session) = self.session_by_endpoint(endpoint) {
            let token = session.token();
            self.remove_session(token)
        }
        else { None }
    }

    pub fn session_by_endpoint(&self, endpoint: E) -> Option<&PlayerSession<E>> {
        self.sessions.values().find(|session| {
            [session.safe_endpoint().as_ref(), session.fast_endpoint().as_ref()]
                .iter()
                .filter_map(|&e| e)
                .find(|e| **e == endpoint)
                .is_some()
        })
    }

    pub fn notify_lost_endpoint(&mut self, endpoint: E) -> Option<&PlayerSession<E>> {
        for (_, session) in &mut self.sessions {
            if let Some(safe_endpoint) = session.safe_endpoint() {
                if *safe_endpoint == endpoint {
                    session.safe_endpoint = None;
                    return Some(session)
                }
            }
            if let Some(fast_endpoint) = session.fast_endpoint() {
                if *fast_endpoint == endpoint {
                    session.fast_endpoint = None;
                    return Some(session)
                }
            }
        }
        None
    }

    pub fn add_fast_endpoint(&mut self, token: SessionToken, fast_endpoint: E) -> Option<()> {
        self.sessions.get_mut(&token).map(|session| session.fast_endpoint = Some(fast_endpoint))
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
                self.add_session(new_token, name, safe_endpoint);
                SessionCreationResult::Recycled(new_token)
            }
        }
        else if self.is_full() {
            SessionCreationResult::Full
        }
        else {
            let new_token = self.generate_unique_token();
            self.add_session(new_token, name, safe_endpoint);
            SessionCreationResult::Created(new_token)
        }
    }

    pub fn connected_endpoints(&self, hint: HintEndpoint) -> impl Iterator<Item = &E> {
        self.sessions()
            .map(move |session| {
                match hint {
                    HintEndpoint::OnlySafe => session.safe_endpoint(),
                    HintEndpoint::PreferedFast => match session.fast_endpoint().is_some() {
                        true => session.fast_endpoint(),
                        false => session.safe_endpoint(),
                    }
                }.as_ref()
            })
            .filter_map(|e| e) // Only connected endpoints
    }

    fn add_session(&mut self, token: SessionToken, name: &str, safe_endpoint: E) {
        let session = PlayerSession {
            token,
            name: name.to_string(),
            safe_endpoint: Some(safe_endpoint),
            fast_endpoint: None,
        };

        self.sessions.insert(token, session);
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
}

impl<E> PlayerSession<E> {
    pub fn is_connected(&self) -> bool {
        self.safe_endpoint.is_some()
    }

    pub fn has_udp(&self) -> bool {
        self.fast_endpoint.is_some()
    }

    pub fn has_connection_lost(&self) -> bool {
        self.safe_endpoint.is_none()
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

    pub fn fast_endpoint(&self) -> &Option<E> {
        &self.fast_endpoint
    }
}


use clap::{crate_version};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Compatibility {
    Fully = 2,
    NotExact = 1,
    None = 0,
}

impl Compatibility {
    pub fn is_compatible(&self) -> bool {
        *self != Compatibility::None
    }
}

enum VersionNumber {
    Major = 0,
    Minor = 1,
    Patch = 2,
}

use VersionNumber::*;

trait Version {
    fn number(&self, number: VersionNumber) -> usize;
    fn is_unstable(&self) -> bool;
}

impl Version for Vec<&str> {
    fn number(&self, number: VersionNumber) -> usize {
        self.get(number as usize).unwrap_or(&"0").parse().unwrap_or(0)
    }

    fn is_unstable(&self) -> bool {
        // It is an alpha version that could break the compatibility at any version change.
        self.number(Major) == 0 && self.number(Minor) == 0
    }
}

pub fn check(client_tag: &str, server_tag: &str) -> Compatibility {
    let server_version: Vec<_> = server_tag.split('.').collect();
    let client_version: Vec<_> = client_tag.split('.').collect();

    if client_version.number(Major) != server_version.number(Major) {
        Compatibility::None
    }
    else if client_version.number(Minor) != server_version.number(Minor) {
        Compatibility::None
    }
    else if client_version.number(Patch) != server_version.number(Patch) {
        if client_version.is_unstable() || server_version.is_unstable() {
            Compatibility::None
        }
        else {
            Compatibility::NotExact
        }
    }
    else {
        Compatibility::Fully
    }
}

pub const fn current() -> &'static str {
    crate_version!()
}

use clap::{crate_version};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Compatibility {
    Fully = 2,
    OkOutdated = 1,
    None = 0,
}

impl Compatibility {
    pub fn is_compatible(&self) -> bool {
        return *self != Compatibility::None
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
}

impl Version for Vec<&str> {
    fn number(&self, number: VersionNumber) -> usize {
        self.get(number as usize).unwrap_or(&"0").parse().unwrap_or(0)
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
        Compatibility::OkOutdated
    }
    else {
        Compatibility::Fully
    }
}

pub fn current() -> &'static str {
    crate_version!()
}

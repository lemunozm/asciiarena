use clap::{crate_version};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Compatibility {
    Fully,
    OkOutdated,
    None,
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

pub fn check(client_tag: &str) -> Compatibility {

    let server_version: Vec<_> = crate_version!().split('.').collect();
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

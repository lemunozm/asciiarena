#[macro_use]
extern crate lazy_static;
extern crate derive_new;

mod client;
mod server;

mod logger;
mod version;
mod message;
mod util;
mod direction;
mod vec2;

use clap::{self, App, AppSettings};

fn main() {
    let matches = App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(client::configure_cli())
        .subcommand(server::configure_cli())
        .get_matches();

    match matches.subcommand() {
        ("client", Some(matches)) => client::run(matches),
        ("server", Some(matches)) => server::run(matches),
        _ => unreachable!(),
    }
}

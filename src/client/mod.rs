mod application;
mod state;
mod input_widgets;
mod actions;
mod server_proxy;
mod terminal;
mod util;

use application::{Application, Config};

use crate::logger::{self};

use clap::{App, Arg, ArgMatches};

use std::net::{SocketAddr};

pub fn configure_cli<'a, 'b>() -> App<'a, 'b> {
    App::new("client")
        .about("Running an asciiarena client")
        .arg(Arg::with_name("log")
            .long("log")
            .short("l")
            .default_value("off")
            .possible_values(&logger::LOG_LEVELS)
            .help("Sets the log level of verbosity")
        )
        .arg(Arg::with_name("character")
            .long("character")
            .short("c")
            .value_name("CAPITAL_LETTER")
            .validator(|name| match super::util::is_valid_character_name(&name) {
                true => Ok(()),
                false => Err("The character must be an unique capital letter".into()),
            })
            .help("Set the player's character. Must be unique in the server")
        )
        .arg(Arg::with_name("host")
            .long("host")
            .short("h")
            .value_name("HOST")
            .validator(|host| match host.parse::<SocketAddr>() {
                Ok(_) => Ok(()),
                Err(_) => Err("Host must be a valid network address".into()),
            })
            .help("Set the server address, ip and port. Example: '127.0.0.1:3001'")
        )
}

pub fn run(matches: &ArgMatches) {
    logger::init(matches.value_of("log").unwrap().parse().unwrap());

    let config = Config {
        character: matches.value_of("character").map(|name| name.chars().next().unwrap()),
        server_addr: matches.value_of("host").map(|addr| addr.parse().unwrap()),
    };

    Application::new(config).run();
}

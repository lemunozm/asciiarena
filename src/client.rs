mod application;
mod configuration;
mod state;
mod store;
mod server_proxy;
mod gui;

use application::{Application};
use configuration::{Config};

use crate::logger::{self};

use clap::{App, Arg, ArgMatches};

use std::net::{SocketAddr};

lazy_static! {
    static ref DEFAULT_LOG_FILE: String = format!(
        "asciiarena_client_{}.log",
        chrono::Local::now().format("%Y-%m-%d_%H:%M:%S")
    );
}

pub fn configure_cli<'a, 'b>() -> App<'a, 'b> {
    App::new("client")
        .about("Running asciiarena client mode")
        .arg(Arg::with_name("log")
            .long("log")
            .short("l")
            .value_name("LEVEL")
            .default_value("off")
            .possible_values(&logger::LOG_LEVELS)
            .help("Set the log level of verbosity")
        )
        .arg(Arg::with_name("log-file")
            .long("log-file")
            .value_name("FILE")
            .default_value(&DEFAULT_LOG_FILE)
            .help("Set the log file")
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
            .help("Set the server address (ip and port). Example: '192.168.0.56:3001'")
        )
}

pub fn run(matches: &ArgMatches) {
    let level = matches.value_of("log").unwrap().parse().unwrap();
    let file_name = matches.value_of("log-file").unwrap();
    logger::init(level, logger::Output::File(file_name));

    let config = Config {
        character: matches.value_of("character").map(|name| name.chars().next().unwrap()),
        server_addr: matches.value_of("host").map(|addr| addr.parse().unwrap()),
    };

    Application::new(config).run();
}

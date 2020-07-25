use crate::client_manager::{ClientManager};
use crate::logger::{self};

use clap::{App, Arg, ArgMatches};

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
        .arg(Arg::with_name("name")
            .long("name")
            .short("n")
            .value_name("NAME")
            .help("Set the player name. Must be unique in the server")
        )
}

pub fn run(matches: &ArgMatches) {
    logger::init(matches.value_of("log").unwrap().parse().unwrap());
    let server_addr = "127.0.0.1:3001".parse().unwrap();
    let player_name = matches.value_of("name");

    if let Some(mut client_manager) = ClientManager::new(server_addr, player_name) {
        client_manager.run();
    }
}

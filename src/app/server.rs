use crate::server_manager::{ServerManager, ServerConfig};
use crate::logger::{self};

use clap::{App, Arg, ArgMatches};

use std::time::{Duration};

pub fn configure_cli<'a, 'b>() -> App<'a, 'b> {
    App::new("server")
        .about("Running an asciiarena server")
        .arg(Arg::with_name("log")
            .long("log")
            .short("l")
            .default_value("info")
            .possible_values(&logger::LOG_LEVELS)
            .help("Sets the log level of verbosity")
        )
}

pub fn run(matches: &ArgMatches) {
    logger::init(matches.value_of("log").unwrap().parse().unwrap());

    let config = ServerConfig {
        tcp_port: 3001,
        udp_port: 3001,
        players_number: 2,
        map_size: 30,
        winner_points: 15,
        init_arena_waiting: Duration::from_secs(3),
    };

    if let Some(mut server_manager) = ServerManager::new(config) {
        server_manager.run();
    }
}

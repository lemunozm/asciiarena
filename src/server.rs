mod server_manager;
mod session;
mod game;
mod arena;

use server_manager::{ServerManager, ServerConfig};

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
        .arg(Arg::with_name("players")
            .long("players")
            .short("p")
            .default_value("2")
            .validator(|value| {
                match value.parse::<u32>() {
                    Ok(number) => if number <= 0 { Err("The value must be > 0".into()) } else { Ok(()) },
                    Err(_) => Err("The value must be a number".into())
                }
            })
            .help("Number of players (> 1). The game will not start until the number of players has been reached.")
        )
}

pub fn run(matches: &ArgMatches) {
    logger::init(matches.value_of("log").unwrap().parse().unwrap());


    let config = ServerConfig {
        tcp_port: 3001,
        udp_port: 3002,
        players_number: matches.value_of("players").unwrap().parse().unwrap(),
        map_size: 20,
        winner_points: 5,
        arena_waiting: Duration::from_secs(3),
    };

    if let Some(mut server_manager) = ServerManager::new(config) {
        server_manager.run();
    }
}

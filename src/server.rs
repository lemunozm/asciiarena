mod server_manager;
mod session;
mod game;
mod arena;

use server_manager::{ServerManager, Config};

use crate::logger::{self};

use clap::{App, Arg, ArgMatches};

use std::time::{Duration};

lazy_static! {
    static ref DEFAULT_TCP_PORT: String = 3549.to_string();
    static ref DEFAULT_UDP_PORT: String = 3549.to_string();
}

pub fn configure_cli<'a, 'b>() -> App<'a, 'b> {
    App::new("server")
        .about("Running asciiarena server mode")
        .arg(Arg::with_name("log")
            .long("log")
            .short("l")
            .value_name("LEVEL")
            .default_value("info")
            .possible_values(&logger::LOG_LEVELS)
            .help("Set the log level of verbosity")
        )
        .arg(Arg::with_name("tcp-port")
            .long("tcp-port")
            .value_name("PORT")
            .default_value(&DEFAULT_TCP_PORT)
            .validator(|port| match port.parse::<u16>() {
                Ok(_) => Ok(()),
                Err(_) => Err("The value must be in range 0..65535".into())
            })
            .help("Set the tcp port for client connections")
        )
        .arg(Arg::with_name("udp-port")
            .long("udp-port")
            .value_name("PORT")
            .default_value(&DEFAULT_UDP_PORT)
            .validator(|port| match port.parse::<u16>() {
                Ok(_) => Ok(()),
                Err(_) => Err("The value must be in range 0..65535".into())
            })
            .help("Set the udp port for client connections")
        )
        .arg(Arg::with_name("map-size")
            .long("map-size")
            .short("s")
            .value_name("SIZE")
            .default_value("20")
            .validator(|port| match port.parse::<usize>() {
                Ok(_) => Ok(()),
                Err(_) => Err("The value must be a positive number".into())
            })
            .help("Set the map size length")
        )
        .arg(Arg::with_name("players")
            .long("players")
            .short("p")
            .value_name("NUMBER")
            .required(true)
            .validator(|value| {
                match value.parse::<u32>() {
                    Ok(number) => match number > 0 {
                        true => Ok(()),
                        false => Err("The value must be > 0".into()),
                    }
                    Err(_) => Err("The value must be a number".into())
                }
            })
            .help("Number of players (> 1). The game will not start until the number of players has been reached.")
        )
}

pub fn run(matches: &ArgMatches) {
    let level = matches.value_of("log").unwrap().parse().unwrap();
    logger::init(level, logger::Output::Stdout);

    let config = Config {
        tcp_port: matches.value_of("tcp-port").unwrap().parse().unwrap(),
        udp_port: matches.value_of("udp-port").unwrap().parse().unwrap(),
        players_number: matches.value_of("players").unwrap().parse().unwrap(),
        map_size: matches.value_of("map-size").unwrap().parse().unwrap(),
        winner_points: 5,
        arena_waiting: Duration::from_secs(3),
    };

    if let Some(mut server_manager) = ServerManager::new(config) {
        server_manager.run();
    }
}

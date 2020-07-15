use crate::server_manager::{ServerManager};
use crate::logger::{self};

use clap::{App, Arg, ArgMatches};

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

    if let Some(mut server_manager) = ServerManager::new(3001, 3001) {
        server_manager.run();
    }
    else {
        log::error!("Could not run server on the specified ports"); //print ports
    }
}

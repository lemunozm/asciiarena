use crate::client_manager::{ClientManager};
use crate::logger::{self};

use clap::{App, Arg, ArgMatches};

pub fn configure_cli<'a, 'b>() -> App<'a, 'b> {
    App::new("client")
        .about("Running an asciiarena client")
        .arg(Arg::with_name("log")
            .long("log")
            .default_value("off")
            .possible_values(&logger::LOG_LEVELS)
            .help("Sets the log level of verbosity")
        )
}

pub fn run(matches: &ArgMatches) {
    logger::init(matches.value_of("log").unwrap().parse().unwrap());

    if let Some(mut client_manager) = ClientManager::new("127.0.0.1:3001".parse().unwrap()) {
        println!("Connected to server");
        if let None = client_manager.run() {
            println!("Connection lost with the server");
        }
    }
    else {
        println!("Could not connect to server");
    }
}

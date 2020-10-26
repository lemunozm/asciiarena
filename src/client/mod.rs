mod application;
mod state;
mod input_widgets;
mod actions;
mod server_proxy;
mod terminal;
mod util;

use application::{Application};

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
    //let server_addr = Some("127.0.0.1:3001".parse().unwrap());
    let server_addr = None;
    let player_name = matches.value_of("name");

    Application::new(server_addr, player_name).run();
}

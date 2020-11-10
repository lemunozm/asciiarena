use fern::colors::{Color, ColoredLevelConfig};
use log::{LevelFilter};
use colored::{Colorize};
use clap::{crate_name};

use std::str::{FromStr};

#[derive(PartialEq)]
pub enum Level {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    Dev
}

#[derive(Debug, Clone)]
pub struct LevelUnknown;

pub const LOG_LEVELS: [&'static str; 7] =
    ["off", "error", "warning", "info", "debug", "trace", "dev"];

impl FromStr for Level {
    type Err = LevelUnknown;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "off" => Ok(Level::Off),
            "error" => Ok(Level::Error),
            "warning" => Ok(Level::Warn),
            "info" => Ok(Level::Info),
            "debug" => Ok(Level::Debug),
            "trace" => Ok(Level::Trace),
            "dev" => Ok(Level::Dev),
            _ => Err(LevelUnknown),
        }
    }
}

pub enum Output<'a> {
    Stdout,
    File(&'a str),
}

pub fn init(level: Level, output: Output) {
    let level_colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Cyan)
        .debug(Color::White)
        .trace(Color::BrightBlack);

    let log_config = fern::Dispatch::new().level(LevelFilter::Off);

    let log_config = match level {
        Level::Off => return,
        Level::Error => log_config.level_for(crate_name!(), LevelFilter::Error),
        Level::Warn => log_config.level_for(crate_name!(), LevelFilter::Warn),
        Level::Info => log_config.level_for(crate_name!(), LevelFilter::Info),
        Level::Debug => log_config.level_for(crate_name!(), LevelFilter::Debug),
        Level::Trace => log_config.level_for(crate_name!(), LevelFilter::Trace),
        Level::Dev => log_config.level(LevelFilter::Trace),
    };

    let log_config = log_config.format(move |out, message, record| {
        let target = if level == Level::Dev {
            format!("[{}] ", record.target())
        }
        else {
            String::new()
        };

        out.finish(format_args!(
            "{}{}{} {}",
            format!("[{}] ", chrono::Local::now().format("%H:%M:%S")).white(),
            target.white(),
            level_colors.color(record.level()),
            format!("{}", message).bright_white(),
        ))
    });

    let log_config = match output {
        Output::Stdout => log_config.chain(std::io::stdout()),
        Output::File(name) => log_config.chain(fern::log_file(name).unwrap()),
    };

    log_config.apply().unwrap();
}


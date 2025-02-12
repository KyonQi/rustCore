use log::{Level, LevelFilter, Log};

use crate::println;

struct Logger;

// enum LEVEL {
//     ERROR,
//     WARN,
//     INFO,
//     DEBUG,
//     TRACE,
// }

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();
    // makefile will specify the LOG var
    log::set_max_level(match option_env!("LOG") {
        Some("ERROR") => LevelFilter::Error,
        Some("WARIN") => LevelFilter::Warn,
        Some("INFO") => LevelFilter::Info,
        Some("DEBUG") => LevelFilter::Debug,
        Some("TRACE") => LevelFilter::Trace,
        _ => LevelFilter::Info,
    });
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            match record.level() {
                Level::Error => {
                    println!("\x1b[{}m{} {}\x1b[0m",
                            31,
                            record.level().as_str(),
                            record.args());
                },
                Level::Warn => {
                    println!("\x1b[{}m{} {}\x1b[0m",
                            93,
                            record.level().as_str(),
                            record.args());
                },
                Level::Info => {
                    println!("\x1b[{}m{} {}\x1b[0m",
                            34,
                            record.level().as_str(),
                            record.args());
                },
                Level::Debug => {
                    println!("\x1b[{}m{} {}\x1b[0m",
                            32,
                            record.level().as_str(),
                            record.args());
                },
                Level::Trace => {
                    println!("\x1b[{}m{} {}\x1b[0m",
                            90,
                            record.level().as_str(),
                            record.args());
                },
            }
        }
    }

    fn flush(&self) {}
}
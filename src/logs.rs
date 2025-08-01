use log::{Level, Metadata, Record};

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            eprintln!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn enable_debug() -> Result<(), log::SetLoggerError> {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Debug))
}

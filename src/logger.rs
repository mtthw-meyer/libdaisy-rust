use log::{Record, Level, Metadata};

#[cfg(any(feature = "rtt"))]
use rtt_target::{rprintln, rtt_init_print};

pub struct Logger;

impl Logger {
    pub fn init(&self) {
        #[cfg(any(feature = "rtt"))]
        rtt_init_print!();
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        #[cfg(any(feature = "rtt"))]
        rprintln!("{} - {}", record.level(), record.args());
    }

    fn flush(&self) {}
}

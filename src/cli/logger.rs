use console::style;

lazy_static! {
    static ref PREFIX_DEBUG: String = style("[DEBUG]".to_string()).dim().for_stderr().to_string();
    static ref PREFIX_INFO: String = style("[INFO]".to_string()).dim().for_stderr().to_string();
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum LogLevel {
    DEBUG,
    INFO,
}

pub struct Logger {
    log_level: LogLevel,
}
impl Logger {
    pub fn new(log_level: LogLevel) -> Logger {
        Logger { log_level }
    }
    pub fn log_debug(&self, message: String) {
        if self.log_level <= LogLevel::DEBUG {
            eprintln!("{} {}", PREFIX_DEBUG.as_str(), message);
        }
    }
    pub fn log_info(&self, message: String) {
        if self.log_level <= LogLevel::INFO {
            eprintln!("{} {}", PREFIX_INFO.as_str(), message);
        }
    }
}

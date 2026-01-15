//! Enumeration representing log levels & support functions.
use super::atoms;

/// An enumeration representing log levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// logs for debugging purposes
    Debug,
    /// logs for informational purposes
    Info,
    /// logs of noteworthy events
    Notice,
    /// logs of warnings
    Warning,
    /// logs of errors
    Error,
    /// logs of critical failures
    Critical,
    /// logs that should alert an operator
    Alert,
    /// logs that require **immediate** attention
    Emergency,
}

impl From<&str> for LogLevel {
    /// Finds the log level from a string.
    fn from(s: &str) -> Self {
        match s {
            "trace" => LogLevel::Debug, // convenience synonym
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "notice" => LogLevel::Notice,
            "warn" => LogLevel::Warning,
            "warning" => LogLevel::Warning, // convenience synonym
            "error" => LogLevel::Error,
            "critical" => LogLevel::Critical,
            "alert" => LogLevel::Alert,
            "emergency" => LogLevel::Emergency,
            "fatal" => LogLevel::Emergency, // convenience synonym
            _ => LogLevel::Debug,
        }
    }
}

impl From<LogLevel> for &str {
    /// Converts the log level to a string.
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Notice => "notice",
            LogLevel::Warning => "warning",
            LogLevel::Error => "error",
            LogLevel::Critical => "critical",
            LogLevel::Alert => "alert",
            LogLevel::Emergency => "emergency",
        }
    }
}

impl LogLevel {
    /// Returns the atom representation of the log level.
    pub fn as_atom(&self) -> rustler::Atom {
        match self {
            LogLevel::Debug => atoms::debug(),
            LogLevel::Info => atoms::info(),
            LogLevel::Notice => atoms::notice(),
            LogLevel::Warning => atoms::warning(),
            LogLevel::Error => atoms::error(),
            LogLevel::Critical => atoms::critical(),
            LogLevel::Alert => atoms::alert(),
            LogLevel::Emergency => atoms::emergency(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::LogLevel;

    #[test]
    fn test_log_level_convenience_synonyms() {
        assert_eq!(LogLevel::from("trace"), LogLevel::Debug);
        assert_eq!(LogLevel::from("warn"), LogLevel::Warning);
        assert_eq!(LogLevel::from("fatal"), LogLevel::Emergency);
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Notice);
        assert!(LogLevel::Notice < LogLevel::Warning);
        assert!(LogLevel::Warning < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Critical);
        assert!(LogLevel::Critical < LogLevel::Alert);
        assert!(LogLevel::Alert < LogLevel::Emergency);
    }
}

// Contains the main API to send log messages.
use super::level::LogLevel;
use rustler::Encoder;
use std::panic::set_hook;
use std::rc::Rc;
use std::sync::LazyLock;

/// A structure to represent log messages.
#[derive(Clone)]
pub struct Log {
    /// specifies the log level of the message
    pub level: LogLevel,
    /// format string for the simple message, uses Erlang formatter syntax
    pub format: String,
    /// arguments for the format string
    pub args: Vec<Rc<dyn Encoder>>,
    /// metadata for the log message
    pub metadata: Vec<(String, Rc<dyn Encoder>)>,
    /// used to catch unsent messages that accidentally get dropped
    pub pending: bool,
}

impl Log {
    /// Create a new Log builder.
    ///
    /// Once created, a log message must be sent explicitly.
    ///
    /// To prevent creating a log message but erroneously failing to send it, a
    /// panic is raised if a message is dropped without being sent.
    ///
    /// If a message is legitimately created but not sent, use the `cancel`
    /// function.
    ///
    /// # Example
    ///
    /// ```
    /// use rustler_logger::*;
    ///
    /// let log = Log::new(LogLevel::Info, "Hello, ~s!")
    ///     .arg("world")
    ///     .send();
    /// ```
    pub fn new(level: LogLevel, format: &str) -> Self {
        Log {
            level,
            format: format.to_string(),
            args: Vec::new(),
            metadata: Vec::new(),
            pending: true,
        }
    }

    /// Builder-style method to put an argument into a log message.
    ///
    /// # Example
    ///
    /// ```
    /// use rustler_logger::*;
    ///
    /// let log = Log::new(LogLevel::Info, "Hello, ~s!")
    ///     .arg("world")
    ///     .send();
    /// ```
    pub fn arg(mut self, arg: impl Encoder + 'static) -> Self {
        self.args.push(Rc::new(arg));
        self
    }

    /// Builder-style method to put an optional argument into a log message.
    ///
    /// If the value is `Some(arg)`, `arg` will be unwrapped and included in the
    /// log message. If the value is `None`, the key will not be included.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use rustler_logger::*;
    ///
    /// let log = Log::new(LogLevel::Info, "Hello, ~s!")
    ///     .opt_arg(Some("world"))
    ///     .send();
    /// ```
    pub fn opt_arg(mut self, arg: Option<impl Encoder + 'static>) -> Self {
        if let Some(arg) = arg {
            self.args.push(Rc::new(arg));
        }
        self
    }

    /// Builder-style method to put an optional argument into a log message, or
    /// a default value if the argument is `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use rustler_logger::*;
    ///
    /// let log = Log::new(LogLevel::Info, "Hello, ~s!")
    ///     .opt_arg_else(Some("world"), "unknown")
    ///     .send();
    /// ```
    pub fn opt_arg_else(
        mut self,
        some_arg: Option<impl Encoder + 'static>,
        none_arg: impl Encoder + 'static,
    ) -> Self {
        if let Some(arg) = some_arg {
            self.args.push(Rc::new(arg));
        } else {
            self.args.push(Rc::new(none_arg));
        }
        self
    }

    /// Builder-style method to put a key-value-pair into a log message.
    ///
    /// # Example
    ///
    /// ```
    /// use rustler_logger::*;
    ///
    /// let log = Log::new(LogLevel::Info, "Hello, {}!")
    ///     .arg("world")
    ///     .meta("user_id", 123)
    ///     .send();
    /// ```
    pub fn meta(mut self, key: &str, value: impl Encoder + 'static) -> Self {
        self.metadata.push((key.to_string(), Rc::new(value)));
        self
    }
    /// Builder-style method to put an optional key-value-pair into a log
    /// message.
    ///
    /// If the value is `None`, the key will not be included in the log message.
    ///
    /// # Example
    ///
    /// ```
    /// use rustler_logger::*;
    ///
    /// let quota: Option<u64> = None;
    /// let uid: Option<u64> = Some(1003);
    ///
    /// let log = Log::new(LogLevel::Info, "Hello, {}!")
    ///     .arg("world")
    ///     .opt_meta("quota", quota)
    ///     .opt_meta("uid", uid)
    ///     .send();
    /// ```
    pub fn opt_meta(mut self, key: &str, value: Option<impl Encoder + 'static>) -> Self {
        if let Some(value) = value {
            self.metadata.push((key.to_string(), Rc::new(value)));
        }
        self
    }

    /// Sends the constructed log message.
    ///
    /// # Example
    ///
    /// ```
    /// use rustler_logger::*;
    ///
    /// let log = Log::new(LogLevel::Info, "Hello, {}!")
    ///     .arg("world")
    ///     .meta("user_id", 123)
    ///     .send();
    /// ```
    #[cfg(not(any(doctest, test, feature = "testing")))]
    pub fn send(mut self) {
        use super::atoms;
        use super::context::ENV;
        use rustler::{Atom, Term};

        if !self.pending {
            panic!("attempt to send a log message that has already been used")
        }

        ENV.with(|env_ptr| {
            let env = *env_ptr;
            let args: Vec<Term> = self.args.iter().map(|arg| arg.encode(env)).collect();
            let metadata_pairs: Vec<(Term, Term)> = self
                .metadata
                .iter()
                .map(|(key, value)| {
                    (
                        Atom::from_str(env, key).unwrap().to_term(env),
                        value.encode(env),
                    )
                })
                .collect();

            let metadata = match Term::map_from_pairs(env, &metadata_pairs[..]) {
                Ok(map) => map,
                Err(_) => panic!("Failed to create metadata map"),
            };
            let logger_proxy_pid = match env.whereis_pid(atoms::logger_proxy()) {
                Some(pid) => pid,
                None => panic!("BEAM logger proxy process is not registered?!"),
            };
            if !logger_proxy_pid.is_alive(env) {
                panic!("BEAM logger proxy process is not alive?!");
            }

            self.pending = false;

            let log_msg = (
                atoms::log(),
                self.level.as_atom(),
                self.format.clone(),
                args.clone(),
                metadata,
            );

            if env.send(&logger_proxy_pid, log_msg).is_err() {
                panic!("failed to ship log message to Elixir");
            };
        });
    }

    // Don't actually try to send in tests. It won't work because we don't have
    // a real `Env`.
    #[cfg(any(doctest, test, feature = "testing"))]
    pub fn send(mut self) {
        if !self.pending {
            panic!("attempt to send a log message that has already been used")
        } else {
            self.pending = false;
        }
    }

    /// Cancel a log message that has not been sent yet.
    ///
    /// If a message is constructed but not sent, it must be cancelled to
    /// prevent a panic. This panic occurs so that log messages won't be
    /// constructed but then accidentally not sent.
    ///
    /// # Example
    ///
    /// ```
    /// use rustler_logger::*;
    ///
    /// let log = Log::new(LogLevel::Info, "Hello, ~s!")
    ///     .arg("world");
    ///
    /// log.cancel();
    /// ```
    pub fn cancel(mut self) {
        if self.pending {
            self.pending = false;
        } else {
            panic!("attempt to cancel a log message that has already been used")
        }
    }
}

// We implement a panic if a message is dropped but hasn't been sent.
impl Drop for Log {
    /// Panic if a log message is dropped without being sent or cancelled.
    fn drop(&mut self) {
        if self.pending {
            panic!("log message generated but not cancelled or sent")
        }
    }
}

static INITIALIZED: LazyLock<bool> = LazyLock::new(|| {
    // set the panic hook
    set_hook(Box::new(super::panic::panic_hook));
    true
});

/// Initialize Elixir logging.
///
/// This is conventionally used in the load function of a Rustler module.
/// Currently only registers the panic hook.
///
/// # Example
///
/// ```ignore
/// // This is necessary to load the panic hook.
/// fn load(_env: Env, _term: Term) -> bool {
///   rustler_logger::log_init();
///   true
/// }
///
/// rustler::init!("Elixir.Your.Module", load = load);
/// ```
pub fn log_init() {
    assert!(*INITIALIZED);
}

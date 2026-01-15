//! Simplify logging from Rust to Elixir.
//!
//! `rustler` provides fairly seamless integration between Rust and Elixir.
//! However, Rust console output is emitted without much concern for what Elixir
//! might be doing with the console. Notably, panic messages can be difficult to
//! read when using `iex`. The lack of integration with Elixir's logging
//! infrastructure also leaves a hole in observability of tools built with
//! `rustler`.
//!
//! This crate provides some of this missing integration.  You get...
//! - the `log_to_elixir` attribute that provides logging functionality in nifs
//! - the `log!` macro to actually emit log messages explicitly
//! - a panic handler that routes panic message to Elixir
//!
//! # Usage
//!
//! 1. Use the `log_to_elixir` attribute on nifs to enable logging.
//! 2. Initialize logging with `rustler_logger::log_init()` (optional).
//! 3. Use logging macros!
//!
//! # Example
//!
//! ```ignore
//! use rustler::{nif, Env, Term};
//! use rustler_logger::{log, log_to_elixir};
//!
//! #[log_to_elixir]
//! #[nif]
//! fn hello() {
//!   log!(
//!     "Hello, ~p! ~p items on your list labeled ~p",  // erlang-style format string
//!     "world", 42, "TODO",                            // format arguments
//!     user="bob", answer=420/10                       // log metadata
//!   );
//! }
//!
//! // This is necessary to load the panic hook.
//! fn load(_env: Env, _term: Term) -> bool {
//!     rustler_logger::log_init();
//!     true
//! }
//!
//! rustler::init!("Elixir.Your.Module", load = load);
//! ```
//!
//! # Formatting
//! The `log!` macro accepts a format string and a list of arguments, similar to
//! the `format!` macro, though the formatting is done by Elixir's logger using
//! the notation provided by OTP. It handles arguments of different types
//! automatically and allows arbitrary expressions for values.
//!
//! # Metadata
//! Assignment statements can be used to add metadata for structured logging.
//! The variable before the `=` will be turned into an atom to conform to Elixir's
//! convention for log message metadata.
//!
//! # Panic Handling
//! A panic hook is provided to emit a more detailed panic message to Elixir's logger.
//! It does not (currently) catch the panic, just resumes panicking.

/// Pre-allocated atoms for use logging.
mod atoms;
/// Infrastructure to provide context tracking to allow logging from anywhere
/// within a NIF thread.
mod context;
/// Enums for indicating log levels.
mod level;
/// Message formatting and sending.
mod message;
/// Panic handling integration and logging code.
mod panic;

// provide API
pub use level::*;
pub use message::*;

// include convenience macros
pub use rustler_logger_macro::*;

// hide this as it's only to be used by convenience macros
#[doc(hidden)]
pub use crate::context::use_env as _use_env;

// hide this as it's only to be used by panic hook
#[doc(hidden)]
pub use crate::panic::send_panic_message as _send_panic_message;

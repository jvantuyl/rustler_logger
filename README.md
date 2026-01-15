# Rustler Logger

[Documentation](https://docs.rs/rustler_logger/latest/rustler_logger/)

`rustler` provides fairly seamless integration between Rust and Elixir.
However, Rust NIFs aren't really integrated with Elixir's logging infrastructure.

This crate provides a set of macros that can be used to log messages directly to the Elixir logger.

### Features

- a `log_to_elixir` attribute macro that wraps functions with code that captures the Elixir environment to send logs
- a `log!` macro to actually emit log messages
- convenience macros for each log level (i.e. `debug!`, `info!`, etc.)
- "keyword argument"-style arguments to set logging metadata
- a panic handler that sends a message with panic information to Elixir

### Usage

```elixir
use rustler::{nif, Env, Term};
use rustler_logger::*;

#[nif]
#[log_to_elixir]
fn show_off_logging() {
  info!(
    "Hello, ~p! ~p items on your list labeled ~p",  // erlang-style format string
    "world", 42, "TODO",                            // format arguments
    user="bob", answer=420/10                       // metadata
  );

  // all of the log levels
  debug!("convenience");
  info!("macros");
  notice!("for");
  warn!("each");
  error!("log");
  critical!("level");
  alert!("are");
  emergency!("available");
  
  // If you need to select a level programmatically, you can use the `log!`
  // macro with a level argument.
  let log_level = LogLevel::Debug;
  log!(log_level, "This is a {} message", log_level.as_str());
}

// This is necessary to load the panic hook.
fn load(_env: Env, _term: Term) -> bool {
    log_init();
    true
}

rustler::init!("Elixir.Your.Module", load = load);
```

### Troubleshooting & Gotchas

Failing to initialize the logger can cause panic messages to not be emitted.

Failing to wrap functions with `log_to_elixir` can cause log messages to not be emitted.

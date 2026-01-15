// Panic Handling / Integration
use super::level::LogLevel;
use super::message::Log;
use std::{any::Any, cell::RefCell, panic::PanicHookInfo};

/// A structure to hold panic information.
pub(crate) struct PanicInfo {
    pub(crate) message: String,
    pub(crate) file: Option<String>,
    pub(crate) line: Option<u32>,
    pub(crate) col: Option<u32>,
}

thread_local! {
    /// Static, thread-local storage for panic information.
    pub(crate) static PANIC_INFO: RefCell<Option<PanicInfo>> = const {RefCell::new(None)};
}

/// The actual panic_hook implementation.
///
/// This function is called when a panic occurs in the Rust code. It captures
/// the panic information and stores it in the PANIC_INFO thread-local variable.
pub fn panic_hook(info: &PanicHookInfo) {
    let err = any_to_string(info.payload());
    let loc = info.location();
    let panic_info = PanicInfo {
        message: err,
        file: loc.map(|l| l.file().to_string()),
        line: loc.map(|l| l.line()),
        col: loc.map(|l| l.column()),
    };
    PANIC_INFO.with(|info| {
        *info.borrow_mut() = Some(panic_info);
    });
}

#[doc(hidden)]
fn any_to_string(err: &dyn Any) -> String {
    match err.downcast_ref::<&'static str>() {
        Some(err_static_str) => err_static_str.to_string(),
        None => match err.downcast_ref::<String>() {
            Some(err_string) => err_string.to_string(),
            None => "Box<dyn Any>".to_string(),
        },
    }
}

/// Function used to package panic information to log to the BEAM VM.
///
/// Used by the panic hook.
pub fn send_panic_message(fname: &str, arity: u32) {
    let panic_info = PANIC_INFO.replace(None);
    let panic_info = panic_info.as_ref();

    let file = match panic_info {
        Some(PanicInfo {
            file: Some(file), ..
        }) => file.clone(),
        _ => "<unknown>".to_string(),
    };
    let line = match panic_info {
        Some(PanicInfo {
            line: Some(line), ..
        }) => line.to_string(),
        _ => "?".to_string(),
    };
    let col = match panic_info {
        Some(PanicInfo { col: Some(col), .. }) => col.to_string(),
        _ => "?".to_string(),
    };

    // We want to use some of the optional argument functionality which isn't
    // available using the quick logging macro, so we construct the message
    // ourselves here.
    Log::new(LogLevel::Critical, "rustler_nif_panic[~s/~s@~s:~s:~s]: ~s")
        .arg(fname.to_string())
        .arg(arity.to_string())
        .arg(file.clone())
        .arg(line.clone())
        .arg(col.clone())
        .opt_arg_else(
            panic_info.map(|p| p.message.clone()),
            "unable to find panic information",
        )
        .meta("file", file)
        .meta("line", line)
        .meta("column", col)
        .send();
}

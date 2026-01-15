use rustler::atoms;

atoms! {
    // tuple tag
    log,
    // log levels
    debug,
    info,
    notice,
    warning,
    error,
    critical,
    alert,
    emergency,
    // return values
    nif_panicked,
    // well-known process names
    logger_proxy,
}

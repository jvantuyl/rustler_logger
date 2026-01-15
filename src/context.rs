//! Context saved to allow generation of Terms to message BEAM
use rustler::Env;
use scoped_thread_local::*;

// Thread-local storage for `rustler::Env` structure needed to message the BEAM.
scoped_thread_local! {
    pub static ENV: for<'a> Env<'a>
}

// A helper function to safely provide the stored `Env` to a block code.
pub fn use_env<F, R>(env: Env, func: F) -> R
where
    F: FnOnce() -> R,
{
    let mut env = env;
    ENV.set(&mut env, func)
}

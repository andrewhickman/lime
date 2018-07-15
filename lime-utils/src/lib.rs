extern crate failure;
#[macro_use]
extern crate log;
extern crate backtrace;

use std::any::Any;
use std::panic::{self, PanicInfo};
use std::thread;

use backtrace::Backtrace;

#[cold]
#[inline(never)]
/// Panic with a payload of a failure::Error.
pub fn throw<E: Into<failure::Error>, T>(err: E) -> T {
    panic!(err.into())
}

#[cold]
#[inline(never)]
/// Custom panic hook that accepts a failure::Error.
pub fn panic_hook(info: &PanicInfo) {
    let msg = fmt_payload(info.payload());
    let thread = thread::current();
    let name = thread.name().unwrap_or("<unnamed>");

    error!(
        "thread '{}' panicked at '{}'\n{:#?}",
        name,
        msg,
        Backtrace::new()
    );
}

/// Set the panic hook.
pub fn set_panic_hook() {
    panic::set_hook(Box::new(panic_hook))
}

fn fmt_payload(payload: &(Any + Send)) -> String {
    if let Some(&s) = payload.downcast_ref::<&'static str>() {
        s.into()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else if let Some(e) = payload.downcast_ref::<failure::Error>() {
        fmt_error(&e)
    } else {
        "Box<Any>".into()
    }
}

fn fmt_error(err: &failure::Error) -> String {
    let mut pretty = err.to_string();
    let mut prev = err.cause();
    while let Some(next) = prev.cause() {
        pretty.push_str(": ");
        pretty.push_str(&next.to_string());
        prev = next;
    }
    pretty
}

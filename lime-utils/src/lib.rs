extern crate failure;
#[macro_use]
extern crate log;
extern crate backtrace;

use std::any::Any;
use std::panic::PanicInfo;
use std::thread;

use backtrace::Backtrace;

#[cold]
#[inline(never)]
/// Panic with a payload of a failure::Error.
pub fn throw<F: Into<failure::Error>, T>(err: F) -> T {
    panic!(err.into())
}

#[cold]
#[inline(never)]
/// Panic with an error message.
pub fn throw_msg<S: Into<String>, T>(msg: S) -> T {
    throw(failure::err_msg(msg.into()))
}

#[cold]
#[inline(never)]
/// Custom panic hook that accepts a failure::Error.
pub fn hook(info: &PanicInfo) {
    let msg = fmt_payload(info.payload());
    let thread = thread::current();
    let name = thread.name().unwrap_or("<unnamed>");

    error!("thread '{}' panicked at '{}'\n{:#?}", name, msg, Backtrace::new());
}

fn fmt_payload(payload: &(Any + Send)) -> String {
    if let Some(&s) = payload.downcast_ref::<&'static str>() {
        s.into()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else if let Some(e) = payload.downcast_ref::<failure::Error>() {
        pretty_error(&e)
    } else {
        "Box<Any>".into()
    }
}

/// Return a prettily formatted error, including its entire causal chain.
pub fn pretty_error(err: &failure::Error) -> String {
    let mut pretty = err.to_string();
    let mut prev = err.cause();
    while let Some(next) = prev.cause() {
        pretty.push_str(": ");
        pretty.push_str(&next.to_string());
        prev = next;
    }
    pretty
}

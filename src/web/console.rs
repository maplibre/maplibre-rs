extern crate console_error_panic_hook;
use std::panic;

pub fn init_console_error_panic_hook() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}
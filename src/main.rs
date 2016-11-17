extern crate rustyline;
extern crate core;

mod ares;
mod rust_runner;

use std::process;
use ares::{Ares, ReplError, StatusCode};
use std::error::Error;

// TODO Docs
fn main() {
	let mut new_ares = Ares::new();
	let status_code = match new_ares {
		Ok(mut ares) => run(ares),
		Err(err) => {
			println!("Error initializing Ares: {}\n...caused by {:?}", err.description(), err.cause());
			1
		}
	};
	process::exit(status_code);
}

fn run(mut ares: Ares) -> StatusCode {
	let exit_result = ares.init(); 
	match exit_result {
		Ok(status_code) => status_code,
		Err(e) => {
			println!("Unknown error: {:?}\n\nExiting...", e);
			1
		}
	}
}
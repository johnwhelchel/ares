extern crate rustyline;

mod ares;
mod rust_runner;

use std::process;
use ares::{Ares, ReplError};
use std::error::Error;

// TODO Docs
fn main() {
	let mut new_ares = Ares::new();
	match new_ares {
		Ok(mut ares) => run(ares),
		Err(err) => println!("Error initializing Ares: {}\n...caused by {:?}", err.description(), err.cause())
	}
}

fn run(mut ares: Ares) {
	let exit_result = ares.init(); 
	match exit_result {
		Ok(status_code) => {
			process::exit(status_code);
		}
		Err(e) => {
			println!("Unknown error: {:?}\n\nExiting...", e);
			process::exit(1);
		}
	}
}
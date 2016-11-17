extern crate rustyline;

mod ares;

use ares::Ares;

fn main() {
	let mut ares : Ares = Default::default();
	ares.init();
}

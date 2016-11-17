use std::process;

use rustyline::error::ReadlineError;
use rustyline::Editor as Readline;

#[derive(Debug)]
pub struct Ares {
	rl: Readline<()>,
	line_number: u32,
	indent_level: u32,
	interrupted: bool,
}

impl Ares {
	pub fn init(&mut self) -> () {
		print!("fn main() {{...\n");
		loop {
			let prompt = self.prompt();
			let readline = self.rl.readline(&prompt);
			match readline {
				Ok(line) => {
					self.execute_line(line);
				}
				Err(ReadlineError::Interrupted) => {
					self.interrupt_handler();
				}
				Err(ReadlineError::Eof) => {
					self.eof_handler();
				}
				Err(err) => {
					self.interrupted = false;
					println!("Unknown error: {:?}", err);
				}
			}
		}
	}

	fn prompt(&self) -> String {
		format!(" ares:{:03}:{}-> ", self.line_number, self.indent_level)
	}

	fn execute_line(&mut self, line: String) {
		self.interrupted = false;
		self.rl.add_history_entry(&line);
		println!(" {}", line);
	}

	fn eof_handler(&mut self) {
		if self.indent_level == 0 {
			process::exit(0);
		} else {
			self.indent_level -= 1;
		}
	}

	fn interrupt_handler(&mut self) {
		if self.interrupted {
			process::exit(1);
		} else {
			self.interrupted = true;
			println!("CTRL-C sent; send again to exit.");
		}
	}
}

impl Default for Ares {
	fn default() -> Ares {
		Ares {
			rl: Default::default(),
			line_number: 1,
			indent_level: 0,
			interrupted: false,
		}
	}
}

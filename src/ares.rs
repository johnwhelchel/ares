use rust_runner::{Runner, RunnerError};
use rustyline::Editor as Readline;

use rustyline::error::ReadlineError;

use std::error;
use std::fmt;

pub type StatusCode = i32;
pub type Exit = Result<StatusCode, ReplError>;

const STANDARD_PROMPT : &'static str = "->";
const PENDING_PROMPT : &'static str = "-*";

#[derive(Debug)]
pub struct Ares<'a> {
	rl: Readline<()>,
	line_number: u32,
	indent_level: u32,
	interrupted: bool,
	prompt_ending: &'a str,
	runner: Runner,
}

#[derive(Debug)]
pub enum ReplError {
	RustyLine(ReadlineError),
	RustRunner(RunnerError),
}

impl From<RunnerError> for ReplError {
	fn from(source: RunnerError) -> ReplError {
		ReplError::RustRunner(source)
	}
}

impl From<ReadlineError> for ReplError {
	fn from(source: ReadlineError) -> ReplError {
		ReplError::RustyLine(source)
	}
}

impl fmt::Display for ReplError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ReplError::RustyLine(ref err) => err.fmt(f),
			ReplError::RustRunner(ref err) => err.fmt(f),
		}
	}
}

impl error::Error for ReplError {
	fn description(&self) -> &str {
		match *self {
			ReplError::RustyLine(ref err) => err.description(),
			ReplError::RustRunner(ref err) => err.description(),
		}
	}

	fn cause(&self) -> Option<&error::Error> {
		match *self {
			ReplError::RustyLine(ref err) => Some(err),
			ReplError::RustRunner(ref err) => Some(err),
		}
	}
}

impl<'a> Ares<'a> {
	pub fn init(&mut self) -> Exit {
		print!("fn main() {{...\n");
		loop {
			let prompt = self.prompt();
			let readline = self.rl.readline(&prompt);
			let handled_line = match readline {
				Ok(line) => self.execute_line(line),
				Err(ReadlineError::Interrupted) => self.interrupt_handler(),
				Err(ReadlineError::Eof) => self.eof_handler(),
				Err(some_other_error) => Some(Err(ReplError::RustyLine(some_other_error))),
			};
			match handled_line {
				Some(exit) => return exit,
				None => (),
			}
		}
	}

	fn prompt(&self) -> String {
		if self.line_number < 1000 {
			format!(" ares:{:03}:{}{} ", self.line_number, self.indent_level, self.prompt_ending)
		} else {
			format!(" ares:{}:{}{} ", self.line_number, self.indent_level, self.prompt_ending)
		}
	}

	fn execute_line(&mut self, line: String) -> Option<Exit> {
		self.interrupted = false;
		self.rl.add_history_entry(&line);
		self.line_number += 1;
		if line.ends_with(";") {
			self.prompt_ending = STANDARD_PROMPT;
			let result = self.runner.execute(line);
			match result {
				Ok(output) => println!(" {}", output),
				Err(RunnerError::Compilation(message)) => {
					println!(" {}", message)
				},
				Err(RunnerError::Io(err)) => println!("IO error: {}", err)
			}
			None
		} else {
			self.runner.code_lines.push(line);
			self.prompt_ending = PENDING_PROMPT;
			None
		}
	}

	fn eof_handler(&mut self) -> Option<Exit> {
		if self.indent_level == 0 {
			Some(Ok(0))
		} else {
			self.indent_level -= 1;
			None
		}
	}

	fn interrupt_handler(&mut self) -> Option<Exit> {
		if self.interrupted {
			Some(Ok(2))
		} else {
			self.interrupted = true;
			println!("CTRL-C sent; send again to exit.");
			None
		}
	}

	pub fn new() -> Result<Ares<'a>, ReplError> {
		let runner = Runner::new()?;
		Ok(Ares {
			rl: Default::default(),
			runner: runner,
			line_number: 1,
			indent_level: 0,
			prompt_ending: STANDARD_PROMPT,
			interrupted: false,
		})
	}
}

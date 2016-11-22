use rust_runner::{Runner, RunnerError};
use rustyline::Editor as Readline;

use rustyline::error::ReadlineError;
// use unicode_segmentation::UnicodeSegmentation;

// use core::array::FixedSizeArray;
use std::error;
use std::fmt;

pub type StatusCode = i32;
pub type Exit = Result<StatusCode, ReplError>;

const STANDARD_PROMPT : &'static str = "->";
const PENDING_PROMPT : &'static str = "-*";
const ACCEPTABLE_LAST_CHARS : &'static [char] = &[';', '{', '}'];

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
		print!("Ares - simple REPL for the Rust language. All lines must end in a semicolon (;), opening brace ({{), or closing brace (}})\nfn main() {{...\n");
		loop {
			let prompt = self.prompt();
			let readline = self.rl.readline(&prompt);
			let handled_line = match readline {
				Ok(line) => self.handle_line(line),
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

	fn handle_line(&mut self, line: String) -> Option<Exit> {
		let last_char = line.chars().last().unwrap(); // Ok that it's not a grapheme since we're only comparing against single codepoints chars
		self.update_ares_state(&line, &last_char);
		if !(*ACCEPTABLE_LAST_CHARS).iter().any(|c| c.eq(&last_char)) {
			println!("Ares is currently restricted to handling statements that end in {:?} only.", ACCEPTABLE_LAST_CHARS);
			return None;
		}

		if self.should_execute_line(last_char) {
			self.prompt_ending = STANDARD_PROMPT;
			let result = self.runner.execute(line);
			match result {
				Ok(output) => println!("=> {}", output),
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

	fn update_ares_state(&mut self, line: &String, last_char: &char) {
		self.interrupted = false;
		self.rl.add_history_entry(&line);
		self.line_number += 1;
		if *last_char == '}' {
			self.indent_level = if self.indent_level == 0 { 0 } else { self.indent_level - 1 };
		} else if *last_char == '{' {
			self.indent_level += 1;
		}
	}

	fn should_execute_line(&self, last_char: char) -> bool {
		(last_char == ';' || last_char == '}') && self.indent_level == 0
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

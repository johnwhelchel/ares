use std::env::current_dir;
use std::error;
use std::fmt;
use std::fs;
use std::io;
use std::io::{Write, Read};
use std::path::{PathBuf, MAIN_SEPARATOR};
use std::process::Command;
use std::borrow::Cow;

#[derive(Debug)]
pub struct Runner {
	pub code_lines: Vec<String>,
	temp_dir: PathBuf,
	code_file_path: PathBuf,
}

#[derive(Debug)]
pub enum RunnerError {
	Io(io::Error),
	Compilation(String)
}

type RunnerResult<T> = Result<T, RunnerError>;

impl fmt::Display for RunnerError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			RunnerError::Io(ref err) => err.fmt(f),
			RunnerError::Compilation(ref string) => string.fmt(f)
		}
	}
}

impl error::Error for RunnerError {
	fn description(&self) -> &str {
		match *self {
			RunnerError::Io(ref err) => err.description(),
			RunnerError::Compilation(_) => "Compilation failed"
		}
	}

	fn cause(&self) -> Option<&error::Error> {
		match *self {
			RunnerError::Io(ref err) => Some(err),
			RunnerError::Compilation(_) => None
		}
	}
}

impl From<io::Error> for RunnerError {
	fn from(source: io::Error) -> RunnerError {
		RunnerError::Io(source)
	}
}

impl Runner {
	pub fn new() -> RunnerResult<Runner> {
		let temp_dir = Runner::generate_temp_dir()?;
		let code_file_path = Runner::generate_code_file(&temp_dir)?;
		Ok(Runner {
			code_lines: Default::default(),
			temp_dir: temp_dir,
			code_file_path: code_file_path,
		})
	}

	pub fn execute(&mut self, next_line: String) -> RunnerResult<String> {
		self.code_lines.push(next_line);
		let result = self.run_code();
		match result {
			Err(RunnerError::Compilation(_)) => {
				self.code_lines.pop();
				result
			},
			_ => result 
		}
	}

	pub fn loc(&self) -> usize {
		self.code_lines.len()
	}

	fn run_code(&self) -> RunnerResult<String> {
		// self.verify_code_compiles()?;
		self.compile_code_with_print()?;
		self.execute_code()
	}

	// fn verify_code_compiles(&self) -> RunnerResult<()> {
	// 	self.write_code()?;
	// 	self.compile_code()?;
	// 	Ok(())
	// }

	fn compile_code_with_print(&self) -> RunnerResult<()> {
		self.write_code_with_print()?;
		self.compile_code()?;
		Ok(())
	}

	// We'll want to return more info from this eventually. We can also avoid rewriting the entire file...
	fn write_code_with_print(&self) -> RunnerResult<()> {
		let mut file = fs::OpenOptions::new().write(true).truncate(true).open(self.code_file_path.as_path())?;
		let mut indent_level = 0i8;
		let mut beginning_of_scope = &self.code_lines[0][..];
		file.write(b"fn main() {\n")?;
		for (i, l) in self.code_lines.iter().enumerate() {
			let is_last_line = i == self.loc() - 1;
			let mut new_line = Cow::Borrowed(l);
			let last_char = new_line.chars().last().unwrap_or(' ');

			if last_char == '{' && indent_level == 0 {
				beginning_of_scope = l;
			}
			if is_last_line {
				new_line = Runner::adjust_last_line(new_line, beginning_of_scope, last_char);
			}
			if last_char == '}' {
				indent_level -= 1;
			}
			for _ in -1..indent_level {
				file.write(b"\t")?;
			}
			file.write(new_line.as_bytes())?;
			file.write(b"\n")?;
			if last_char == '{' {
				indent_level += 1;
			}
		}
		file.write(b"}")?;
		Ok(())
	}

	fn adjust_last_line<'a>(l : Cow<'a, String>, beginning_of_scope : &str, last_char : char) -> Cow<'a, String> {
		if last_char == ';' {
			let split_by_eq = l.split("=").collect::<Vec<&str>>();
			let is_expression = split_by_eq.len() == 1;
			if is_expression {
				let mut split_by_whitespace = split_by_eq[0].split_whitespace();
				let is_use = split_by_whitespace.next().unwrap() == "use";
				if is_use {
					let used_value = split_by_whitespace.next().unwrap();
					return Cow::Owned(format!("{}\nprint!(\"Using {}\");", l, used_value))
				} else {
					return Cow::Owned(format!("let __ares_tmp = {}\n\tprint!(\"{{:?}}\", __ares_tmp);", l));
				}
			} else {
				let var_name = split_by_eq[0].split_whitespace().last().unwrap();
				return Cow::Owned(format!("{}\n\tprint!(\"{{:?}}\", {});", l, var_name));
			}
		} else
			if beginning_of_scope.starts_with("fn ") 
			|| beginning_of_scope.starts_with("impl ") 
			|| beginning_of_scope.starts_with("trait ")
			|| beginning_of_scope.starts_with("struct ") {
				let mut to_print = String::from(beginning_of_scope);
				to_print.pop(); // Remove ending brace
				return Cow::Owned(format!("}}\n\tprint!(\"{}\");", to_print))
			}
		l
	}

	// fn write_code(&self) -> RunnerResult<()> {
	// 	let mut file = fs::OpenOptions::new().write(true).open(self.code_file_path.as_path())?;
	// 	file.write(b"fn main() {\n")?;
	// 	for l in self.code_lines.iter() {
	// 		file.write(b"\t")?;
	// 		file.write(l.as_bytes())?;
	// 		file.write(b"\n")?;
	// 	}
	// 	file.write(b"}")?;
	// 	Ok(())
	// }

	// We'll want to return more info from this eventually
	fn compile_code(&self) -> RunnerResult<()> {
		let output = Command::new("rustc")
			.arg("ares.rs")
			.current_dir(self.temp_dir.as_path())
			.output()?;
		match output.status.code() {
			Some(0) => Ok(()),
			_ => {
				let error_to_show = String::from_utf8(output.stderr).unwrap()
					.split("\nerror: aborting").next().unwrap()
					.to_string();
				self.print_full_code();
				Err(RunnerError::Compilation(error_to_show))
			}
		}
	}

	fn print_full_code(&self) {
		let mut file = fs::OpenOptions::new().read(true).open(self.code_file_path.as_path()).unwrap();
		let mut s = String::new();
		file.read_to_string(&mut s).unwrap();
		println!("Full code: \n{}", s);
	}

	// We'll want to return more info from this eventually
	fn execute_code(&self) -> RunnerResult<String> {
		let command = format!(".{}ares", MAIN_SEPARATOR);
		let output = Command::new(command)
			.current_dir(self.temp_dir.as_path())
			.output()?;
		match output.status.code() {
			Some(0) => Ok(String::from_utf8_lossy(&output.stdout).into_owned()),
			_ => panic!("TODO Runtime error in code")
		}
	}

	fn generate_temp_dir() -> RunnerResult<PathBuf> {
		let mut curr_dir = current_dir()?.to_path_buf();
		curr_dir.push(".tmp_ares");
		fs::create_dir(curr_dir.as_path())?;
		Ok(curr_dir)
	}

	fn generate_code_file(dir: &PathBuf) -> RunnerResult<PathBuf> {
		let mut code_file_path = dir.clone();
		code_file_path.push("ares.rs");
		fs::OpenOptions::new().create(true).write(true).open(code_file_path.as_path())?;
		Ok(code_file_path)
	}
}

impl Drop for Runner {
	fn drop(&mut self) {
		fs::remove_dir_all(self.temp_dir.as_path()).unwrap();
	}
}
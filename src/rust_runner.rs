use std::env::current_dir;
use std::error;
use std::fmt;
use std::fs;
use std::io;
use std::io::Write;
use std::path::{PathBuf, MAIN_SEPARATOR};
use std::process::Command;

#[derive(Debug)]
pub struct Runner {
	pub code_lines: Vec<String>,
	temp_dir: PathBuf,
	code_file_path: PathBuf,
}

#[derive(Debug)]
pub enum RunnerError {
	Io(io::Error),
}

type RunnerResult<T> = Result<T, RunnerError>;

impl fmt::Display for RunnerError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			RunnerError::Io(ref err) => err.fmt(f),
		}
	}
}

impl error::Error for RunnerError {
	fn description(&self) -> &str {
		match *self {
			RunnerError::Io(ref err) => err.description(),
		}
	}

	fn cause(&self) -> Option<&error::Error> {
		match *self {
			RunnerError::Io(ref err) => Some(err),
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
		self.run_code()
	}

	fn run_code(&self) -> RunnerResult<String> {
		self.write_code()?;
		self.compile_code()?;
		self.execute_code()
	}

	// We'll want to return more info from this eventually
	fn write_code(&self) -> RunnerResult<()> {
		let mut file = fs::OpenOptions::new().write(true).open(self.code_file_path.as_path())?;
		file.write(b"fn main() {\n")?;
		for l in &self.code_lines {
			write!(&mut file, "\t{}\n", *l)?;
		}
		file.write(b"}")?;
		Ok(())
	}

	// We'll want to return more info from this eventually
	fn compile_code(&self) -> RunnerResult<()> {
		let output = Command::new("rustc")
			.arg("ares.rs")
			.current_dir(self.temp_dir.as_path())
			.output()?;
		match output.status.code() {
			Some(0) => Ok(()),
			_ => panic!("TODO Compilation failed. Use StdOut and StdErr")
		}
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
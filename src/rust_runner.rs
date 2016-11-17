use std::path::{Path, PathBuf};
use std::fs;
use std::env::current_dir;
use std::io;
use std::fmt;
use std::error;

#[derive(Debug)]
pub struct Runner {
	pub code_lines: Vec<String>,
	temp_dir: PathBuf,
	code_file_path: PathBuf
}

#[derive(Debug)]
pub enum RunnerError {
	Io(io::Error)
}

impl fmt::Display for RunnerError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			RunnerError::Io(ref err) => err.fmt(f)
		}
	}
}

impl error::Error for RunnerError {
	fn description(&self) -> &str {
		match *self {
			RunnerError::Io(ref err) => err.description()
		}
	}

	fn cause(&self) -> Option<&error::Error> {
		match *self {
			RunnerError::Io(ref err) => Some(err)
		}
	}
}

impl From<io::Error> for RunnerError {
	fn from(source: io::Error) -> RunnerError {
		RunnerError::Io(source)
	}
}

impl Runner {
	pub fn new() -> Result<Runner, RunnerError> {
		let temp_dir = Runner::generate_temp_dir()?;
		let code_file_path = Runner::generate_code_file(&temp_dir)?;
		Ok(Runner {
			code_lines: Default::default(),
			temp_dir: temp_dir,
			code_file_path: code_file_path
		})
	}

	pub fn execute(&mut self, next_line: String) -> Result<String, RunnerError> {
		self.code_lines.push(next_line);
		Ok("Worked".to_string())
	}

	fn generate_temp_dir() -> Result<PathBuf, RunnerError> {
		let mut curr_dir = current_dir()?.to_path_buf();
		curr_dir.push(".tmp_ares");
		fs::create_dir(curr_dir.as_path())?;
		Ok(curr_dir)
	}

	fn generate_code_file(dir: &PathBuf) -> Result<PathBuf, RunnerError> {
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
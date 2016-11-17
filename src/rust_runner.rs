use std::path::{Path, PathBuf};
use std::fs;
use std::env::current_dir;
use std::io;
use std::fmt;
use std::error;

#[derive(Debug)]
pub struct Runner<'a> {
	pub code_lines: Vec<String>,
	temp_dir: Option<&'a Path>,
	code_file_path: Option<&'a Path>
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

impl<'a> Runner<'a> {
	pub fn new() -> Result<Runner<'a>, RunnerError> {
		let mut runner = Runner {
			code_lines: Default::default(),
			temp_dir: None,
			code_file_path: None
		};
		runner.init();
		Ok(runner)
	}

	pub fn execute(&mut self, next_line: String) -> Result<String, RunnerError> {
		self.code_lines.push(next_line);
		Ok("Worked".to_string())
	}

	fn init(&mut self) -> Result<(), RunnerError> {
		self.generate_temp_dir()?;
		self.generate_code_file()?;
		Ok(())
	}

	fn generate_temp_dir(&mut self) -> Result<(), RunnerError> {
		let mut curr_dir = current_dir()?.to_path_buf();
		curr_dir.push(".tmp_ares");
		let curr_dir = curr_dir.as_path();
		fs::create_dir(curr_dir)?;
		self.temp_dir = Some(&curr_dir);
		Ok(())
	}

	fn generate_code_file(&mut self) -> Result<(), RunnerError> {
		let mut code_file_path = self.temp_dir.unwrap().to_path_buf();
		code_file_path.push("ares.rs");
		let code_file_path = code_file_path.as_path();
		fs::OpenOptions::new().create(true).write(true).open(code_file_path)?;
		self.code_file_path = Some(&code_file_path);
		Ok(())		
	}
}

impl<'a> Drop for Runner<'a> {
	fn drop(&mut self) {
		fs::remove_dir_all(self.temp_dir.unwrap()).unwrap();
	}
}
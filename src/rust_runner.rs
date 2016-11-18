use std::env::current_dir;
use std::error;
use std::fmt;
use std::fs;
use std::io;
use std::io::{Write, Read};
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

	fn verify_code_compiles(&self) -> RunnerResult<()> {
		self.write_code()?;
		self.compile_code()?;
		Ok(())
	}

	fn compile_code_with_print(&self) -> RunnerResult<()> {
		self.write_code_with_print()?;
		self.compile_code()?;
		Ok(())
	}

	// We'll want to return more info from this eventually. We can also avoid rewriting the entire file...
	fn write_code_with_print(&self) -> RunnerResult<()> {
		let mut file = fs::OpenOptions::new().write(true).open(self.code_file_path.as_path())?;
		file.write(b"fn main() {\n")?;
		for (i, l) in self.code_lines.iter().enumerate() {
			let mut new_line = l.clone();
			if i == self.loc() - 1 {
				let mut var_name = "__ares_tmp";
				let split_by_eq = l.split("=").collect::<Vec<&str>>();
				if split_by_eq.len() == 1 {
					new_line = format!("let __ares_tmp = {};", new_line);
				} else {
					let before_first_eq = split_by_eq[0];
					var_name = before_first_eq.split_whitespace().last().unwrap();
				}
				new_line = format!("{}\n\tprint!(\"{{:?}}\", {});", new_line, var_name);
			}
			file.write(b"\t")?;
			file.write(new_line.as_bytes())?;
			file.write(b"\n")?;
		}
		file.write(b"}")?;
		Ok(())
	}

	fn write_code(&self) -> RunnerResult<()> {
		let mut file = fs::OpenOptions::new().write(true).open(self.code_file_path.as_path())?;
		file.write(b"fn main() {\n")?;
		for l in self.code_lines.iter() {
			file.write(b"\t")?;
			file.write(l.as_bytes())?;
			file.write(b"\n")?;
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
		println!("Full code: {}", s);
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
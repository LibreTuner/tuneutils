extern crate serde_yaml;

use std::rc::Rc;
use std::result;
use std::default;
use std::collections::HashMap;

use std::path::Path;
use std::fs;
use std::convert;
use std::io::{self, Read};

#[serde(rename_all = "lowercase")]
#[derive(Debug, Serialize, Deserialize)]
pub enum DownloadMode {
	Mazda1,
	None,
}

impl default::Default for DownloadMode {
	fn default() -> DownloadMode {
		DownloadMode::None
	}
}

#[serde(rename_all = "lowercase")]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum FlashMode {
	Mazda1,
	None,
}

impl default::Default for FlashMode {
	fn default() -> FlashMode {
		FlashMode::None
	}
}

#[serde(rename_all = "lowercase")]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum LogMode {
	Uds,
	None,
}

impl default::Default for LogMode {
	fn default() -> LogMode {
		LogMode::None
	}
}

#[serde(rename_all = "lowercase")]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Endianness {
	Big,
	Little,
}

#[serde(rename_all = "lowercase")]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum DataType {
	Uint8,
    Uint16,
    Uint32,
    Uint64,
    Float32,
    Float64,
    Int8,
    Int16,
    Int32,
    Int64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Identifier {
	pub offset: u32,
	pub data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Axis {
	pub name: String,
	pub id: String,
	#[serde(rename = "type")]
	pub axis_type: String,

	#[serde(default)]
	pub start: f64,
	#[serde(default)]
	pub increment: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Table {
	pub id: usize,
	pub name: String,
	pub description: String,
	pub category: String,
	pub data_type: DataType,

	#[serde(default = "default_table_dimension")]
	pub width: usize,
	#[serde(default = "default_table_dimension")]
	pub height: usize,

	#[serde(default = "max_table_constraint")]
	pub maximum: f64,
	#[serde(default = "min_table_constraint")]
	pub minimum: f64,

	#[serde(default)]
	pub axis_x_id: String,
	#[serde(default)]
	pub axis_y_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pid {
	pub name: String,
	pub description: String,
	pub formula: String,
	pub unit: String,
	pub datatype: DataType,
	pub id: u32,
	pub code: u16,
}

fn default_table_dimension() -> usize {
	1
}

fn max_table_constraint() -> f64 {
	use std::f64;
	f64::MAX
}

fn min_table_constraint() -> f64 {
	use std::f64;
	f64::MIN
}


/// A specific model of an ECU e.g. Mazdaspeed6 made in 2006 for California
#[derive(Debug, Deserialize, Serialize)]
pub struct Model {
	pub id: String,
	pub name: String,

	#[serde(rename = "tables")]
	#[serde(default)]
	// <id, offset>
	pub table_offsets: HashMap<usize, usize>,

	#[serde(rename = "axes")]
	#[serde(default)]
	// <id, offset>
	pub axis_offsets: HashMap<String, usize>,

	#[serde(default)]
	pub identifiers: Vec<Identifier>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Transfer {
	#[serde(default)]
	pub download_mode: DownloadMode,
	#[serde(default)]
	pub flash_mode: FlashMode,
	// Security key
	pub key: String,
	// Server ID for ISO-TP requests
	pub server_id: u16,
}

/// A specific platform e.g. Mazdaspeed6
#[derive(Debug, Deserialize, Serialize)]
pub struct Main {
	pub name: String,
	pub id: String,

	transfer: Transfer,
	pub baudrate: u32,
	#[serde(default)]
	pub log_mode: LogMode,
	pub endianness: Endianness,

	// Flash region
	pub flash_offset: usize,
	pub flash_size: usize,

	pub rom_size: usize,

	pub tables: Vec<Table>,
	pub pids: Vec<Pid>,
	pub vins: Vec<String>,

	#[serde(skip)]
	pub models: Vec<Rc<Model>>,
}

impl Main {
	/// Searches for a model that matches the id
	pub fn find(&self, id: &str) -> Option<&Rc<Model>> {
		self.models.iter().find(|&model| model.id == id)
	}
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	Yaml(serde_yaml::Error),
	Io(io::Error),
}

impl convert::From<serde_yaml::Error> for Error {
	fn from(err: serde_yaml::Error) -> Error {
		Error::Yaml(err)
	}
}

impl convert::From<io::Error> for Error {
	fn from(err: io::Error) -> Error {
		Error::Io(err)
	}
}

pub struct Definitions {
	pub definitions: Vec<Rc<Main>>,
}

impl default::Default for Definitions {
	fn default() -> Definitions {
		Definitions {definitions: Vec::new()}
	}
}

impl Definitions {
	pub fn load(&mut self, base: &Path) -> Result<()> {
		for entry in fs::read_dir(base)? {
			let entry = entry?;
			let path = entry.path();
			if path.is_dir() {
				// Check for a main.toml
				let main_path = path.join("main.yaml");
				if !main_path.is_file() {
					continue;
				}

				// Open it
				let mut contents = fs::read_to_string(main_path)?;
				let mut main: Main = serde_yaml::from_str(&contents)?;

				// Load models
				for entry in fs::read_dir(path)? {
					let entry = entry?;
					let path = entry.path();

					if !path.is_file() { continue; }
					if let Some(ext) = path.extension() {
						if ext != "yaml" { continue; }
					} else {
						continue;
					}
					if let Some(name) = path.file_name() {
						if name == "main.yaml" { continue; }
					}

					contents.clear();
					{
						let mut file = fs::File::open(path)?;
						file.read_to_string(&mut contents)?;
					}
					let model: Model = serde_yaml::from_str(&contents)?;
					main.models.push(Rc::new(model));
				}

				self.definitions.push(Rc::new(main));
			}
		}

		Ok(())
	}

	/// Searches for the main definition with the matching id
	pub fn find(&self, id: &str) -> Option<&Rc<Main>> {
		self.definitions.iter().find(|&def| def.id == id)
	}
}
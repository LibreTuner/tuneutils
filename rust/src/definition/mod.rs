extern crate serde_yaml;

use serde::Deserialize;
use serde::de::{self, Visitor};
use std::fmt;
use std::result;
use std::default;

use std::path::Path;
use std::fs::{self, DirEntry};
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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
pub enum Endianness {
	Big,
	Little,
}

#[serde(rename_all = "lowercase")]
#[derive(Debug, Serialize, Deserialize)]
pub enum TableType {
	Uint8,
    Uint16,
    Uint32,
    Float,
    Int8,
    Int16,
    Int32,
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
	pub id: isize,
	pub name: String,
	pub description: String,
	pub category: String,
	pub data_type: TableType,

	#[serde(default = "default_table_dimension")]
	pub width: isize,
	#[serde(default = "default_table_dimension")]
	pub height: isize,

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
	pub id: u32,
	pub code: u16,
}

fn default_table_dimension() -> isize {
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

#[derive(Debug, Deserialize, Serialize)]
pub struct TableOffset {
	pub id: usize,
	pub offset: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AxisOffset {
	pub id: usize,
	pub offset: usize,
}

/// A specific model of an ECU e.g. Mazdaspeed6 made in 2006 for California
#[derive(Debug, Deserialize, Serialize)]
pub struct Model {
	pub id: String,
	pub name: String,

	#[serde(rename = "table")]
	#[serde(default)]
	pub table_offsets: Vec<TableOffset>,

	#[serde(rename = "axis")]
	#[serde(default)]
	pub axis_offsets: Vec<AxisOffset>,

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

	// Flash region
	pub flash_offset: usize,
	pub flash_size: usize,

	pub rom_size: usize,

	pub tables: Vec<Table>,
	pub pids: Vec<Pid>,
	pub vins: Vec<String>,

	#[serde(skip)]
	pub models: Vec<Model>,
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
	pub definitions: Vec<Main>,
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
				let mut contents = String::new();
				{
					let mut file = fs::File::open(main_path)?;
					file.read_to_string(&mut contents)?;
				}
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
					main.models.push(model);
				}

				self.definitions.push(main);
			}
		}

		Ok(())
	}
}
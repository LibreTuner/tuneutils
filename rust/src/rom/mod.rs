extern crate serde_yaml;

use std::rc::Rc;
use std::result;
use std::io;
use std::fs;
use std::convert;
use std::path::{Path, PathBuf};

pub mod tune;

use definition;



pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	Yaml(serde_yaml::Error),
	Io(io::Error),
	InvalidMainId,
	InvalidModelId,
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



pub struct Rom {
	model: Rc<definition::Model>,
	main: Rc<definition::Main>,
	name: String,
	data: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RomMeta {
	name: String,
	id: String,
	main_name: String,
	model_name: String,

	#[serde(skip)]
	data_path: PathBuf,
}

impl Rom {
	pub fn load(meta: &RomMeta, definitions: &definition::Definitions) -> Result<Rom> {
		let main = definitions.find(&meta.main_name).ok_or(Error::InvalidMainId)?;
		let model = main.find(&meta.model_name).ok_or(Error::InvalidModelId)?;
		let data = fs::read(&meta.data_path)?;

		Ok(Rom {
			model: model.clone(),
			main: main.clone(),
			name: meta.name.clone(),
			data,
		})
	}
}

pub struct Manager {
	roms: Vec<RomMeta>,
}

impl Manager {
	pub fn load(&mut self, base: &Path) -> Result<()> {
		for entry in fs::read_dir(base)? {
			let entry = entry?;

			if !entry.file_type()?.is_file() {
				continue;
			}

			let path = entry.path();
			if let Some(ext) = path.extension() {
				if ext != "yaml" { continue; }
			} else {
				continue;
			}

			let contents = fs::read_to_string(path)?;
			let mut meta: RomMeta = serde_yaml::from_str(&contents)?;
			meta.data_path = base.join(&meta.id);
			self.roms.push(meta);
		}

		Ok(())
	}
}
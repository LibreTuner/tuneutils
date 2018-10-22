extern crate serde_yaml;

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use error::{Error, Result};

pub mod tune;

use definition;


#[derive(Debug)]
pub struct Rom {
	meta: RomMeta,
	data: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RomData {
	name: String,
	id: String,
	main_name: String,
	model_name: String,
}

#[derive(Debug, Clone)]
pub struct RomMeta {
	pub name: String,
	pub id: String,
	pub model: Rc<definition::Model>,
	pub main: Rc<definition::Main>,

	pub data_path: PathBuf,
}


pub struct RomManager {
	roms: Vec<RomMeta>,
	loaded_roms: RefCell<HashMap<String, Weak<Rom>>>,
}

impl RomManager {
	pub fn load(&mut self, base: &Path, definitions: &definition::Definitions) -> Result<()> {
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
			let data: RomData = serde_yaml::from_str(&contents)?;
			let main = definitions.find(&data.main_name).ok_or(Error::InvalidMainId)?;
			let model = main.find(&data.model_name).ok_or(Error::InvalidModelId)?;

			self.roms.push(RomMeta {
				data_path: base.join(&data.id),
				
				id: data.id,
				name: data.name,
				main: main.clone(),
				model: model.clone(),
			});
		}

		Ok(())
	}

	/// Searches for a ROM meta with the specified id
	pub fn search(&self, id: &str) -> Option<&RomMeta> {
		self.roms.iter().find(|ref rom| rom.id == id)
	}

	/// Loads a ROM or retrieves it from the cache.
	pub fn load_rom(&self, meta: &RomMeta) -> Result<Rc<Rom>> {
		// Check if the ROM is cached
		let mut loaded_roms = self.loaded_roms.borrow_mut();
		if let Some(ptr) = loaded_roms.get(&meta.id) {
			// Check if the pointer is valid
			if let Some(rom) = ptr.upgrade() {
				return Ok(rom);
			}
		}
		// The ROM is not cached so we load it (and cache it)
		let data = fs::read(&meta.data_path)?;

		let rom = Rc::new(Rom {
			meta: meta.clone(),
			data,
		});
		// Cache it
		loaded_roms.insert(meta.name.clone(), Rc::downgrade(&rom));
		Ok(rom)
	}
}
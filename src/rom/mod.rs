extern crate serde_yaml;

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
	error::{Error, Result},
	definition,
};

pub mod tune;


#[derive(Debug)]
pub struct Rom {
	meta: RomMeta,
	data: Vec<u8>,
}

impl Rom {
	/// Saves ROM data to file
	pub fn save(&self) -> Result<()> {
		fs::write(&self.meta.data_path, &self.data)?;
		Ok(())
	}

	/// Loads ROM from file. Internal use only; use `RomManager::load_rom`
	fn load(meta: RomMeta) -> Result<Rom> {
		let data = fs::read(&meta.data_path)?;
		Ok(Rom {
			meta,
			data,
		})
	}
}

#[derive(Debug, Clone)]
pub struct RomMeta {
	pub name: String,
	pub id: String,
	pub model: Arc<definition::Model>,
	pub platform: Arc<definition::Main>,

	pub data_path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SerializedRomMeta {
	pub name: String,
	pub id: String,
	pub model: String,
	pub platform: String,
}

impl RomMeta {
	/// Converts the RomMeta to a serialized version for saving
	pub fn to_serialized(&self) -> SerializedRomMeta {
		SerializedRomMeta {
			name: self.name.clone(),
			id: self.id.clone(),
			model: self.model.id.clone(),
			platform: self.platform.id.clone(),
		}
	}
}

pub struct RomManager {
	base: PathBuf,
	pub roms: Vec<RomMeta>,
	loaded_roms: RefCell<HashMap<String, Weak<Rom>>>,
}


impl RomManager {
	pub fn new(base: PathBuf) -> RomManager {
		RomManager {
			base,
			roms: Vec::new(),
			loaded_roms: RefCell::new(HashMap::new()),
		}
	}

	/// Loads rom metadata
	pub fn load(&mut self, definitions: &definition::Definitions) -> Result<()> {
		// Clear previous data
		self.roms.clear();
		if !self.base.exists() {
			return Ok(());
		}

		let meta_path = self.base.join("roms.yaml");
		if !meta_path.is_file() {
			return Ok(());
		}

		let rom_data: Vec<SerializedRomMeta> = serde_yaml::from_str(&fs::read_to_string(&meta_path)?)?;

		for meta in rom_data {
			let platform = definitions.find(&meta.platform).ok_or(Error::InvalidPlatformId)?;
			let model = platform.find(&meta.model).ok_or(Error::InvalidModelId)?;

			self.roms.push(RomMeta {
				data_path: self.base.join(&meta.id),
				
				id: meta.id,
				name: meta.name,
				platform: platform.clone(),
				model: model.clone(),
			});
		}

		Ok(())
	}

	/// Saves all metadata
	pub fn save_meta(&self) -> Result<()> {
		// Convert to serialized format
		let serialized: Vec<SerializedRomMeta> = self.roms.iter().map(|x| x.to_serialized()).collect();

		fs::write(&self.base.join("roms.yaml"), serde_yaml::to_string(&serialized)?)?;
		Ok(())
	}

	/// Searches for a ROM meta with the specified id
	pub fn search(&self, id: &str) -> Option<&RomMeta> {
		self.roms.iter().find(|ref rom| rom.id == id)
	}

	/// Creates a new ROM, adds it to the database. It will NOT be saved.
	/// Note: `save_meta()` should be called as the ROM metadata will not be saved by this function.
	/// `Rom::save()` should also be called to save the ROM data.
	/// If another ROM with the same id already exists, undefined behavior will occur.
	pub fn new_rom(&mut self, name: String, id: String, platform: Arc<definition::Main>, model: Arc<definition::Model>, data: Vec<u8>) -> Rc<Rom> {
		let meta = RomMeta {
			data_path: self.base.join(&id),

			name,
			id,
			model,
			platform,
		};

		self.roms.push(meta.clone());
		let rom = Rc::new(Rom {
			meta,
			data,
		});
		self.loaded_roms.borrow_mut().insert(rom.meta.id.clone(), Rc::downgrade(&rom));
		rom
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
		let rom = Rc::new(Rom::load(meta.clone())?);
		// Cache it
		loaded_roms.insert(meta.id.clone(), Rc::downgrade(&rom));
		Ok(rom)
	}
}
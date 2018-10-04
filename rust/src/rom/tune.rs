extern crate serde_yaml;

use std::fs;
use std::path::{PathBuf, Path};
use super::{Error, Result};

#[derive(Debug, Deserialize, Serialize)]
pub struct TuneMeta {
	name: String,
	id: String,
	rom_id: String,

	#[serde(skip)]
	data_path: PathBuf,
}

pub struct Manager {
	tunes: Vec<TuneMeta>,
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
			let mut meta: TuneMeta = serde_yaml::from_str(&contents)?;
			meta.data_path = base.join(&meta.id);
			self.tunes.push(meta);
		}

		Ok(())
	}
}
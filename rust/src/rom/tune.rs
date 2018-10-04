extern crate serde_yaml;
extern crate bit_vec;

use std::fs;
use std::path::{PathBuf, Path};
use super::{Error, Result, Rom};
use definition::TableType;

use self::bit_vec::BitVec;

pub enum TableEntry {
	U64(u64),
	U32(u32),
	U16(u16),
	U8(u8),
	I64(i64),
	I32(i32),
	I16(i16),
	I8(i8),
	F64(f64),
	F32(f32),
}

pub trait TableData {
	fn get(&self, x: usize, y: usize) -> Option<TableEntry>;
	fn set(&mut self, x: usize, y: usize, entry: TableEntry);

	fn height(&self) -> usize;
	fn width(&self) -> usize;
}

pub struct TableDataBase<T> {
	maximum: T,
	minimum: T,
	height: usize,
	width: usize,
	data: Vec<T>,
}

impl<T> TableData for TableDataBase<T> {
	fn get(&self, x: usize, y: usize) -> Option<TableEntry> {
		if x >= self.width || y >= self.height {
			return None;
		}
		return Some()
	}

	fn set(&mut self, x: usize, y: usize, entry: TableEntry) {
		if x >= self.width || y >= self.height {
			return None;
		}


	}

	fn height(&self) -> usize {
		self.height
	}

	fn width(&self) -> usize {
		self.width
	}
}

pub struct Table {
	data_type: TableType,
	dirty: bool,
	modified: BitVec,

	data: Box<TableData>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TuneMeta {
	name: String,
	id: String,
	rom_id: String,

	#[serde(skip)]
	data_path: PathBuf,
}

pub struct Tune<'a> {
	rom: &'a Rom,
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
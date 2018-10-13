#![feature(nll)]

extern crate serde_yaml;
extern crate num_traits;
extern crate byteorder;
extern crate bv;
use self::bv::BitVec;

use std::fs;
use std::path::{PathBuf, Path};
use std::rc::Rc;
use std::collections::HashMap;

use self::num_traits::{Num, PrimInt, AsPrimitive};
use self::byteorder::{ByteOrder, BigEndian, LittleEndian, ReadBytesExt};

use super::{Error, Result, Rom, RomManager};

use definition::{DataType, Endianness};
use definition::Table as TableDefinition;

use numvariant::NumVariant;




pub struct TableMeta {
	name: String,
	description: String,
}

pub fn deserialize_table<O: ByteOrder>(datatype: DataType, data: &[u8], size: usize) -> Result<Vec<NumVariant>> {
	let mut reader = data;
	let mut data = Vec::new();
	match datatype {
		DataType::Uint8 => {
			for _ in 0..size {
				data.push(NumVariant::from(reader.read_u8()?));
			}
		},
		DataType::Uint16 => {
			for _ in 0..size {
				data.push(NumVariant::from(reader.read_u16::<O>()?));
			}
		},
		DataType::Uint32 => {
			for _ in 0..size {
				data.push(NumVariant::from(reader.read_u32::<O>()?));
			}
		},
		DataType::Uint64 => {
			for _ in 0..size {
				data.push(NumVariant::from(reader.read_u64::<O>()?));
			}
		}
		DataType::Int8 => {
			for _ in 0..size {
				data.push(NumVariant::from(reader.read_i8()?));
			}
		},
		DataType::Int16 => {
			for _ in 0..size {
				data.push(NumVariant::from(reader.read_i16::<O>()?));
			}
		},
		DataType::Int32 => {
			for _ in 0..size {
				data.push(NumVariant::from(reader.read_i32::<O>()?));
			}
		},
		DataType::Int64 => {
			for _ in 0..size {
				data.push(NumVariant::from(reader.read_i64::<O>()?));
			}
		},
		DataType::Float32 => {
			for _ in 0..size {
				data.push(NumVariant::from(reader.read_f32::<O>()?));
			}
		},
		DataType::Float64 => {
			for _ in 0..size {
				data.push(NumVariant::from(reader.read_f64::<O>()?));
			}
		},
		_ => unimplemented!()
	}
	Ok(data)
}

pub struct Table {
	pub data_type: DataType,
	pub dirty: bool,
	modified: BitVec,
	data: Vec<NumVariant>,
	height: usize,

	meta: TableMeta,
}

impl Table {
	/// Loads a table from the definition and raw data
	pub fn load_raw(definition: &TableDefinition, data: &[u8], endianness: Endianness) -> Result<Table> {
		let size = definition.width * definition.height;
		let data = match endianness {
			Endianness::Big => deserialize_table::<BigEndian>(definition.data_type, data, size)?,
			Endianness::Little => deserialize_table::<LittleEndian>(definition.data_type, data, size)?,
		};
		Ok(Table {
			data_type: definition.data_type,
			dirty: false,
			modified: BitVec::new_fill(false, size as u64),
			data,
			height: definition.height,
			meta: TableMeta {
				name: definition.name.clone(),
				description: definition.description.clone(),
			},
		})
	}

	pub fn height(&self) -> usize {
		self.height
	}

	pub fn width(&self) -> usize {
		self.data.len() / self.height
	}

	/// Returns true if the data at (`x`, `y`) has been modified
	pub fn modified(&self, x: usize, y: usize) -> bool {
		self.modified[(y * self.height + x) as u64]
	}

	pub fn name(&self) -> &str {
		&self.meta.name
	}

	pub fn description(&self) -> &str {
		&self.meta.description
	}

	/// Returns true if this is a two-dimensional table
	pub fn is_2d(&self) -> bool {
		self.height > 1
	}

	/// Returns true if this is a one-dimensional table
	pub fn is_1d(&self) -> bool {
		self.height == 1
	}

	/// Returns true if this table has one entry
	pub fn is_single(&self) -> bool {
		self.data.len() == 1
	}

	pub fn set(&mut self, x: usize, y: usize, data: NumVariant) {
		// TODO: Convert data to the correct type
		self.data[y * self.height + x] = data;
	}

	/// Expects the data to be in range. If not, it will panic.
	pub fn get(&self, x: usize, y: usize) -> NumVariant {
		self.data[y * self.height + x]
	}
}


#[derive(Debug, Deserialize, Serialize)]
pub struct TuneMeta {
	name: String,
	id: String,
	rom_id: String,

	#[serde(skip)]
	data_path: PathBuf,
}

pub struct Tune {
	pub rom: Rc<Rom>,
	pub tables: HashMap<usize, Table>,
}

impl Tune {
	pub fn load(meta: &TuneMeta, roms: &RomManager) -> Result<Tune> {
		// Search for the ROM
		let rom_meta = roms.search(&meta.rom_id).ok_or(Error::InvalidRomId)?;
		let rom = roms.load_rom(rom_meta)?;

		let mut tables = HashMap::new();
		// TODO: Load tables from file

		Ok(Tune {
			rom: rom.clone(),
			tables,
		})
	}

	/// Gets a table. Returns Error::NotLoaded even if the id is invalid
	pub fn get_table(&self, id: usize) -> Result<&Table> {
		self.tables.get(&id).ok_or(Error::NotLoaded)
	}

	/// Loads a table
	pub fn load_table(&mut self, id: usize) -> Result<&Table> {
		// Search for the table id
		if let Some(table_def) = self.rom.meta.main.tables.iter().find(|ref table| table.id == id) {
			// Get the offset
			let offset = self.rom.meta.model.table_offsets.get(&id).ok_or(Error::NoTableOffset)?;

			// Load the table from the ROM
			let table = Table::load_raw(table_def, &self.rom.data[*offset..], self.rom.meta.main.endianness)?;
			self.tables.insert(id, table);
			// Unwrap because we just inserted it
			return Ok(self.tables.get(&id).unwrap());
		}
		// The table does not exist
		Err(Error::InvalidTableId)
	}

	/// Gets or loads a table.
	pub fn get_or_load_table(&mut self, id: usize) -> Result<&Table> {
		// This is an ugly pattern but we can't fix this without NLL
		if self.tables.contains_key(&id) {
			return self.tables.get(&id).ok_or(Error::InvalidTableId); // This should never error
		}
		self.load_table(id)
	}
}

pub struct TuneManager {
	tunes: Vec<TuneMeta>,
}

impl TuneManager {
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
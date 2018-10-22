extern crate serde_yaml;
extern crate byteorder;
extern crate bv;
use self::bv::BitVec;

use std::fs;
use std::io;
use std::path::{PathBuf};
use std::rc::Rc;
use std::collections::HashMap;
use std::convert;
use std::marker;

use self::byteorder::{ByteOrder, BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

use super::{Error, Result, Rom, RomManager};

use definition::{DataType, Endianness};
use definition::Table as TableDefinition;

use numvariant::NumVariant;




pub struct TableMeta {
	name: String,
	description: String,
}

trait TableDataTrait {
	fn size(&self) -> usize;
	fn get(&self, i: usize) -> NumVariant;
	fn set(&mut self, i: usize, data: NumVariant);
	fn serialize(&self, endianness: Endianness) -> Result<Vec<u8>>;
}

struct TableData<T> {
	data: Vec<T>,
}

trait TableType
where Self: Sized {
	fn deserialize<R: ReadBytesExt, O: ByteOrder>(data: &mut R) -> io::Result<Self>;
	fn serialize<W: WriteBytesExt, O: ByteOrder>(&self, data: &mut W) -> io::Result<()>;
}

impl TableType for u8 {
	fn deserialize<R: ReadBytesExt, O: ByteOrder>(data: &mut R) -> io::Result<Self> {
		data.read_u8()
	}

	fn serialize<W: WriteBytesExt, O: ByteOrder>(&self, data: &mut W) -> io::Result<()> {
		data.write_u8(*self)
	}
}

impl TableType for u16 {
	fn deserialize<R: ReadBytesExt, O: ByteOrder>(data: &mut R) -> io::Result<Self> {
		data.read_u16::<O>()
	}

	fn serialize<W: WriteBytesExt, O: ByteOrder>(&self, data: &mut W) -> io::Result<()> {
		data.write_u16::<O>(*self)
	}
}

impl TableType for u32 {
	fn deserialize<R: ReadBytesExt, O: ByteOrder>(data: &mut R) -> io::Result<Self> {
		data.read_u32::<O>()
	}

	fn serialize<W: WriteBytesExt, O: ByteOrder>(&self, data: &mut W) -> io::Result<()> {
		data.write_u32::<O>(*self)
	}
}

impl TableType for u64 {
	fn deserialize<R: ReadBytesExt, O: ByteOrder>(data: &mut R) -> io::Result<Self> {
		data.read_u64::<O>()
	}

	fn serialize<W: WriteBytesExt, O: ByteOrder>(&self, data: &mut W) -> io::Result<()> {
		data.write_u64::<O>(*self)
	}
}

impl TableType for i8 {
	fn deserialize<R: ReadBytesExt, O: ByteOrder>(data: &mut R) -> io::Result<Self> {
		data.read_i8()
	}

	fn serialize<W: WriteBytesExt, O: ByteOrder>(&self, data: &mut W) -> io::Result<()> {
		data.write_i8(*self)
	}
}

impl TableType for i16 {
	fn deserialize<R: ReadBytesExt, O: ByteOrder>(data: &mut R) -> io::Result<Self> {
		data.read_i16::<O>()
	}

	fn serialize<W: WriteBytesExt, O: ByteOrder>(&self, data: &mut W) -> io::Result<()> {
		data.write_i16::<O>(*self)
	}
}

impl TableType for i32 {
	fn deserialize<R: ReadBytesExt, O: ByteOrder>(data: &mut R) -> io::Result<Self> {
		data.read_i32::<O>()
	}

	fn serialize<W: WriteBytesExt, O: ByteOrder>(&self, data: &mut W) -> io::Result<()> {
		data.write_i32::<O>(*self)
	}
}

impl TableType for i64 {
	fn deserialize<R: ReadBytesExt, O: ByteOrder>(data: &mut R) -> io::Result<Self> {
		data.read_i64::<O>()
	}

	fn serialize<W: WriteBytesExt, O: ByteOrder>(&self, data: &mut W) -> io::Result<()> {
		data.write_i64::<O>(*self)
	}
}

impl TableType for f32 {
	fn deserialize<R: ReadBytesExt, O: ByteOrder>(data: &mut R) -> io::Result<Self> {
		data.read_f32::<O>()
	}

	fn serialize<W: WriteBytesExt, O: ByteOrder>(&self, data: &mut W) -> io::Result<()> {
		data.write_f32::<O>(*self)
	}
}

impl TableType for f64 {
	fn deserialize<R: ReadBytesExt, O: ByteOrder>(data: &mut R) -> io::Result<Self> {
		data.read_f64::<O>()
	}

	fn serialize<W: WriteBytesExt, O: ByteOrder>(&self, data: &mut W) -> io::Result<()> {
		data.write_f64::<O>(*self)
	}
}


impl<T> TableData<T> 
where T: TableType {
	/// Deserializes a table with the specified size in T units
	fn deserialize<O: ByteOrder>(data: &[u8], size: usize) -> Result<TableData<T>> {
		let mut reader = data;
		let mut deserialized = Vec::new();
		for _ in 0..size {
			deserialized.push(T::deserialize::<_,O>(&mut reader)?);
		}
		Ok(TableData {
			data: deserialized,
		})
	}

	fn serialize_order<O: ByteOrder>(&self) -> Result<Vec<u8>> {
		let mut serialized = Vec::new();
		for data in self.data.iter() {
			data.serialize::<_,O>(&mut serialized)?;
		}
		Ok(serialized)
	}
}

impl<T> TableDataTrait for TableData<T> where
T: convert::From<NumVariant> + marker::Copy + TableType, NumVariant: convert::From<T> {
	fn size(&self) -> usize {
		self.data.len()
	}

	fn get(&self, i: usize) -> NumVariant {
		NumVariant::from(self.data[i])
	}

	fn set(&mut self, i: usize, data: NumVariant) {
		self.data[i] = data.into();
	}

	fn serialize(&self, endianness: Endianness) -> Result<Vec<u8>> {
		match endianness {
			Endianness::Big => self.serialize_order::<BigEndian>(),
			Endianness::Little => self.serialize_order::<LittleEndian>(),
		}
	}
}

fn deserialize_table<O: ByteOrder>(datatype: DataType, data: &[u8], size: usize) -> Result<Box<TableDataTrait>> {
	match datatype {
		DataType::Uint8 => Ok(Box::new(TableData::<u8>::deserialize::<O>(data, size)?)),
		DataType::Uint16 => Ok(Box::new(TableData::<u16>::deserialize::<O>(data, size)?)),
		DataType::Uint32 => Ok(Box::new(TableData::<u32>::deserialize::<O>(data, size)?)),
		DataType::Uint64 => Ok(Box::new(TableData::<u64>::deserialize::<O>(data, size)?)),
		DataType::Int8 => Ok(Box::new(TableData::<i8>::deserialize::<O>(data, size)?)),
		DataType::Int16 => Ok(Box::new(TableData::<i16>::deserialize::<O>(data, size)?)),
		DataType::Int32 => Ok(Box::new(TableData::<i32>::deserialize::<O>(data, size)?)),
		DataType::Int64 => Ok(Box::new(TableData::<i64>::deserialize::<O>(data, size)?)),
		DataType::Float32 => Ok(Box::new(TableData::<f32>::deserialize::<O>(data, size)?)),
		DataType::Float64 => Ok(Box::new(TableData::<f64>::deserialize::<O>(data, size)?)),
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SerializedTable {
	id: usize,
	data: Vec<u8>,
}

pub struct Table {
	pub data_type: DataType,
	pub dirty: bool,
	modified: BitVec,
	data: Box<TableDataTrait>,
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

	pub fn save_raw(&self, endianness: Endianness) -> Result<Vec<u8>> {
		self.data.serialize(endianness)
	}

	pub fn height(&self) -> usize {
		self.height
	}

	pub fn width(&self) -> usize {
		self.data.size() / self.height
	}

	/// Returns true if the data at (`x`, `y`) has been modified
	pub fn modified(&self, x: usize, y: usize) -> bool {
		self.modified[(y * self.height + x) as u64]
	}

	/// Returns true if the table has been modified from the original ROM
	pub fn dirty(&self) -> bool {
		self.dirty
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
		self.data.size() == 1
	}

	pub fn set(&mut self, x: usize, y: usize, data: NumVariant) {
		// TODO: Convert data to the correct type
		self.data.set(y * self.height + x, data);
	}

	/// Expects the data to be in range. If not, it will panic.
	pub fn get(&self, x: usize, y: usize) -> NumVariant {
		self.data.get(y * self.height + x)
	}
}


#[derive(Debug, Deserialize, Serialize, Clone)]
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
	pub meta: TuneMeta,
}

impl Tune {
	/// Loads a tune from file
	pub fn load(meta: &TuneMeta, roms: &RomManager) -> Result<Tune> {
		// Search for the ROM
		let rom_meta = roms.search(&meta.rom_id).ok_or(Error::InvalidRomId)?;
		let rom = roms.load_rom(rom_meta)?;

		let mut tables = HashMap::new();
		// Load tables from file
		let contents = fs::read_to_string(&meta.data_path)?;
		let table_array: Vec<SerializedTable> = serde_yaml::from_str(&contents)?;
		for table in table_array {
			// Locate table definition
			if let Some(table_def) = rom_meta.main.find_table(table.id) {
				tables.insert(table.id, Table::load_raw(table_def, &table.data, rom_meta.main.endianness)?);
			} else {
				return Err(Error::InvalidTableId);
			}
		}

		Ok(Tune {
			rom: rom.clone(),
			tables,
			meta: meta.clone(),
		})
	}

	/// Saves the tune to file. The filepath is Tune::meta::data_path
	pub fn save(&self) -> Result<()> {
		let mut tables = Vec::new();
		for table in self.tables.iter() {
			if table.1.dirty() {
				// We only save modified tables
				tables.push(SerializedTable {
					id: *table.0,
					data: table.1.save_raw(self.rom.meta.main.endianness)?,
				});
			}
		}
		// Write to file
		fs::write(&self.meta.data_path, serde_yaml::to_string(&tables).unwrap())?;
		Ok(())
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
		// This is an ugly pattern that can't be fixed without NLL
		if self.tables.contains_key(&id) {
			return self.tables.get(&id).ok_or(Error::InvalidTableId); // This should never error
		}
		self.load_table(id)
	}
}

#[derive(Debug)]
pub struct TuneManager {
	tunes: Vec<TuneMeta>,
	base: PathBuf,
}

impl TuneManager {
	pub fn load(base: PathBuf) -> Result<TuneManager> {
		let path = base.join("tunes.yaml");
		if !path.is_file() {
			return Ok(TuneManager {
				tunes: Vec::new(),
				base,
			});
		}

		// Load metadata
		Ok(TuneManager {
			tunes: serde_yaml::from_str(&fs::read_to_string(&path)?)?,
			base,
		})
	}

	pub fn save(&self) -> Result<()> {
		fs::write(&self.base.join("tunes.yaml"), serde_yaml::to_string(&self.tunes).unwrap())?;
		Ok(())
	}

	/// Adds a new tune to the database. Note: this WILL NOT save, you must call `save()`
	/// The data_path may not be preserved; it will be loaded as "$TUNE_PATH/id"
	fn add(&mut self, tune: &Tune) {
		self.tunes.push(tune.meta.clone());
	}

	fn add_meta(&mut self, name: String, id: String, rom_id: String) {
		self.tunes.push(TuneMeta {
			name,
			id,
			rom_id,
			data_path: PathBuf::default(),
		})
	}

	/// Creates a new tune from a ROM and adds it to the database.
	/// Note: this WILL NOT save, you must call `save()`
	pub fn new(&mut self, name: String, id: String, rom: &Rc<Rom>) -> Tune {
		let tune = Tune {
			rom: rom.clone(),
			tables: HashMap::new(),
			meta: TuneMeta {
				data_path: self.base.join(&id),
				name,
				id,
				rom_id: rom.meta.id.clone(),
			}
		};
		self.add(&tune);
		tune
	}
}
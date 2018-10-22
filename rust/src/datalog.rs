use numvariant::NumVariant;
use definition::Pid;
use std::sync::atomic::{AtomicBool, Ordering};
use protocols::uds::UdsInterface;
use std::thread;
use std::time;
use std::cell::RefCell;

use error::Result;

extern crate eval;

pub struct Entry {
	pub data: RefCell<Vec<NumVariant>>,
	pub pid: Pid,
	pub expr: eval::Expr,
}

pub struct Log {
	pub entries: Vec<Entry>,

	callbacks: Vec<Box<RefCell<FnMut(&Entry, NumVariant)>>>,
}

impl Default for Log {
	fn default() -> Log {
		Log {
			entries: Vec::new(),
			callbacks: Vec::new(),
		}
	}
}


impl Log {
	/// Adds a new entry and returns the index
	pub fn add_entry(&mut self, pid: Pid) -> Result<usize> {
		let index = self.entries.len();
		let expr = eval::Expr::new(pid.formula.clone()).compile()?;
		self.entries.push(Entry {
			data: RefCell::new(Vec::new()),
			expr,
			pid,
		});
		Ok(index)
	}

	/// Adds data to an entry
	pub fn add_data(&self, entry: &Entry, data: NumVariant) {
		entry.data.borrow_mut().push(data);
		for cb in &self.callbacks {
			let mut closure = cb.borrow_mut();
			(&mut *closure)(entry, data);
		}
	}

	pub fn register<F: FnMut(&Entry, NumVariant) + 'static>(&mut self, callback: F) {
		self.callbacks.push(Box::new(RefCell::new(callback)));
	}
}

pub trait Logger {
	fn run(&mut self) -> Result<()>;

	fn stop(&mut self);
}

pub struct UdsLogger<'a> {
	log: &'a mut Log,
	interface: &'a UdsInterface,
	running: AtomicBool,
}

impl<'a> UdsLogger<'a> {
	pub fn new(log: &'a mut Log, interface: &'a UdsInterface) -> UdsLogger<'a> {
		UdsLogger {
			log,
			interface,
			running: AtomicBool::new(false),
		}
	}
}

impl<'a> Logger for UdsLogger<'a> {
	fn run(&mut self) -> Result<()> {
		assert_eq!(self.running.load(Ordering::SeqCst), false);
		*self.running.get_mut() = true;

		// Start logging
		while self.running.load(Ordering::Relaxed) {
			let log = &mut self.log;
			for entry in &log.entries {
				// Send UDS request
				let response = self.interface.request(((entry.pid.code & 0xFF00) >> 8) as u8, &[(entry.pid.code & 0xFF) as u8])?;

				let mut context = eval::Context::new();
				if response.len() >= 3 {
					context.insert("C".to_string(), eval::to_value(response[2]));
				}
				if response.len() >= 2 {
					context.insert("B".to_string(), eval::to_value(response[1]));
				}
				if response.len() >= 1 {
					context.insert("A".to_string(), eval::to_value(response[0]));
				}

				// Evaluate the expression
				let val = eval::ExecOptions::new(&entry.expr).contexts(&[context]).exec()?;
				let num = (|| {
					if val.is_u64() {
						return NumVariant::U64(val.as_u64().unwrap());
					}
					if val.is_i64() {
						return NumVariant::I64(val.as_i64().unwrap());
					}
					if val.is_f64() {
						return NumVariant::F64(val.as_f64().unwrap());
					}
					// This shouldn't ever happen
					return NumVariant::I64(0);
				})();

				// Add the data to the log
				log.add_data(entry, num);
			}
			thread::sleep(time::Duration::from_millis(100));
		}

		*self.running.get_mut() = false;

		Ok(())
	}

	fn stop(&mut self) {
		*self.running.get_mut() = false;
	}
}
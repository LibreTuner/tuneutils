use std::convert::From;

#[derive(Clone, Copy, Debug)]
pub enum NumVariant {
	I8(i8),
	I16(i16),
	I32(i32),
	I64(i64),
	U8(u8),
	U16(u16),
	U32(u32),
	U64(u64),
	F32(f32),
	F64(f64),
}

impl From<i8> for NumVariant {
	fn from(num: i8) -> Self {
		NumVariant::I8(num)
	}
}

impl From<i16> for NumVariant {
	fn from(num: i16) -> Self {
		NumVariant::I16(num)
	}
}

impl From<i32> for NumVariant {
	fn from(num: i32) -> Self {
		NumVariant::I32(num)
	}
}

impl From<i64> for NumVariant {
	fn from(num: i64) -> Self {
		NumVariant::I64(num)
	}
}

impl From<u8> for NumVariant {
	fn from(num: u8) -> Self {
		NumVariant::U8(num)
	}
}

impl From<u16> for NumVariant {
	fn from(num: u16) -> Self {
		NumVariant::U16(num)
	}
}

impl From<u32> for NumVariant {
	fn from(num: u32) -> Self {
		NumVariant::U32(num)
	}
}

impl From<u64> for NumVariant {
	fn from(num: u64) -> Self {
		NumVariant::U64(num)
	}
}

impl From<f32> for NumVariant {
	fn from(num: f32) -> Self {
		NumVariant::F32(num)
	}
}

impl From<f64> for NumVariant {
	fn from(num: f64) -> Self {
		NumVariant::F64(num)
	}
}

macro_rules! impl_variant {
	( $($x:ty), *) => {
		$(
			impl From<NumVariant> for $x {
				fn from(var: NumVariant) -> Self {
					match var {
						NumVariant::I8(num) => num as Self,
						NumVariant::I16(num) => num as Self,
						NumVariant::I32(num) => num as Self,
						NumVariant::I64(num) => num as Self,
						NumVariant::U8(num) => num as Self,
						NumVariant::U16(num) => num as Self,
						NumVariant::U32(num) => num as Self,
						NumVariant::U64(num) => num as Self,
						NumVariant::F32(num) => num as Self,
						NumVariant::F64(num) => num as Self,
					}
				}
			}
		)*
	}
}

impl_variant!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64);
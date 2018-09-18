#[cfg(windows)]
extern crate j2534;

pub mod can;
#[cfg(linux)]
pub mod socketcan;

#[cfg(windows)]
pub mod j2534can;
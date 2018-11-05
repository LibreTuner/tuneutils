#[macro_use]
extern crate serde_derive;
extern crate serde;

#[cfg(feature = "j2534")]
extern crate j2534;

pub mod error;
pub mod numvariant;
pub mod protocols;
pub mod download;
pub mod flash;
pub mod link;
pub mod authenticator;
pub mod definition;
pub mod rom;
pub mod datalog;

pub use self::error::{Error, Result};
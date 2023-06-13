#![doc = include_str!("../README.md")]
#![recursion_limit = "1024"]

mod compression;
pub mod config;
pub(crate) mod crypt;
pub mod db;
pub mod error;
pub(crate) mod format;
pub(crate) mod hmac_block_stream;
mod io;
mod key;
pub(crate) mod variant_dictionary;
pub(crate) mod xml_db;

pub use self::{
    config::DatabaseConfig,
    db::Database,
    error::{Error, Result},
    key::DatabaseKey,
};
pub use uuid::Uuid;

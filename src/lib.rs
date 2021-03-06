#[macro_use]
extern crate anyhow;

pub mod cfg;
#[macro_use]
pub mod cli;
pub mod env_file;
pub mod run_file;
pub mod template;
pub mod utils;

pub const BIN_NAME: &'static str = "sht";

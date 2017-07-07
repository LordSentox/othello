use std::fs::File;
use std::io::prelude::*;
use std::io;
use toml;

pub enum ReadError {
	IO(io::Error),
	TOML(toml::de::Error)
}

// Holds the server configuration.
#[derive(Deserialize)]
pub struct Config {
	port: u16,
	max_clients: usize
}

impl Config {
	// Load the configuration from the toml file dedicated to the server.
	pub fn load() -> Result<Config, ReadError> {
		let mut file = match File::open("server.toml") {
			Ok(file) => file,
			Err(err) => return Err(ReadError::IO(err))
		};

		let mut contents = String::new();
		match file.read_to_string(&mut contents) {
			Ok(_) => {},
			Err(err) => return Err(ReadError::IO(err))
		}

		match toml::from_str(&contents) {
			Ok(conf) => Ok(conf),
			Err(err) => Err(ReadError::TOML(err))
		}
	}
}

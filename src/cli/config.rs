use std::fs::File;
use std::io::prelude::*;
use std::io;
use toml;

lazy_static! {
	pub static ref CONFIG: Config = {
		match Config::load() {
			Ok(conf) => conf,
			Err(err) => panic!("Could not read client.toml: {:?}", err)
		}
	};
}

#[derive(Debug)]
pub enum ReadError {
	IO(io::Error),
	TOML(toml::de::Error)
}

#[derive(Deserialize)]
pub struct Network {
	pub server_ip: String,
	pub server_port: u16,
	pub login_name: Option<String>
}

#[derive(Deserialize)]
pub struct Graphics {
	pub board: String,
	pub square_size: Option<u16>,
	pub white_piece: String,
	pub black_piece: String,
	pub shadow: Option<String>,
	pub white_score_colour: Vec<u8>,
	pub black_score_colour: Vec<u8>
}

// Holds the client configuration.
#[derive(Deserialize)]
pub struct Config {
	pub network: Network,
	pub graphics: Graphics
}

impl Config {
	// Load the configuration from the toml file dedicated to the server.
	pub fn load() -> Result<Config, ReadError> {
		let mut file = match File::open("client.toml") {
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

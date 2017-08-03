use std::fs::File;
use std::io::prelude::*;
use std::io;
use toml;

lazy_static! {
	static ref CONFIG: Config = {
		match Config::load() {
			Ok(conf) => conf,
			Err(err) => panic!("Could not find configuration file 'client.toml'")
		}
	};
}

pub enum ReadError {
	IO(io::Error),
	TOML(toml::de::Error)
}

#[derive(Deserialize)]
pub struct Network {
	server_ip: String,
	server_port: u16,
	login_name: String
}

#[derive(Deserialize)]
pub struct Graphics {
	board: String,
	square_size: Option<u16>,
	stone: String
}

// Holds the client configuration.
#[derive(Deserialize)]
pub struct Config {
	network: Network,
	graphics: Graphics
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

use bincode::{serialize, deserialize, Bounded};
use serde::Serialize;
use serde::de::DeserializeOwned;

use std::marker::Sized;
use std::net::TcpStream;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, PartialEq)]
pub struct PChangeNameRequest {
	pub name: String
}

impl Packet for PChangeNameRequest {
	fn bin_size() -> u64 { 32 }
	fn id() -> u8 { 0 }
}

// The Packet trait implements the base functionality of all packets.
pub trait Packet: Serialize + DeserializeOwned {
	fn bin_size() -> u64;
	fn id() -> u8;

	fn write_to_stream(&self, tcp_stream: &mut TcpStream) -> bool
	where Self: Sized {
		let size = Bounded(Self::bin_size() + 1);

		let data: Vec<u8> = match serialize(&self, size) {
			Ok(data) => data,
			Err(err) => return false
		};

		// Write the id to the stream
		match tcp_stream.write(&[Self::id(); 1]) {
			Ok(len) => if len != 1 { return false; },
			Err(err) => return false
		};

		// TODO: What if the second stage fails? The next packets could become
		// corrupted. Needs TESTS!

		match tcp_stream.write(&data) {
			Ok(len) => println!("Sent packet over network, size {}", len),
			Err(err) => return false
		}

		true
	}

	fn read_from_stream(&mut self, tcp_stream: &mut TcpStream) {
		let mut data: Vec<u8> = vec![0; Self::bin_size() as usize];

		match tcp_stream.read(&mut data) {
			Ok(len) => assert!(len > 0),
			Err(err) => panic!("Error receiving packet: {}", err)
		};

		*self = deserialize(&data).unwrap();
	}
}

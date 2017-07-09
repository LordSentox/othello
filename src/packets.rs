use bincode::{serialize, deserialize, Bounded};
use serde::Serialize;
use serde::de::DeserializeOwned;

use std::marker::Sized;
use std::net::TcpStream;
use std::io::{Read, Write};

// Build an empty Packet for the given id.
// If it returns an Err, the Packet id is invalid.
pub fn build(id: u8) -> Result<Box<DownPacket>, ()> {
	let p: &Packet = match id {
		PChangeNameRequest::ID => &PChangeNameRequest { name: "unknown".to_string() },
		PClientList::ID => &PClientList { clients: Vec::new() },
		PRequestGame::ID => &PRequestGame { receiver: String::new() },
		_ => return Err(())
	};

	Ok(p)
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct PChangeNameRequest {
	pub name: String
}

impl Packet for PChangeNameRequest {
	fn bin_size() -> u64 { 32 }
	fn id() -> u8 { 0 }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct PClientList {
	pub clients: Vec<String>
}

impl Packet for PClientList {
	fn bin_size() -> u64 { 1024 }
	fn id() -> u8 { 1 }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct PRequestGame {
	pub receiver: String
}

impl Packet for PRequestGame {
	fn bin_size() -> u64 { 64 }
	fn id() -> u8 { 2 }
}

// The Packet trait implements the base functionality of all packets.
pub trait Packet {
	fn bin_size() -> u64;
	fn id() -> u8;
}

pub trait UpPacket: Packet + Serialize {
	fn write_to_stream(&self, tcp_stream: &mut TcpStream) -> bool
	where Self: Sized {
		let size = Bounded(Self::bin_size() + 1);

		let data: Vec<u8> = match serialize(&self, size) {
			Ok(data) => data,
			Err(err) => { println!("{}", err); return false; }
		};

		// Write the id to the stream
		match tcp_stream.write(&[Self::ID; 1]) {
			Ok(len) => if len != 1 { return false; },
			Err(err) => { println!("{}", err); return false; }
		};

		// TODO: What if the second stage fails? The next packets could become
		// corrupted. Needs TESTS!

		match tcp_stream.write(&data) {
			Ok(len) => println!("Sent packet over network, size {}", len),
			Err(err) => { println!("{}", err); return false; }
		}

		true
	}
}

pub trait DownPacket: Packet + DeserializeOwned {
	fn read_from_stream(&mut self, tcp_stream: &mut TcpStream) {
		let mut data: Vec<u8> = vec![0; Self::bin_size() as usize];

		match tcp_stream.read(&mut data) {
			Ok(len) => assert!(len > 0),
			Err(err) => panic!("Error receiving packet: {}", err)
		};

		*self = deserialize(&data).unwrap();
	}
}

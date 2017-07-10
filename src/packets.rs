use bincode::{serialize, deserialize, Bounded};
use serde::Serialize;
use serde::de::DeserializeOwned;

use std::marker::Sized;
use std::net::TcpStream;
use std::io::{Read, Write};

pub const MAX_PACKET_SIZE: u64 = 1024;

#[derive(Serialize, Deserialize, PartialEq)]
pub enum Packet {
	/// Request to change the name of the sender internally to "String" (Client->Server only)
	ChangeNameRequest(String),
	/// Request the updated client list manually (Client->Server only)
	RequestClientList,
	/// The complete list of all client ids and names in "Vec<(u64, String)>" (Server->Client only)
	ClientList(Vec<(u64, String)>),
	/// Request a game. On the server, "String" is the name of the requestee, on the client the name
	/// of the one who has requested.
	RequestGame(String)
}

pub fn write_to_stream(p: &Packet, stream: &mut TcpStream) -> bool {
	let size = Bounded(MAX_PACKET_SIZE);

	let data: Vec<u8> = match serialize(&p, size) {
		Ok(data) => data,
		Err(err) => { println!("Error serialising packet: {}", err); return false; }
	};

	match stream.write(&data) {
		Ok(len) => {
			assert!(len <= MAX_PACKET_SIZE as usize);
			true
		},
		Err(err) => {
			println!("Failed writing packet to stream: {}", err);
			false
		}
	}
}

// TODO: Later, the buffer could be borrowed, which would increase performance.
// This has to be crosschecked with the inner workings of bytecode, however.
/// Read a packet from the stream. This returns a packet, in case one could be read
/// in conjunction with a bool stating false, in case the stream has been closed.
pub fn read_from_stream(stream: &mut TcpStream) -> (Option<Packet>, bool) {
	let mut data: Vec<u8> = vec![0; MAX_PACKET_SIZE as usize];

	match stream.read(&mut data) {
		Ok(len) => {
			if len == 0 {
				(None, false)
			}
			else {
				(Some(deserialize(&data).unwrap()), true)
			}
		}
		Err (err) => {
			println!("Error receiving packet: {}", err);
			(None, true)
		}
	}
}

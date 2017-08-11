use bincode::{serialize, deserialize, Bounded, Error};

use std::net::TcpStream;
use std::io::{Read, Write};
use std::io;

use board::Piece;

pub type ClientId = u64;
pub use std::u64::MAX as ClientIdMAX;

pub const MAX_PACKET_SIZE: u64 = 512;

#[derive(Debug)]
pub enum PacketReadError {
	/// The packet could not be properly deserialised.
	DeserializeError(Error),
	/// The packet could not be read properly from the stream.
	IOError(io::Error),
	/// The connection has been closed by the peer socket.
	Closed
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Packet {
	/// Packet to let the client know which id it will have. This is the first packet sent to the
	/// client by the server when the connection has been established.
	ConnectSuccess(ClientId),
	/// This packet is never actually sent over the network, but it is used internally to handle
	/// disconnects a little more gracefully and easily.
	Disconnect,
	/// Login into the server. The server will then answer with a LoginResponse.
	Login(String),
	/// Positive login response to a client. The argument is the client_id the
	/// server will use to identify the client.
	LoginAccept,
	/// Negative login response to a client. The argument is the reason of the denial.
	LoginDeny(String),
	/// Request the updated client list manually (Client->Server only)
	RequestClientList,
	/// The complete list of all client ids and names in "Vec<(u64, String)>" (Server->Client only)
	ClientList(Vec<(ClientId, String)>),
	/// Request a game. On the server, ClientId is the id of the requestee, on the client the id
	/// of the one who has requested.
	/// In direction Client->Server it is also the package used to accept a request, simply by
	/// making a request to the client that had requested the game in the first place.
	RequestGame(ClientId),
	/// Deny a game response. If there was no request registered in the first place, this will
	/// probably do nothing at all.
	/// In direction Server->Client the id is of the one who has denied.
	/// In direction Client->Server it is the id of the one to deny the game of.
	DenyGame(ClientId),
	/// Start a game with a fresh board. This is Server->Client only and the colour the client will
	/// be controlling is sent, as well as the id of the opponent.
	StartGame(ClientId, Piece),
	/// Place a piece onto the board of the client with the ClientId, which is always the id of the
	/// opponent. The server will never respond to this packet, since the client is supposed to
	/// check the rules themselves aswell.
	PlacePiece(ClientId, u8, u8),
	/// Message to or from another client. If it is in direction Server->Client, the ID of the client
	/// that has sent the message is the id, in direction Client->Server it's the id of the client
	/// it is directed at.
	Message(ClientId, String)
}

impl Packet {
	pub fn write_to_stream(&self, stream: &mut TcpStream) -> bool {
		let size = Bounded(MAX_PACKET_SIZE);

		let data: Vec<u8> = match serialize(&self, size) {
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
	pub fn read_from_stream(stream: &mut TcpStream) -> Result<Packet, PacketReadError> {
		let mut data: Vec<u8> = vec![0; MAX_PACKET_SIZE as usize];

		match stream.read(&mut data) {
			Ok(len) => {
				if len == 0 {
					Err(PacketReadError::Closed)
				}
				else {
					match deserialize(&data) {
						Ok(p) => Ok(p),
						Err(err) => {
							Err(PacketReadError::DeserializeError(err))
						}
					}
				}
			}
			// XXX: There might be some cleanup to do in case of the following
			// error, which still needs to be tested.
			Err (err) => Err(PacketReadError::IOError(err))
		}
	}
}

use std::net::{Shutdown, TcpStream};
use std::sync::Mutex;
use std::io;
use packets::*;
use std::time::Duration;

pub enum DirSocket {
	Read,
	Write,
	Both
}

/// A safe wrapper around a stream, which allows exactly one thread to read
/// and one thread to write to the stream at a given time.
/// The Remote however cannot be cloned, since that would undermine the
/// safety we are trying to establish in the first place.
pub struct Remote {
	read: Mutex<TcpStream>,
	write: Mutex<TcpStream>
}

impl Remote {
	/// Wrap a TcpStream into a remote object.
	/// This fails if the stream is not cloneable.
	pub fn new(stream: TcpStream) -> Result<Remote, io::Error> {
		// Try to clone the stream, because Rust would not allow us to have
		// two mutable streams otherwise, which is perfectly safe when one
		// is used for reading and one for writing.
		let stream_clone = match stream.try_clone() {
			Ok(stream) => stream,
			Err(err) => return Err(err)
		};

		// At this point, the assignment of the streams is completely arbitrary,
		// since both streams could do the same work.
		Ok(Remote {
			read: Mutex::new(stream),
			write: Mutex::new(stream_clone)
		})
	}

	/// Set the timeout of the read-portion or the send-portion of the socket.
	/// If set to None, the part in question will block indefinately.
	pub fn set_timeout(&self, timeout: Option<Duration>, dir: DirSocket) -> io::Result<()> {
		match dir {
			DirSocket::Read => self.read.lock().unwrap().set_read_timeout(timeout),
			DirSocket::Write => self.write.lock().unwrap().set_write_timeout(timeout),
			DirSocket::Both => {
				if let Err(err) = self.read.lock().unwrap().set_read_timeout(timeout) {
					return Err(err);
				}

				self.write.lock().unwrap().set_write_timeout(timeout)
			}
		}
	}

	/// Try to read the next incoming packet. This blocks until the stream is closed, the packet
	/// has been read (With or without error) or the timeout is triggered.
	/// Returns the packet if available and false, in case the stream has been
	/// closed.
	pub fn read_packet(&self) -> Result<Packet, PacketReadError> {
		let mut read_lock = self.read.lock().unwrap();

		Packet::read_from_stream(&mut read_lock)
	}

	/// Write the packet to the stream. Returns true if successful,
	/// false if an error occured.
	pub fn write_packet(&self, p: &Packet) -> bool {
		let mut write_lock = self.write.lock().unwrap();

		p.write_to_stream(&mut write_lock)
	}

	/// Shuts down the connection. After this it will be impossible to send
	/// or read anything from the stream.
	pub fn shutdown(&self) {
		// Reading and sending will be seperately shut down, as to not
		// disturb any operation that might still be in the process.
		self.read.lock().unwrap().shutdown(Shutdown::Read).expect("Error while shutting down read thread.");
		self.write.lock().unwrap().shutdown(Shutdown::Write).expect("Error while shutting down write thread.");
	}
}

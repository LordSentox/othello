use std::sync::{Arc, Weak, Mutex, RwLock};
use std::sync::mpsc::{self, Sender, Receiver};
use remote::Remote;
use packets::*;
use std::thread::{self, JoinHandle};
use super::nethandler::*;
use std::collections::{HashMap, VecDeque};

/// Connection to a client on the Network.
pub struct NetClient {
    id: ClientId,
    remote: Arc<Remote>,
	packets: ArcRw<Vec<Weak<Mutex<VecDeque<Packet>>>>>, // Receiver of all the packets of this client.
    pt_handle: Option<JoinHandle<()>> // JoinHandle of the packet thread.
}

impl NetClient {
    /// Create a new client from the remote provided. This will start listening for packets
    /// id is the clients id on the NetHandler.
    /// Remote is the socket the client will receive packets from and send packets to.
    /// global_bus is the bus, where all packets of all clients will be sent to.
    pub (super) fn start(nethandler: Arc<NetHandler>, id: ClientId, remote: Remote) -> NetClient {
        let remote = Arc::new(remote);
		let packets = Arc::new(RwLock::new(Vec::new()));

        // Start the packet receiving thread.
        let remote_clone = remote.clone();
		let packets_clone: ArcRw<Vec<Weak<Mutex<VecDeque<Packet>>>>> = packets.clone();
        let pt_handle = thread::spawn(move || {
            loop {
				// Read the packet from the remote of this client.
                let packet = match remote_clone.read_packet() {
                    Ok(p) => p,
                    Err(PacketReadError::Closed) => {
                        // Create a disconnection packet. Then the other parts can decide how this
                        // will be handled.
                        Packet::Disconnect
                    },
                    Err(err) => {
                        // An error occured. Ignore this packet.
                        println!("Error reading packet from client [{}]. {:?}", id, err);
                        continue;
                    }
                };

				// Send the packet to all handlers that are subscribed specifically to this client.
				for s in &*packets_clone.read().unwrap() {
					if let Some(s) = s.upgrade() {
						s.lock().unwrap().push_back(packet.clone());
					}
				}

				// Check if any of the receivers for the clients packets have hung up.
				// TODO: Same as the push_packet function in the NetHandler.
				if let Ok(mut packets) = packets_clone.try_write() {
					packets.retain(|ref s| {s.upgrade().is_some()});
				}

				// Send the packet to the global packets VecDeque, so it can be handled by everyone
				// that is globally subscribed.
				nethandler.push_packet(id, packet.clone());

                // If the Disconnection packet has been created, there will no longer be anything
                // to do, so the client will be stopped.
                if let Packet::Disconnect = packet {
                    break;
                }
            }
        });

        NetClient {
            id: id,
            remote: remote,
			packets: packets,
            pt_handle: Some(pt_handle)
        }
    }

    /// Disconnect the client forcefully. If a message is provided, it will be sent to the client
    /// before the remote is shut down.
    /// Blocks until the client is completely dead.
    pub (super) fn disconnect(&mut self, msg: Option<&str>) {
		// TODO: At the moment it would be awkward to implement, since the remote class is not
		// that mature. Also the current server doesn't really need to disconnect anyone at the
		// moment, so It'll have to do for now.
		unimplemented!();

		// Now the remote can be killed and the packet receive thread will be
		// joined, thus shutting down the client.
		if self.pt_handle.is_none() {
			println!("[WARNING] Could not disconnect client. Client was already disconnected.");
			return;
		}

		// Send the message to the client if provided.
		if let Some(msg) = msg {
			let p = Packet::Message(SERVER_ID, msg.to_string());
			assert!(self.remote.write_packet(&p));
		}

		let handle = self.pt_handle.take().unwrap();
		match handle.join() {
			Ok(_) => println!("Client [{}] was disconnected successfully.", self.id),
			Err(err) => panic!("Warning: Client was shut down with error {:?}", err)
		}
    }

	/// Send a packet to the other end of this NetClient
	pub (super) fn send(&self, p: &Packet) -> bool {
		self.remote.write_packet(&p)
	}

	/// Subscribe to this client and receive all the packets from them. They are then saved into the packets-
	/// VecDeque provided.
	pub fn subscribe(&self, packets: Weak<Mutex<VecDeque<Packet>>>) {
		self.packets.write().unwrap().push(packets)
	}
}

use std::sync::{Arc, RwLock};
use std::sync::mpsc::{self, Sender, Receiver};
use remote::Remote;
use packets::*;
use std::thread::{self, JoinHandle};
use super::nethandler::*;
use std::collections::HashMap;

/// Connection to a client on the Network.
pub struct NetClient {
    id: ClientId,
    remote: Arc<Remote>,
	packets: Receiver<Packet>, // Receiver of all the packets of this client.
    pt_handle: Option<JoinHandle<()>> // JoinHandle of the packet thread.
}

impl NetClient {
    /// Create a new client from the remote provided. This will start listening for packets
    /// id is the clients id on the NetHandler.
    /// Remote is the socket the client will receive packets from and send packets to.
    /// global_bus is the bus, where all packets of all clients will be sent to.
    pub (super) fn from_remote(client_map: ArcRw<HashMap<ClientId, Arc<NetClient>>>, id: ClientId, remote: Remote, packet_tx: Sender<Packet>) -> NetClient {
        let remote = Arc::new(remote);

		// Create a personal channel for all packets that will be received by this client.
		let (sender, receiver) = mpsc::channel();

        // Start the packet receiving thread.
        let remote_clone = remote.clone();
        let pt_handle = thread::spawn(move || {
            loop {
				// Read the packet from the remote of this client.
                let packet = match remote_clone.read_packet() {
                    Ok(p) => p,
                    Err(PacketReadError::Closed) => {
                        // The client has disconnected. Remove it from the client map.
                        client_map.write().unwrap().remove(&id);

                        println!("Client [{}] disconnected.", id);
                        break; // End the receiving thread.
                    },
                    Err(err) => {
                        // An error occured. Ignore this packet.
                        println!("Error reading packet from client [{}]. {:?}", id, err);
                        continue;
                    }
                };

				// Send the packet to the local subscribers of this client, then to the global
                // subscribers.
				match packet_tx.send(packet.clone()) {
					Ok(_) => {},
					Err(_) => println!("[WARNING] Packet received on client thread that could not be distributed.")
				}

				sender.send(packet).unwrap();
            }
        });

        NetClient {
            id: id,
            remote: remote,
			packets: receiver,
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
}

unsafe impl Send for NetClient {}
unsafe impl Sync for NetClient {}

use bus::Bus;
use std::sync::{Arc, RwLock};
use remote::Remote;
use packets::*;
use std::thread::{self, JoinHandle};
use super::nethandler::*;

/// Connection to a client on the Network.
pub struct NetClient {
    id: ClientId,
    remote: ArcRw<Remote>,
    bus: ArcRw<Bus<Packet>>,
    global_bus: ArcRw<Bus<(ClientId, Packet)>>,
    pt_handle: Option<JoinHandle<()>> // JoinHandle of the packet thread.
}

impl NetClient {
    /// Create a new client from the remote provided. This will start listening for packets
    /// id is the clients id on the NetHandler.
    /// Remote is the socket the client will receive packets from and send packets to.
    /// global_bus is the bus, where all packets of all clients will be sent to.
    pub fn from_remote(id: ClientId, remote: Remote, global_bus: ArcRw<Bus<(ClientId, Packet)>>) -> NetClient {
        let bus = Arc::new(RwLock::new(Bus::new(10)));
        let remote = Arc::new(RwLock::new(remote));

        // Start the packet receiving thread.
        let bus_clone = bus.clone();
        let remote_clone = remote.clone();
        let global_bus_clone = global_bus.clone();
        let pt_handle = thread::spawn(move || {
            loop {
				// Read the packet from the remote of this client.
                let packet = match remote_clone.read_packet() {
                    Ok(p) => p,
                    Err(PacketReadError::Closed) => {
                        // The client has disconnected. Remove it from the client map.
                        clients_clone.write().unwrap().remove(&last_id);

                        println!("Client [{}] disconnected.", last_id);
                        break; // End the receiving thread.
                    },
                    Err(err) => {
                        // An error occured. Ignore this packet.
                        println!("Error reading packet from client [{}]. {:?}", last_id, err);
                        continue;
                    }
                };

				// Send the packet to the local subscribers of this client, then to the global
                // subscribers.
                bus_clone.read().unwrap().broadcast(packet.clone());
                global_bus_clone.read().unwrap().broadcast(packet);
            }
        });

        NetClient {
            id: id,
            remote: remote,
            bus: bus,
            global_bus: global_bus,
            pt_handle: Some(pt_handle)
        }
    }

    /// Disconnect the client forcefully. If a message is provided, it will be sent to the client
    /// before the remote is shut down.
    /// Blocks until the client is completely dead.
    pub fn disconnect(&self) {

    }
}

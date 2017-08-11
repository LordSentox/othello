use std::sync::{Arc, Weak, Mutex};
use super::{Game, Master, NetClient, NetHandler};
use packets::*;
use std::collections::{HashSet, VecDeque};

pub struct GameHandler {
    nethandler: Arc<NetHandler>,
    games: Vec<Weak<Game>>,
    /// All pending requests the first id is the requester, the second the requestee who has not
    /// yet answered.
    pending: HashSet<(ClientId, ClientId)>,
    packets: Arc<Mutex<VecDeque<(ClientId, Packet)>>>
}

impl GameHandler {
    pub fn new(nethandler: Arc<NetHandler>) -> GameHandler {
        // Subscribe to the NetHandler, then return the GameHandler with an empty games list, since
        // naturally nothing has been requested yet.
        let packets = Arc::new(Mutex::new(VecDeque::new()));
        nethandler.subscribe(Arc::downgrade(&packets));

        GameHandler {
            nethandler: nethandler,
            games: Vec::new(),
            pending: HashSet::new(),
            packets: packets
        }
    }

    pub fn handle_packets(&mut self) {
        loop {
            let (client, packet) = match self.packets.lock().unwrap().pop_front() {
                Some(cp) => cp,
                None => break
            };

            match packet {
                Packet::Disconnect => self.handle_disconnect(client),
                Packet::RequestGame(to) => self.handle_game_request(client, to),
                Packet::DenyGame(to) => self.handle_deny_game(client, to),
                _ => {}
            }
        }

		// Check for games that are no longer running, to prevent memory leakage in form of the
		// Games-Vector just groing with long dead games.
		self.games.retain(|ref game| { game.upgrade().is_some() });
    }

    fn handle_disconnect(&mut self, client: ClientId) {
        // All game requests to the client will be denied.
        for &(from, to) in &self.pending {
            if to == client {
                self.nethandler.send(from, &Packet::DenyGame(to));
            }
        }

        // Remove all game requests the client was involved in.
        self.pending.retain(|&(ref from, ref to)| { *from != client && *to != client });
    }

    fn handle_game_request(&mut self, from: ClientId, to: ClientId) {
        // In case the request has already been made, it can be ignored.
        if self.pending.contains(&(from, to)) {
            println!("Duplicate game request from [{}] to [{}] was ignored. Still awaiting answer.", from, to);
            return;
        }

        if self.pending.contains(&(to, from)) {
            self.pending.remove(&(to, from));

            println!("Starting game between [{}] and [{}]", from, to);

            // There has been no explicit response, but since both have requested a game from the
            // other client, we can assume that the game can be started.
            self.start_game(from, to);
        }

        self.pending.insert((from, to));
        println!("Added game request from [{}] to [{}]", from, to);
        self.nethandler.send(to, &Packet::RequestGame(from));
    }

    fn handle_deny_game(&mut self, from: ClientId, to: ClientId) {
		if !self.pending.remove(&(to, from)) {
			// There was no request, so there is nothing to deny.
			println!("[WARNING] Blocked DenyGame packet, since there has never been a request.");
			return;
		}

		// There was a request, so inform the one it has been denied from of it.
		self.nethandler.send(to, &Packet::DenyGame(from));
    }

    fn start_game(&mut self, client1: ClientId, client2: ClientId) {
        let client1 = match self.nethandler.get_client(client1) {
            Some(c) => c,
            None => return
        };

        let client2 = match self.nethandler.get_client(client2) {
            Some(c) => c,
            None => return
        };

        let game = match Game::new(client1, client2) {
            Some(g) => g,
            None => return
        };

        // The game has been started successfully. Add it to the games of this GameHandler.
        self.games.push(game);
    }
}

use std::sync::{Arc, Weak, Mutex};
use board::*;
use std::thread;
use std::collections::VecDeque;
use packets::*;

use super::NetClient;

pub struct Game {
    board: Mutex<Board>,
    black: Weak<NetClient>,
    black_packets: Arc<Mutex<VecDeque<Packet>>>,
    white: Weak<NetClient>,
    white_packets: Arc<Mutex<VecDeque<Packet>>>
}

impl Game {
    /// Create (and start) a new game between the two clients provided. This will spawn a new
    /// thread and handle the entire game-flow. The Weak-pointer to the game will expire when
    /// the game has ended.
    pub fn new(black: Weak<NetClient>, white: Weak<NetClient>) -> Option<Weak<Game>> {
        let black_arc = match black.upgrade() {
            Some(arc) => arc,
            None => return None
        };

        let white_arc = match white.upgrade() {
            Some(arc) => arc,
            None => return None
        };

        // The game can be started. Send the information to both the clients.
        black_arc.send(&Packet::StartGame(white_arc.id(), Piece::Black));
        white_arc.send(&Packet::StartGame(black_arc.id(), Piece::White));

        // Subscribe to both clients.
        let black_packets = Arc::new(Mutex::new(VecDeque::new()));
        let white_packets = Arc::new(Mutex::new(VecDeque::new()));
        black_arc.subscribe(Arc::downgrade(&black_packets));
        white_arc.subscribe(Arc::downgrade(&white_packets));

        let game = Arc::new(Game {
            board: Mutex::new(Board::new()),
            black: black,
            black_packets: black_packets,
            white: white,
            white_packets: white_packets
        });

        // The Weak reference that will be returned. The Arc<Game> will be captured by the new thread.
        let game_weak = Arc::downgrade(&game);

        // Everything is prepared. The Game-Thread can be started.
        thread::spawn(move || {
            while game.is_running() {
                game.handle_packets();
            }
        });

        Some(game_weak)
    }

    fn handle_packets(&self) {
        unimplemented!();
    }

    pub fn is_running(&self) -> bool {
        unimplemented!();
    }
}

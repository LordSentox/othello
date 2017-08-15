use std::sync::{Arc, Weak, Mutex};
use board::*;
use std::thread;
use std::time::Duration;
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

				thread::sleep(Duration::from_millis(50));
            }
        });

        Some(game_weak)
    }

    fn handle_packets(&self) {
		// Handle the packets of the black client.
		loop {
			let packet = match self.black_packets.lock().unwrap().pop_front() {
				Some(p) => p,
				None => break
			};

			self.handle_packet(packet, Piece::Black);
		}

		// Handle the packets of the white client.
		loop {
			let packet = match self.white_packets.lock().unwrap().pop_front() {
				Some(p) => p,
				None => break
			};

			self.handle_packet(packet, Piece::White);
		}
    }

	fn handle_packet(&self, packet: Packet, piece: Piece) {
		match packet {
			Packet::PlacePiece(opponent_id, x, y) => {
				let (player, opponent) = match self.player_opponent(piece) {
					Some(po) => po,
					None => return // One of the players has disconnected.
				};

				// Since the Game gets all packets from the client in question, the opponent might
				// not be the one played in this game. The packet can then be ignored.
				if opponent_id != opponent.id() {
					return;
				}

				// The stone can now be tried to set on the board, to check if it is a valid move.
				if self.board.lock().unwrap().place((x, y), piece) {
					// Inform the opponent of the move.
					if !opponent.send(&Packet::PlacePiece(player.id(), x, y)) {
						panic!("Could not send packet to opponent, leaving board in illegal state.");
					}
				}
			},
			Packet::Pass(opponent_id) => {
				let (player, opponent) = match self.player_opponent(piece) {
					Some(po) => po,
					None => return // One of the players has disconnected.
				};

				if opponent_id != opponent.id() {
					return;
				}

				// Check that the player who tried to pass is the one whos turn it is.
				let mut board_lock = self.board.lock().unwrap();
				if board_lock.turn() == piece {
					// Pass for the client who sent the packet and let the other client know that
					// their opponent has passed.
					board_lock.pass();
					opponent.send(&Packet::Pass(player.id()));
				}
			}
			_ => {}
		}
	}

	/// Tries to upgrade both the players and return them. The first NetClient returned is the
	/// one corresponding to the piece, the other is the opponent.
	fn player_opponent(&self, piece: Piece) -> Option<(Arc<NetClient>, Arc<NetClient>)> {
		let player = match self.get_player(piece).upgrade() {
			Some(p) => p,
			None => return None
		};

		let opponent = match self.get_player(piece.opposite()).upgrade() {
			Some(p) => p,
			None => return None
		};

		Some((player, opponent))
	}

	pub fn get_player(&self, piece: Piece) -> Weak<NetClient> {
		match piece {
			Piece::Black => self.black.clone(),
			Piece::White => self.white.clone()
		}
	}

    pub fn is_running(&self) -> bool {
		// TODO: This should check if the clients may have abandoned the game or the game is over.

		// Check that both players are still connected.
		self.white.upgrade().is_some() && self.black.upgrade().is_some()
    }
}

//! Petit utilitaire en ligne de commande pour lancer un perft depuis une
//! FEN arbitraire — pratique pour comparer manuellement à d'autres moteurs
//! de référence (Stockfish `go perft N`, par ex.) sans passer par les
//! tests automatisés.
//!
//! Usage : `chess-perft [depth] [fen...]` (par défaut : position de
//! départ, profondeur 4).

use std::env;

use chess_engine::board::Board;
use chess_engine::perft::perft;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let depth: u32 = args.first().and_then(|s| s.parse().ok()).unwrap_or(4);

    let fen = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        chess_engine::board::STARTING_FEN.to_string()
    };

    let board = match Board::from_fen(&fen) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("FEN invalide : {e}");
            std::process::exit(1);
        }
    };

    for d in 1..=depth {
        let nodes = perft(&board, d);
        println!("perft({d}) = {nodes}");
    }
}

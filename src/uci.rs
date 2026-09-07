//! Sous-ensemble du protocole [UCI](https://en.wikipedia.org/wiki/Universal_Chess_Interface)
//! (*Universal Chess Interface*) : suffisant pour être piloté par n'importe
//! quelle interface graphique d'échecs standard (`cutechess`, une CLI UCI,
//! etc.), sans dépendre du protocole plus ancien et plus verbeux XBoard.
//!
//! Le traitement des commandes est séparé de toute lecture stdin/stdout
//! (`handle_line` prend une `&str` et renvoie les lignes de réponse sous
//! forme de `Vec<String>`) précisément pour rester testable sans processus
//! réel — le binaire `chess-uci` (`src/bin/uci.rs`) n'est qu'une boucle
//! `stdin`/`stdout` autour de cette logique.

use crate::board::Board;
use crate::movegen::legal_moves;
use crate::moves::Move;
use crate::search::{self, SearchLimits};

pub const ENGINE_NAME: &str = "Chess Engine";
pub const ENGINE_AUTHOR: &str = "Mahouna";
const DEFAULT_SEARCH_DEPTH: u32 = 5;

pub struct UciEngine {
    board: Board,
}

impl Default for UciEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl UciEngine {
    pub fn new() -> Self {
        UciEngine {
            board: Board::starting_position(),
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Traite une ligne de commande UCI et renvoie les lignes de réponse à
    /// écrire sur la sortie standard (peut être vide).
    pub fn handle_line(&mut self, line: &str) -> Vec<String> {
        let mut tokens = line.split_whitespace();
        match tokens.next() {
            Some("uci") => vec![
                format!("id name {ENGINE_NAME}"),
                format!("id author {ENGINE_AUTHOR}"),
                "uciok".to_string(),
            ],
            Some("isready") => vec!["readyok".to_string()],
            Some("ucinewgame") => {
                self.board = Board::starting_position();
                vec![]
            }
            Some("position") => {
                self.handle_position(tokens);
                vec![]
            }
            Some("go") => {
                let depth = parse_go_depth(tokens).unwrap_or(DEFAULT_SEARCH_DEPTH);
                let info = search::search(&self.board, SearchLimits::depth(depth));
                let uci_move = info
                    .best_move
                    .map(|mv| mv.to_uci())
                    .unwrap_or_else(|| "0000".to_string());
                vec![
                    format!(
                        "info depth {} score cp {} nodes {}",
                        info.depth, info.score, info.nodes
                    ),
                    format!("bestmove {uci_move}"),
                ]
            }
            Some("quit") | Some("stop") => vec![],
            _ => vec![],
        }
    }

    fn handle_position(&mut self, mut tokens: std::str::SplitWhitespace) {
        match tokens.next() {
            Some("startpos") => self.board = Board::starting_position(),
            Some("fen") => {
                let fen_fields: Vec<&str> = tokens.clone().take_while(|&t| t != "moves").collect();
                if let Ok(board) = Board::from_fen(&fen_fields.join(" ")) {
                    self.board = board;
                }
                for _ in 0..fen_fields.len() {
                    tokens.next();
                }
            }
            _ => return,
        }

        if tokens.next() == Some("moves") {
            for mv_str in tokens {
                if let Some(mv) = find_move_by_uci(&self.board, mv_str) {
                    self.board = self.board.make_move(mv);
                }
                // Un coup UCI non reconnu (mal formé, ou illégal dans la
                // position courante) est silencieusement ignoré plutôt que
                // de paniquer : une interface graphique buguée ne doit pas
                // faire planter le moteur.
            }
        }
    }
}

fn parse_go_depth(mut tokens: std::str::SplitWhitespace) -> Option<u32> {
    while let Some(token) = tokens.next() {
        if token == "depth" {
            return tokens.next().and_then(|d| d.parse().ok());
        }
    }
    None
}

fn find_move_by_uci(board: &Board, uci: &str) -> Option<Move> {
    legal_moves(board).into_iter().find(|mv| mv.to_uci() == uci)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uci_handshake_returns_id_and_uciok() {
        let mut engine = UciEngine::new();
        let response = engine.handle_line("uci");
        assert!(response.iter().any(|l| l.starts_with("id name")));
        assert_eq!(response.last().unwrap(), "uciok");
    }

    #[test]
    fn isready_returns_readyok() {
        let mut engine = UciEngine::new();
        assert_eq!(engine.handle_line("isready"), vec!["readyok".to_string()]);
    }

    #[test]
    fn position_startpos_with_moves_updates_the_board() {
        let mut engine = UciEngine::new();
        engine.handle_line("position startpos moves e2e4 e7e5");
        // La case de prise en passant e6 reste ouverte après 1...e5 (double
        // poussée de pion), tant qu'aucun coup ne l'a "consommée" ou fait
        // expirer : c'est correct, pas un oubli du test.
        assert_eq!(
            engine.board().to_fen(),
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2"
        );
    }

    #[test]
    fn position_fen_loads_an_arbitrary_position() {
        let mut engine = UciEngine::new();
        let fen = "8/8/8/8/8/8/8/K6k w - - 0 1";
        engine.handle_line(&format!("position fen {fen}"));
        assert_eq!(engine.board().to_fen(), fen);
    }

    #[test]
    fn go_returns_a_legal_bestmove_from_the_starting_position() {
        let mut engine = UciEngine::new();
        let response = engine.handle_line("go depth 2");
        let bestmove_line = response
            .iter()
            .find(|l| l.starts_with("bestmove"))
            .expect("une ligne bestmove doit être renvoyée");
        let mv_str = bestmove_line.strip_prefix("bestmove ").unwrap();
        assert!(find_move_by_uci(engine.board(), mv_str).is_some());
    }
}

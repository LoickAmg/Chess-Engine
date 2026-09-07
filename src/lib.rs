//! Moteur d'échecs 2D... non, 8x8 : représentation de plateau, génération de
//! coups légale, évaluation, recherche par élagage alpha-bêta, protocole UCI.
//!
//! Zéro dépendance externe pour le cœur du moteur (`board`, `movegen`,
//! `eval`, `search`) — seul le binaire `chess-uci` fait de l'I/O sur
//! stdin/stdout, sans bibliothèque tierce non plus.

pub mod board;
pub mod eval;
pub mod movegen;
pub mod moves;
pub mod perft;
pub mod piece;
pub mod search;
pub mod uci;

pub use board::Board;
pub use moves::{Move, MoveFlag};
pub use piece::{Color, Piece, PieceType, Square};

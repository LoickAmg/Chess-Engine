//! Notation algébrique standard (SAN) : `Cf3`, `exd5`, `O-O`, `e8=D+`, `Dxf7#`.
//!
//! En français les pièces s'écrivent R (roi), D (dame), T (tour), F (fou), C (cavalier) ;
//! en anglais K, Q, R, B, N. Le pion n'a pas de lettre.

use crate::board::Board;
use crate::movegen::legal_moves;
use crate::moves::{Move, MoveFlag};
use crate::piece::PieceType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    French,
    English,
}

pub fn piece_letter(kind: PieceType, lang: Language) -> &'static str {
    match (lang, kind) {
        (_, PieceType::Pawn) => "",
        (Language::French, PieceType::Knight) => "C",
        (Language::French, PieceType::Bishop) => "F",
        (Language::French, PieceType::Rook) => "T",
        (Language::French, PieceType::Queen) => "D",
        (Language::French, PieceType::King) => "R",
        (Language::English, PieceType::Knight) => "N",
        (Language::English, PieceType::Bishop) => "B",
        (Language::English, PieceType::Rook) => "R",
        (Language::English, PieceType::Queen) => "Q",
        (Language::English, PieceType::King) => "K",
    }
}

/// Notation d'un coup légal `mv` joué dans `board`.
pub fn to_san(board: &Board, mv: Move, lang: Language) -> String {
    let mut san = match mv.flag {
        MoveFlag::CastleKingside => "O-O".to_string(),
        MoveFlag::CastleQueenside => "O-O-O".to_string(),
        _ => {
            let piece = board
                .piece_at(mv.from)
                .expect("un coup part d'une case occupée");
            let capture = mv.flag.is_capture();
            let mut out = String::new();
            if piece.kind == PieceType::Pawn {
                if capture {
                    out.push((b'a' + mv.from.file()) as char);
                }
            } else {
                out.push_str(piece_letter(piece.kind, lang));
                out.push_str(&disambiguation(board, mv, piece.kind));
            }
            if capture {
                out.push('x');
            }
            out.push_str(&mv.to.to_algebraic());
            if let Some(promo) = mv.flag.promotion() {
                out.push('=');
                out.push_str(piece_letter(promo, lang));
            }
            out
        }
    };
    let next = board.make_move(mv);
    if next.is_in_check(next.side_to_move) {
        san.push(if legal_moves(&next).is_empty() {
            '#'
        } else {
            '+'
        });
    }
    san
}

/// Lettre de colonne, chiffre de rangée, ou les deux quand plusieurs pièces du même type
/// peuvent atteindre la même case (`Cbd2`, `T1e1`, `Dh4e1`).
fn disambiguation(board: &Board, mv: Move, kind: PieceType) -> String {
    let rivals: Vec<Move> = legal_moves(board)
        .into_iter()
        .filter(|other| {
            other.to == mv.to
                && other.from != mv.from
                && board.piece_at(other.from).map(|p| p.kind) == Some(kind)
        })
        .collect();
    if rivals.is_empty() {
        return String::new();
    }
    let file = (b'a' + mv.from.file()) as char;
    let rank = (b'1' + mv.from.rank()) as char;
    if rivals.iter().all(|r| r.from.file() != mv.from.file()) {
        file.to_string()
    } else if rivals.iter().all(|r| r.from.rank() != mv.from.rank()) {
        rank.to_string()
    } else {
        format!("{file}{rank}")
    }
}

/// Retrouve un coup légal à partir de sa notation UCI (`e2e4`, `e7e8q`).
pub fn parse_uci(board: &Board, uci: &str) -> Option<Move> {
    legal_moves(board).into_iter().find(|m| m.to_uci() == uci)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn san(fen: &str, uci: &str, lang: Language) -> String {
        let board = Board::from_fen(fen).unwrap();
        let mv = parse_uci(&board, uci).unwrap_or_else(|| panic!("{uci} illégal"));
        to_san(&board, mv, lang)
    }

    const START: &str = crate::board::STARTING_FEN;

    #[test]
    fn simple_moves_in_both_languages() {
        assert_eq!(san(START, "e2e4", Language::French), "e4");
        assert_eq!(san(START, "g1f3", Language::French), "Cf3");
        assert_eq!(san(START, "g1f3", Language::English), "Nf3");
    }

    #[test]
    fn captures_castling_and_check() {
        let fen = "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5Q2/PPPP1PPP/RNB1K1NR w KQkq - 2 3";
        assert_eq!(san(fen, "f3f7", Language::French), "Dxf7#");
        let castle = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        assert_eq!(san(castle, "e1g1", Language::French), "O-O");
        assert_eq!(san(castle, "e1c1", Language::French), "O-O-O");
        let pawn = "4k3/8/8/3p4/4P3/8/8/4K3 w - - 0 1";
        assert_eq!(san(pawn, "e4d5", Language::French), "exd5");
    }

    #[test]
    fn promotion_and_disambiguation() {
        let promo = "8/4P3/8/8/8/8/k7/4K3 w - - 0 1";
        assert_eq!(san(promo, "e7e8q", Language::French), "e8=D");
        let knights = "4k3/8/8/8/8/8/8/1N2KN2 w - - 0 1";
        assert_eq!(san(knights, "b1d2", Language::French), "Cbd2");
        let rooks = "4k3/8/8/R7/8/8/8/R3K3 w - - 0 1";
        assert_eq!(san(rooks, "a1a3", Language::French), "T1a3");
    }
}

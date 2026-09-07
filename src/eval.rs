//! Fonction d'évaluation statique : donne un score en centipions (1/100 de
//! pion) pour une position, du point de vue du camp au trait (convention
//! "négamax" — un score positif est toujours bon pour celui qui doit jouer).
//!
//! Deux composantes seulement, volontairement simples pour un projet
//! pédagogique : matériel (valeurs de pièces classiques) et position
//! (tables pièce-case — une même pièce ne vaut pas la même chose selon la
//! case qu'elle occupe, par ex. un cavalier au bord du plateau contrôle
//! moins de cases qu'un cavalier centralisé). Pas de phase de partie
//! (milieu de partie / finale) distincte : c'est une simplification connue
//! et documentée (voir README) — un roi centralisé est ainsi toujours
//! légèrement pénalisé même en finale, où c'est pourtant un atout.

use crate::board::Board;
use crate::piece::{Color, Piece, PieceType, Square};

pub const PAWN_VALUE: i32 = 100;
pub const KNIGHT_VALUE: i32 = 320;
pub const BISHOP_VALUE: i32 = 330;
pub const ROOK_VALUE: i32 = 500;
pub const QUEEN_VALUE: i32 = 900;
pub const KING_VALUE: i32 = 20_000;

pub fn piece_value(kind: PieceType) -> i32 {
    match kind {
        PieceType::Pawn => PAWN_VALUE,
        PieceType::Knight => KNIGHT_VALUE,
        PieceType::Bishop => BISHOP_VALUE,
        PieceType::Rook => ROOK_VALUE,
        PieceType::Queen => QUEEN_VALUE,
        PieceType::King => KING_VALUE,
    }
}

// Tables pièce-case, du point de vue des blancs (rangée 0 = rang 1, la
// rangée du camp blanc), en centipions. Valeurs "maison", inspirées des
// heuristiques standard largement enseignées (centraliser cavaliers/fous,
// avancer les pions vers la promotion, garder le roi à l'abri avant la
// finale) plutôt que reproduites depuis une table publiée précise.
#[rustfmt::skip]
const PAWN_TABLE: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
      5,  10,  10, -20, -20,  10,  10,   5,
      5,  -5, -10,   0,   0, -10,  -5,   5,
      0,   0,   0,  20,  20,   0,   0,   0,
      5,   5,  10,  25,  25,  10,   5,   5,
     10,  10,  20,  30,  30,  20,  10,  10,
     50,  50,  50,  50,  50,  50,  50,  50,
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
const KNIGHT_TABLE: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -30,   5,  10,  15,  15,  10,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

#[rustfmt::skip]
const BISHOP_TABLE: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   5,   0,   0,   0,   0,   5, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,   0,  10,  10,  10,  10,   0, -10,
    -10,   5,   5,  10,  10,   5,   5, -10,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

#[rustfmt::skip]
const ROOK_TABLE: [i32; 64] = [
      0,   0,   5,  10,  10,   5,   0,   0,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
      5,  10,  10,  10,  10,  10,  10,   5,
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
const QUEEN_TABLE: [i32; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   5,   0,   0,   0,   0, -10,
    -10,   5,   5,   5,   5,   5,   0, -10,
      0,   0,   5,   5,   5,   5,   0,  -5,
     -5,   0,   5,   5,   5,   5,   0,  -5,
    -10,   0,   5,   5,   5,   5,   0, -10,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -20, -10, -10,  -5,  -5, -10, -10, -20,
];

#[rustfmt::skip]
const KING_TABLE: [i32; 64] = [
     20,  30,  10,   0,   0,  10,  30,  20,
     20,  20,   0,   0,   0,   0,  20,  20,
    -10, -20, -20, -20, -20, -20, -20, -10,
    -20, -30, -30, -40, -40, -30, -30, -20,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
];

fn piece_square_bonus(kind: PieceType, sq: Square, color: Color) -> i32 {
    // Les tables ci-dessus sont écrites du point de vue des blancs (index 0
    // = rang 1). Pour les noirs, on symétrise verticalement en inversant
    // le rang — leur rang 1 (leur camp de départ) correspond à l'index 56.
    let index = match color {
        Color::White => sq.index(),
        Color::Black => Square::new(sq.file(), 7 - sq.rank()).index(),
    };
    match kind {
        PieceType::Pawn => PAWN_TABLE[index],
        PieceType::Knight => KNIGHT_TABLE[index],
        PieceType::Bishop => BISHOP_TABLE[index],
        PieceType::Rook => ROOK_TABLE[index],
        PieceType::Queen => QUEEN_TABLE[index],
        PieceType::King => KING_TABLE[index],
    }
}

fn piece_score(piece: Piece, sq: Square) -> i32 {
    piece_value(piece.kind) + piece_square_bonus(piece.kind, sq, piece.color)
}

/// Score matériel + positionnel, du point de vue des BLANCS (positif =
/// avantage blanc). `evaluate` (ci-dessous) le convertit ensuite du point
/// de vue du camp au trait pour le négamax.
pub fn evaluate_white_relative(board: &Board) -> i32 {
    let mut score = 0;
    for (sq, piece) in board.pieces_of(Color::White) {
        score += piece_score(piece, sq);
    }
    for (sq, piece) in board.pieces_of(Color::Black) {
        score -= piece_score(piece, sq);
    }
    score
}

/// Score du point de vue du camp au trait — convention négamax : un score
/// positif est toujours bon pour `board.side_to_move`.
pub fn evaluate(board: &Board) -> i32 {
    let white_relative = evaluate_white_relative(board);
    match board.side_to_move {
        Color::White => white_relative,
        Color::Black => -white_relative,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_position_is_exactly_balanced() {
        let board = Board::starting_position();
        assert_eq!(evaluate_white_relative(&board), 0);
        assert_eq!(evaluate(&board), 0);
    }

    #[test]
    fn an_extra_queen_is_a_large_material_advantage() {
        let with_extra_queen = Board::from_fen("4k3/8/8/8/8/8/8/QQQQK3 w - - 0 1").unwrap();
        let baseline = Board::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let gain = evaluate_white_relative(&with_extra_queen) - evaluate_white_relative(&baseline);
        assert!(gain > 3 * QUEEN_VALUE);
    }

    #[test]
    fn evaluate_flips_sign_depending_on_side_to_move() {
        let white_to_move = Board::from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1").unwrap();
        let black_to_move = Board::from_fen("4k3/8/8/8/8/8/4P3/4K3 b - - 0 1").unwrap();
        assert_eq!(evaluate(&white_to_move), -evaluate(&black_to_move));
    }

    #[test]
    fn a_centralized_knight_is_valued_above_a_cornered_one() {
        let centralized = Board::from_fen("4k3/8/8/3N4/8/8/8/4K3 w - - 0 1").unwrap();
        let cornered = Board::from_fen("4k3/8/8/8/8/8/8/N3K3 w - - 0 1").unwrap();
        assert!(evaluate_white_relative(&centralized) > evaluate_white_relative(&cornered));
    }
}

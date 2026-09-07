//! Représentation d'un coup.
//!
//! Un `Move` ne porte que `from`/`to`/`flag` — il ne sait pas tout seul
//! quelle pièce bouge ni ce qu'il capture, cette information vit dans le
//! plateau au moment de `make_move`. C'est délibéré : deux coups identiques
//! en `from`/`to`/`flag` sont bien le même coup peu importe le plateau, ce
//! qui simplifie les comparaisons (utile pour le tri des coups, les tables
//! de transposition, et la correspondance "coup joué == coup attendu" dans
//! les tests).

use crate::piece::{PieceType, Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveFlag {
    Quiet,
    DoublePawnPush,
    Capture,
    EnPassantCapture,
    CastleKingside,
    CastleQueenside,
    Promotion(PieceType),
    PromotionCapture(PieceType),
}

impl MoveFlag {
    pub fn is_capture(self) -> bool {
        matches!(
            self,
            MoveFlag::Capture | MoveFlag::EnPassantCapture | MoveFlag::PromotionCapture(_)
        )
    }

    pub fn promotion(self) -> Option<PieceType> {
        match self {
            MoveFlag::Promotion(p) | MoveFlag::PromotionCapture(p) => Some(p),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub flag: MoveFlag,
}

impl Move {
    pub fn new(from: Square, to: Square, flag: MoveFlag) -> Move {
        Move { from, to, flag }
    }

    /// Notation "coordonnée" façon UCI : `e2e4`, `e7e8q` pour une promotion.
    pub fn to_uci(self) -> String {
        let promo = match self.flag.promotion() {
            Some(PieceType::Knight) => "n",
            Some(PieceType::Bishop) => "b",
            Some(PieceType::Rook) => "r",
            Some(PieceType::Queen) => "q",
            _ => "",
        };
        format!("{}{}{}", self.from, self.to, promo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uci_notation_for_a_quiet_move() {
        let m = Move::new(
            Square::from_algebraic("e2").unwrap(),
            Square::from_algebraic("e4").unwrap(),
            MoveFlag::DoublePawnPush,
        );
        assert_eq!(m.to_uci(), "e2e4");
    }

    #[test]
    fn uci_notation_for_a_promotion() {
        let m = Move::new(
            Square::from_algebraic("e7").unwrap(),
            Square::from_algebraic("e8").unwrap(),
            MoveFlag::Promotion(PieceType::Queen),
        );
        assert_eq!(m.to_uci(), "e7e8q");
    }

    #[test]
    fn is_capture_covers_all_capture_variants() {
        assert!(MoveFlag::Capture.is_capture());
        assert!(MoveFlag::EnPassantCapture.is_capture());
        assert!(MoveFlag::PromotionCapture(PieceType::Queen).is_capture());
        assert!(!MoveFlag::Quiet.is_capture());
        assert!(!MoveFlag::Promotion(PieceType::Queen).is_capture());
    }
}

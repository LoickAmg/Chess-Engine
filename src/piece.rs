//! Types de base : couleur, type de pièce, pièce, case.
//!
//! Une case est stockée comme un seul `u8` dans `0..64` (0 = a1, 7 = h1,
//! 56 = a8, 63 = h8 — l'indexation "little-endian rank-file" standard des
//! moteurs d'échecs) plutôt qu'une paire `(file, rank)`, pour que le
//! plateau (`board.rs`) puisse s'indexer directement par un tableau de 64
//! cases sans conversion.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// +1 pour les blancs (qui avancent vers les rangs croissants), -1 pour
    /// les noirs. Utilisé pour la direction d'avance des pions et le signe
    /// de l'évaluation relative au camp au trait.
    pub fn pawn_direction(self) -> i8 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }

    pub fn home_rank(self) -> u8 {
        match self {
            Color::White => 0,
            Color::Black => 7,
        }
    }

    pub fn promotion_rank(self) -> u8 {
        match self {
            Color::White => 7,
            Color::Black => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    pub const ALL: [PieceType; 6] = [
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
        PieceType::King,
    ];

    /// Lettre FEN majuscule (le contexte applique la casse selon la couleur).
    pub fn to_fen_char(self) -> char {
        match self {
            PieceType::Pawn => 'P',
            PieceType::Knight => 'N',
            PieceType::Bishop => 'B',
            PieceType::Rook => 'R',
            PieceType::Queen => 'Q',
            PieceType::King => 'K',
        }
    }

    pub fn from_fen_char(c: char) -> Option<PieceType> {
        match c.to_ascii_uppercase() {
            'P' => Some(PieceType::Pawn),
            'N' => Some(PieceType::Knight),
            'B' => Some(PieceType::Bishop),
            'R' => Some(PieceType::Rook),
            'Q' => Some(PieceType::Queen),
            'K' => Some(PieceType::King),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceType,
}

impl Piece {
    pub fn new(color: Color, kind: PieceType) -> Self {
        Piece { color, kind }
    }

    pub fn to_fen_char(self) -> char {
        let c = self.kind.to_fen_char();
        match self.color {
            Color::White => c,
            Color::Black => c.to_ascii_lowercase(),
        }
    }

    pub fn from_fen_char(c: char) -> Option<Piece> {
        let kind = PieceType::from_fen_char(c)?;
        let color = if c.is_ascii_uppercase() {
            Color::White
        } else {
            Color::Black
        };
        Some(Piece::new(color, kind))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Square(pub u8);

impl Square {
    pub const COUNT: usize = 64;

    pub fn new(file: u8, rank: u8) -> Square {
        debug_assert!(file < 8 && rank < 8);
        Square(rank * 8 + file)
    }

    pub fn from_index(index: u8) -> Square {
        debug_assert!(index < 64);
        Square(index)
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }

    pub fn file(self) -> u8 {
        self.0 % 8
    }

    pub fn rank(self) -> u8 {
        self.0 / 8
    }

    /// Retourne la case décalée de `(delta_file, delta_rank)`, ou `None` si
    /// le résultat sortirait du plateau — c'est ce test de débordement qui
    /// permet à la génération de coups de rester correcte sur les bords
    /// (un cavalier en a1 ne "revient" pas par h-quelquechose).
    pub fn try_offset(self, delta_file: i8, delta_rank: i8) -> Option<Square> {
        let file = self.file() as i8 + delta_file;
        let rank = self.rank() as i8 + delta_rank;
        if (0..8).contains(&file) && (0..8).contains(&rank) {
            Some(Square::new(file as u8, rank as u8))
        } else {
            None
        }
    }

    pub fn from_algebraic(s: &str) -> Option<Square> {
        let bytes = s.as_bytes();
        if bytes.len() != 2 {
            return None;
        }
        let file = bytes[0];
        let rank = bytes[1];
        if !(b'a'..=b'h').contains(&file) || !(b'1'..=b'8').contains(&rank) {
            return None;
        }
        Some(Square::new(file - b'a', rank - b'1'))
    }

    pub fn to_algebraic(self) -> String {
        let file = (b'a' + self.file()) as char;
        let rank = (b'1' + self.rank()) as char;
        format!("{file}{rank}")
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_algebraic())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_new_and_index_round_trip() {
        assert_eq!(Square::new(0, 0), Square(0)); // a1
        assert_eq!(Square::new(7, 0), Square(7)); // h1
        assert_eq!(Square::new(0, 7), Square(56)); // a8
        assert_eq!(Square::new(7, 7), Square(63)); // h8
    }

    #[test]
    fn square_file_and_rank_are_recovered_correctly() {
        let sq = Square::new(3, 5); // d6
        assert_eq!(sq.file(), 3);
        assert_eq!(sq.rank(), 5);
    }

    #[test]
    fn algebraic_round_trip_for_every_square() {
        for rank in 0..8 {
            for file in 0..8 {
                let sq = Square::new(file, rank);
                let alg = sq.to_algebraic();
                assert_eq!(Square::from_algebraic(&alg), Some(sq));
            }
        }
    }

    #[test]
    fn try_offset_rejects_out_of_board_targets() {
        let a1 = Square::from_algebraic("a1").unwrap();
        assert_eq!(a1.try_offset(-1, 0), None);
        assert_eq!(a1.try_offset(0, -1), None);
        assert_eq!(a1.try_offset(1, 1), Square::from_algebraic("b2"));

        let h8 = Square::from_algebraic("h8").unwrap();
        assert_eq!(h8.try_offset(1, 0), None);
        assert_eq!(h8.try_offset(0, 1), None);
    }

    #[test]
    fn piece_fen_char_round_trip() {
        for &kind in PieceType::ALL.iter() {
            for &color in &[Color::White, Color::Black] {
                let piece = Piece::new(color, kind);
                let c = piece.to_fen_char();
                assert_eq!(Piece::from_fen_char(c), Some(piece));
            }
        }
    }

    #[test]
    fn color_opposite_is_involutive() {
        assert_eq!(Color::White.opposite(), Color::Black);
        assert_eq!(Color::Black.opposite(), Color::White);
        assert_eq!(Color::White.opposite().opposite(), Color::White);
    }
}

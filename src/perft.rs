//! Perft ("**per**formance **t**est") : compte le nombre de positions
//! atteignables à une profondeur donnée depuis une position de départ.
//!
//! C'est la méthode standard, largement utilisée par la communauté des
//! programmeurs d'échecs, pour **prouver** qu'un générateur de coups est
//! correct plutôt que de le croire sur parole : les comptages exacts pour
//! la position de départ et plusieurs positions de référence (dont
//! "Kiwipete", conçue spécifiquement pour piéger les bugs de roque/prise en
//! passant/promotion) sont connus et publiés. Un générateur avec un bug —
//! un roque mal filtré, une prise en passant oubliée dans un coin du
//! plateau — produit presque toujours un écart au perft, même quand les
//! quelques positions testées "à la main" semblent correctes.

use crate::board::Board;
use crate::movegen;

pub fn perft(board: &Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }
    let moves = movegen::legal_moves(board);
    if depth == 1 {
        return moves.len() as u64;
    }
    moves
        .into_iter()
        .map(|mv| perft(&board.make_move(mv), depth - 1))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Comptages de référence pour la position de départ, largement publiés
    // (par ex. Chess Programming Wiki, "Perft Results").
    const STARTPOS_PERFT: [u64; 5] = [1, 20, 400, 8_902, 197_281];

    #[test]
    fn perft_matches_known_values_for_the_starting_position() {
        let board = Board::starting_position();
        for (depth, &expected) in STARTPOS_PERFT.iter().enumerate() {
            assert_eq!(
                perft(&board, depth as u32),
                expected,
                "perft({depth}) incorrect pour la position de départ"
            );
        }
    }

    #[test]
    fn perft_matches_known_values_for_kiwipete() {
        // "Kiwipete", position de test très utilisée conçue par Peter
        // McKenzie pour exercer roques, prises en passant et promotions
        // simultanément.
        let board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
                .unwrap();
        assert_eq!(perft(&board, 1), 48);
        assert_eq!(perft(&board, 2), 2_039);
        assert_eq!(perft(&board, 3), 97_862);
    }

    #[test]
    fn perft_matches_known_values_for_a_position_with_en_passant_pins() {
        // Position de test n°3 de la suite perft classique (Steven Edwards) :
        // met en jeu des clouages combinés à des prises en passant.
        let board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
        assert_eq!(perft(&board, 1), 14);
        assert_eq!(perft(&board, 2), 191);
        assert_eq!(perft(&board, 3), 2_812);
        assert_eq!(perft(&board, 4), 43_238);
    }

    #[test]
    #[ignore = "plus lent (~5M feuilles) : lancer explicitement avec --ignored pour une vérification approfondie"]
    fn perft_five_matches_known_value_for_the_starting_position() {
        let board = Board::starting_position();
        assert_eq!(perft(&board, 5), 4_865_609);
    }
}

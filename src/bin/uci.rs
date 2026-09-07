//! Boucle UCI : lit des commandes sur stdin, écrit les réponses sur
//! stdout. Toute la logique vit dans `chess_engine::uci::UciEngine` — ce
//! binaire n'est qu'une fine couche d'E/S, ce qui la rend testable sans
//! lancer de vrai processus (voir les tests dans `src/uci.rs`).

use std::io::{self, BufRead, Write};

use chess_engine::uci::UciEngine;

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut engine = UciEngine::new();

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let trimmed = line.trim();
        if trimmed == "quit" {
            break;
        }
        for response_line in engine.handle_line(trimmed) {
            let _ = writeln!(out, "{response_line}");
        }
        let _ = out.flush();
    }
}

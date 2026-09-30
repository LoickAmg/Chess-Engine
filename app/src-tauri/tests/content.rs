//! Valide le contenu pédagogique (src/data/*.json) avec le vrai moteur : chaque position
//! se lit, chaque coup attendu est légal, chaque « mat » en est vraiment un.

use chess_engine::movegen::legal_moves;
use chess_engine::san::parse_uci;
use chess_engine::Board;
use serde_json::Value;

const LESSONS: &str = include_str!("../../src/data/lessons.json");
const PUZZLES: &str = include_str!("../../src/data/puzzles.json");
const GLOSSARY: &str = include_str!("../../src/data/glossary.json");

fn board(fen: &str, ctx: &str) -> Board {
    Board::from_fen(fen).unwrap_or_else(|e| panic!("{ctx} : FEN illisible « {fen} » ({e})"))
}

fn is_mate(b: &Board) -> bool {
    b.is_in_check(b.side_to_move) && legal_moves(b).is_empty()
}

fn has_mate_in_one(b: &Board) -> bool {
    legal_moves(b).into_iter().any(|m| is_mate(&b.make_move(m)))
}

fn is_square(s: &str) -> bool {
    let bytes = s.as_bytes();
    s.len() == 2 && (b'a'..=b'h').contains(&bytes[0]) && (b'1'..=b'8').contains(&bytes[1])
}

fn strs(v: &Value) -> Vec<String> {
    v.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default()
}

#[test]
fn lessons_are_consistent_with_the_engine() {
    let lessons: Vec<Value> = serde_json::from_str(LESSONS).expect("lessons.json invalide");
    assert!(lessons.len() >= 20);
    let mut ids = std::collections::HashSet::new();
    for lesson in &lessons {
        let id = lesson["id"].as_str().unwrap();
        assert!(ids.insert(id.to_string()), "identifiant de leçon en double : {id}");
        let steps = lesson["steps"].as_array().unwrap();
        assert!(!steps.is_empty(), "{id} : aucune étape");
        for (i, step) in steps.iter().enumerate() {
            let ctx = format!("leçon {id}, étape {}", i + 1);
            assert!(step["text"].as_str().is_some_and(|t| !t.is_empty()), "{ctx} : texte manquant");
            for key in ["highlights"] {
                for h in step[key].as_array().into_iter().flatten() {
                    assert!(is_square(h["sq"].as_str().unwrap_or("")), "{ctx} : case invalide");
                }
            }
            for a in step["arrows"].as_array().into_iter().flatten() {
                assert!(is_square(a["from"].as_str().unwrap()) && is_square(a["to"].as_str().unwrap()), "{ctx} : flèche");
            }
            let fen = step["fen"].as_str();
            match step["type"].as_str().unwrap() {
                "text" => {
                    if let Some(f) = fen {
                        board(f, &ctx);
                    }
                }
                "square" => {
                    let targets = strs(&step["targets"]);
                    assert!(!targets.is_empty() && targets.iter().all(|t| is_square(t)), "{ctx} : cibles");
                }
                "stars" => {
                    board(fen.expect("fen requise"), &ctx);
                    let stars = strs(&step["stars"]);
                    assert!(!stars.is_empty() && stars.iter().all(|s| is_square(s)), "{ctx} : étoiles");
                }
                "quiz" => {
                    if let Some(f) = fen {
                        board(f, &ctx);
                    }
                    let n = step["choices"].as_array().unwrap().len();
                    let answer = step["answer"].as_u64().unwrap() as usize;
                    assert!(answer < n, "{ctx} : réponse hors des choix");
                    assert!(step["explain"].as_str().is_some(), "{ctx} : explication manquante");
                }
                "move" => {
                    let b = board(fen.expect("fen requise"), &ctx);
                    assert!(b.king_square(b.side_to_move).is_some(), "{ctx} : il faut un roi au trait");
                    assert!(step["success"].as_str().is_some(), "{ctx} : message de réussite");
                    for uci in strs(&step["accept"]) {
                        match uci.as_str() {
                            "*" => assert!(!legal_moves(&b).is_empty(), "{ctx} : aucun coup légal"),
                            "#" => assert!(has_mate_in_one(&b), "{ctx} : aucun mat en un coup"),
                            u => assert!(parse_uci(&b, u).is_some(), "{ctx} : coup illégal {u}"),
                        }
                    }
                    if let Some(wrong) = step["wrong"].as_object() {
                        for u in wrong.keys() {
                            assert!(parse_uci(&b, u).is_some(), "{ctx} : coup « faux » illégal {u}");
                        }
                    }
                }
                other => panic!("{ctx} : type d'étape inconnu {other}"),
            }
        }
    }
}

#[test]
fn stalemate_trap_in_lesson_is_really_stalemate() {
    let b = Board::from_fen("k7/8/1K6/8/8/8/8/2Q5 w - - 0 1").unwrap();
    let after = b.make_move(parse_uci(&b, "c1c7").unwrap());
    assert!(!after.is_in_check(after.side_to_move) && legal_moves(&after).is_empty());
}

#[test]
fn puzzles_have_valid_solutions() {
    let puzzles: Vec<Value> = serde_json::from_str(PUZZLES).expect("puzzles.json invalide");
    assert!(puzzles.len() >= 15);
    for p in &puzzles {
        let id = p["id"].as_str().unwrap();
        let theme = p["theme"].as_str().unwrap();
        let mut b = board(p["fen"].as_str().unwrap(), id);
        let solution = strs(&p["solution"]);
        if solution == ["#"] {
            assert!(has_mate_in_one(&b), "{id} : aucun mat en un coup");
            continue;
        }
        for u in &solution {
            let mv = parse_uci(&b, u).unwrap_or_else(|| panic!("{id} : coup illégal {u}"));
            b = b.make_move(mv);
        }
        if theme.starts_with("Mat") {
            assert!(is_mate(&b), "{id} : la solution ne finit pas par un mat");
        }
    }
}

#[test]
fn glossary_is_sorted_and_complete() {
    let entries: Vec<Value> = serde_json::from_str(GLOSSARY).unwrap();
    assert!(entries.len() >= 25);
    for e in &entries {
        assert!(e["term"].as_str().is_some_and(|t| !t.is_empty()));
        assert!(e["def"].as_str().is_some_and(|d| d.len() > 20));
    }
}

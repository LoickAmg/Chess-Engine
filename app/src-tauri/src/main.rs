// Pas de console en plus de la fenêtre sous Windows (en version finale).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    chess_academie_lib::run()
}

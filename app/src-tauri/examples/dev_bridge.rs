//! Passerelle de développement : expose les commandes du moteur en HTTP local pour tester
//! l'interface dans un navigateur (`npm run dev`) sans lancer la fenêtre Tauri.
//!
//!     cargo run --release --example dev_bridge
//!
//! Requête : `POST http://127.0.0.1:1431/<commande>` avec les arguments en JSON.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:1431").expect("port 1431 occupé");
    println!("Passerelle Chess Académie sur http://127.0.0.1:1431");
    for stream in listener.incoming().flatten() {
        thread::spawn(move || {
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request_line = String::new();
            if reader.read_line(&mut request_line).is_err() {
                return;
            }
            let mut length = 0usize;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).is_err() || line == "\r\n" || line.is_empty() {
                    break;
                }
                if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    length = v.trim().parse().unwrap_or(0);
                }
            }
            let mut body = vec![0; length];
            let _ = reader.read_exact(&mut body);
            let mut parts = request_line.split_whitespace();
            let method = parts.next().unwrap_or("");
            let cmd = parts.next().unwrap_or("/").trim_start_matches('/').to_string();
            let (status, payload) = if method == "OPTIONS" {
                ("204 No Content", String::new())
            } else {
                let args: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();
                match chess_academie_lib::dispatch(&cmd, &args) {
                    Ok(v) => ("200 OK", v.to_string()),
                    Err(e) => ("400 Bad Request", serde_json::Value::String(e).to_string()),
                }
            };
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\
                 Access-Control-Allow-Headers: content-type\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
                payload.len()
            );
            let mut out = stream;
            let _ = out.write_all(response.as_bytes());
        });
    }
}

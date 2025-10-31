//! Game save and load functionality

use crate::board::Board;
use crate::utils::Move;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const SAVE_DIR: &str = ".chess_saves";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedGame {
    pub username: String,
    pub fen: String,
    pub moves: Vec<String>,
    pub timestamp: String,
    pub white_player: String,
    pub black_player: String,
}

impl SavedGame {
    pub fn new(username: String, board: &Board, moves: &[Move], white: String, black: String) -> Self {
        SavedGame {
            username,
            fen: board.to_fen(),
            moves: moves.iter().map(|m| m.to_string()).collect(),
            timestamp: chrono_timestamp(),
            white_player: white,
            black_player: black,
        }
    }
}

// Simple timestamp function
fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    format!("{}", duration.as_secs())
}

pub struct GameManager {
    save_dir: PathBuf,
}

impl GameManager {
    pub fn new() -> Self {
        let save_dir = PathBuf::from(SAVE_DIR);
        if !save_dir.exists() {
            let _ = fs::create_dir(&save_dir);
        }
        GameManager { save_dir }
    }

    pub fn save_game(&self, game: &SavedGame) -> Result<String, String> {
        let filename = format!("{}_{}.json", game.username, game.timestamp);
        let filepath = self.save_dir.join(&filename);

        match serde_json::to_string_pretty(game) {
            Ok(json) => match fs::write(&filepath, json) {
                Ok(_) => Ok(filename),
                Err(e) => Err(format!("Failed to write file: {}", e)),
            },
            Err(e) => Err(format!("Failed to serialize: {}", e)),
        }
    }

    pub fn list_saves(&self, username: &str) -> Vec<(String, SavedGame)> {
        let mut saves = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.save_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy().to_string();
                    if filename_str.starts_with(username) && filename_str.ends_with(".json") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(game) = serde_json::from_str::<SavedGame>(&content) {
                                saves.push((filename_str, game));
                            }
                        }
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        saves.sort_by(|a, b| b.1.timestamp.cmp(&a.1.timestamp));
        saves
    }

    pub fn load_game(&self, filename: &str) -> Result<SavedGame, String> {
        let filepath = self.save_dir.join(filename);

        match fs::read_to_string(&filepath) {
            Ok(content) => match serde_json::from_str::<SavedGame>(&content) {
                Ok(game) => Ok(game),
                Err(e) => Err(format!("Failed to parse save: {}", e)),
            },
            Err(e) => Err(format!("Failed to read file: {}", e)),
        }
    }

    pub fn delete_game(&self, filename: &str) -> Result<(), String> {
        let filepath = self.save_dir.join(filename);
        fs::remove_file(&filepath).map_err(|e| format!("Failed to delete: {}", e))
    }
}

impl Default for GameManager {
    fn default() -> Self {
        Self::new()
    }
}

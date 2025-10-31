//! User authentication and profile management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const USERS_FILE: &str = ".chess_users.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    #[serde(skip_serializing)]
    password_hash: String,
    pub games_played: u32,
    pub games_won: u32,
    pub games_drawn: u32,
    pub games_lost: u32,
    pub rating: u32,
}

impl User {
    pub fn new(username: String, password: String) -> Self {
        User {
            username,
            password_hash: Self::hash_password(&password),
            games_played: 0,
            games_won: 0,
            games_drawn: 0,
            games_lost: 0,
            rating: 1200,
        }
    }

    fn hash_password(password: &str) -> String {
        // Simple hash for demonstration (in production, use bcrypt or argon2)
        format!("{:x}", md5_hash(password.as_bytes()))
    }

    pub fn verify_password(&self, password: &str) -> bool {
        self.password_hash == Self::hash_password(password)
    }

    pub fn win_rate(&self) -> f32 {
        if self.games_played == 0 {
            0.0
        } else {
            (self.games_won as f32 / self.games_played as f32) * 100.0
        }
    }
}

// Simple MD5-like hash for demonstration
fn md5_hash(data: &[u8]) -> u64 {
    let mut hash = 0x123456789ABCDEFu64;
    for &byte in data {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
}

pub struct AuthManager {
    users: HashMap<String, User>,
    users_file: PathBuf,
}

impl AuthManager {
    pub fn new() -> Self {
        let mut manager = AuthManager {
            users: HashMap::new(),
            users_file: PathBuf::from(USERS_FILE),
        };
        manager.load_users();
        manager
    }

    fn load_users(&mut self) {
        if self.users_file.exists() {
            if let Ok(content) = fs::read_to_string(&self.users_file) {
                if let Ok(users) = serde_json::from_str::<HashMap<String, User>>(&content) {
                    self.users = users;
                }
            }
        }
    }

    fn save_users(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.users) {
            let _ = fs::write(&self.users_file, json);
        }
    }

    pub fn register(&mut self, username: String, password: String) -> Result<User, String> {
        if username.is_empty() || password.is_empty() {
            return Err("Username and password cannot be empty".to_string());
        }

        if username.len() < 3 {
            return Err("Username must be at least 3 characters".to_string());
        }

        if password.len() < 4 {
            return Err("Password must be at least 4 characters".to_string());
        }

        if self.users.contains_key(&username) {
            return Err("Username already exists".to_string());
        }

        let user = User::new(username.clone(), password);
        self.users.insert(username.clone(), user.clone());
        self.save_users();
        Ok(user)
    }

    pub fn login(&self, username: &str, password: &str) -> Result<User, String> {
        if let Some(user) = self.users.get(username) {
            if user.verify_password(password) {
                Ok(user.clone())
            } else {
                Err("Invalid password".to_string())
            }
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn update_user(&mut self, user: &User) {
        self.users.insert(user.username.clone(), user.clone());
        self.save_users();
    }

    pub fn list_users(&self) -> Vec<String> {
        self.users.keys().cloned().collect()
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

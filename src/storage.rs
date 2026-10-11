use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub github_token: String,
    pub gist_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PasswordEntry {
    pub title: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Vault {
    pub entries: Vec<PasswordEntry>,
}

impl Vault {
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Failed to serialize vault")
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        serde_json::from_slice(bytes).unwrap_or_else(|_| Vault::default())
    }

    pub fn save_local<P: AsRef<Path>>(path: P, encrypted_data: &[u8]) -> io::Result<()> {
        let mut file = fs::File::create(path)?;
        file.write_all(encrypted_data)?;
        Ok(())
    }

    pub fn load_local<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    // --- NEW: Helper to save/load GitHub Config ---
    pub fn save_config(config: &Config) -> io::Result<()> {
        let data = serde_json::to_string(config)?;
        fs::write("config.json", data)
    }

    pub fn load_config() -> Config {
        fs::read_to_string("config.json")
            .map(|s| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default()
    }
}
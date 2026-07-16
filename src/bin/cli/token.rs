use std::{env, fs, io::Result, path::PathBuf};

fn token_path() -> PathBuf {
    let home = env::var("HOME").expect("Home must be set");
    PathBuf::from(home).join(".config/rust-expense-tracker/token")
}

pub fn save_token(token: &str) -> Result<()> {
    let path = token_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, token)
}

pub fn load_token() -> Option<String> {
    fs::read_to_string(token_path()).ok()
}

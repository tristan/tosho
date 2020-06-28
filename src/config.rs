use std::fs::File;
use std::io::Read;
use std::env;
use std::path::PathBuf;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub sabnzbd: SabnzbdConfig
}

#[derive(Debug, Deserialize)]
pub struct SabnzbdConfig {
    pub url: String,
    pub apikey: String
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String
}

impl Config {
    pub fn load() -> Config {
        let mut home_dir: PathBuf = env::var_os("HOME")
            .map(PathBuf::from).unwrap();
        home_dir.push(".config");
        home_dir.push("tosho");
        home_dir.push("tosho.toml");
        let mut file = File::open(&home_dir).unwrap();
        let mut toml_str = String::new();
        file.read_to_string(&mut toml_str).unwrap();
        toml::from_str(&toml_str).unwrap()
    }
}

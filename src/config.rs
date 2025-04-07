use toml;
use serde::Deserialize;
use std::fs::File as StdFile;
use std::io::Read;

pub static CONFIG: Lazy<Mutex<Option<Config>>> = Lazy::new(|| Mutex::new(None));
use std::sync::Mutex;
use once_cell::sync::Lazy;

#[derive(Deserialize)]
pub struct Config {
    pub server: ServerConfig,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub wwwroot: String,
    pub allow_public: bool
}

pub fn load_config(path: &str) -> Result<Config, toml::de::Error> {
    let mut file = StdFile::open(path).expect(format!("Failed to open config file {}", path).as_str());
    let mut content = String::new();
    file.read_to_string(&mut content).expect(format!("Failed to read config file {}", path).as_str());
    let config: Config = toml::de::from_str(content.as_str())?;
    Ok(config)
}

impl Config {
    pub fn new(path: &str) -> Self {
        match load_config(path) {
            Ok(config) => config,
            Err(_) => panic!("Cannot read configuration file at {}", path)
        }
    }
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Self { server: self.server.clone() }
    }
}

impl Clone for ServerConfig {
    fn clone(&self) -> Self {
        Self {
            port: self.port.clone(),
            wwwroot: self.wwwroot.clone(),
            allow_public: self.allow_public.clone()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self { server: Default::default() }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { port: 3000, wwwroot: ".".to_string(),allow_public: false }
    }
}

pub fn set(value: Config) {
    let mut config = CONFIG.lock().unwrap();
    *config = Some(value);
}

pub fn get() -> Option<Config>{
    let mut data = CONFIG.lock().unwrap();
    data.clone()
}

    
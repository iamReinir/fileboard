use toml;
use serde::Deserialize;
use std::fs::File as StdFile;
use std::io::Read;

//
// use std::convert::Infallible;
use http_body_util::Empty;
use std::sync::Mutex;
use once_cell::sync::Lazy;


use http_body_util::Full;
use hyper::body::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt};

//

pub static CONFIG: Lazy<Mutex<Option<Config>>> = Lazy::new(|| Mutex::new(None));

#[derive(Deserialize)]
#[derive(Default)]
pub struct Config {
    pub server: ServerConfig,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub wwwroot: String,
    pub allow_public: bool,
    pub host: String,
    pub max_file_size: usize
}

pub fn load_config(path: &str) -> Result<Config, toml::de::Error> {
    let mut file = StdFile::open(path).unwrap_or_else(|_| panic!("Failed to open config file {}", path));
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap_or_else(|_| panic!("Failed to read config file {}", path));
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
            port: self.port,
            wwwroot: self.wwwroot.clone(),
            allow_public: self.allow_public,
            host: self.host.clone(),
            max_file_size: self.max_file_size
        }
    }
}


impl Default for ServerConfig {
    fn default() -> Self {
        Self { 
            port: 3000, 
            wwwroot: ".".to_string(),
            allow_public: false, 
            host: "http://localhost:3000".to_string(),
            max_file_size: 20 * 1024 * 1024 // 20MB
        }
    }   
}

pub fn set(value: Config) {
    let mut config = CONFIG.lock().unwrap();
    *config = Some(value);
}

pub fn get() -> Option<Config>{
    let data = CONFIG.lock().unwrap();
    data.clone()
}


// We create some utility functions to make Empty and Full bodies
// fit our broadened Response body type.
pub fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
pub fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
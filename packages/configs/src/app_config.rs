use config::{Config, File};
use serde::Deserialize;
use std::convert::TryFrom;
use std::error::Error;
use std::str;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub storage_path: String,
    pub api_port: u16,
    pub log_level: String,
    pub embedding_model: String,
}

impl TryFrom<Config> for AppConfig {
    type Error = Box<dyn Error>;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        Ok(Self {
            storage_path: config.get::<String>("storage_path")?,
            api_port: config.get::<u16>("api_port")?,
            log_level: config.get::<String>("log_level")?,
            embedding_model: config.get::<String>("embedding_model")?,
        })
    }
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();
        Self {
            storage_path: std::env::var("STORAGE_PATH").expect("STORAGE_PATH must be set"),
            api_port: std::env::var("API_PORT")
                .expect("API_PORT must be set")
                .parse()
                .expect("API_PORT must be a number"),
            log_level: std::env::var("LOG_LEVEL").expect("LOG_LEVEL must be set"),
            embedding_model: std::env::var("EMBEDDING_MODEL").expect("EMBEDDING_MODEL must be set"),
        }
    }

    #[allow(deprecated)]
    pub fn from_file() -> Self {
        let mut settings = Config::default();
        settings.merge(File::with_name("config")).unwrap();
        settings.try_into().unwrap()
    }

    pub fn default() -> Self {
        Self {
            storage_path: "./data".to_string(),
            api_port: 8080,
            log_level: "info".to_string(),
            embedding_model: "./models/embedding_model.bin".to_string(),
        }
    }
}

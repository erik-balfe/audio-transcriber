use anyhow::{Context, Result};
use keyring::Entry;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub api_endpoint: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub max_file_size_bytes: usize,
    pub show_remove_api_key_button: bool,
    pub api_key: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        debug!("Entering Config::load()");
        info!("Loading configuration");
        let mut config = Self::default();
        debug!("Default configuration created");

        // Load API key from keyring
        debug!("Attempting to load API key from keyring");
        match Self::get_api_key() {
            Ok(Some(api_key)) => {
                info!("API key loaded from keyring");
                debug!("Validating loaded API key");

                // Validate the loaded API key
                if Self::is_valid_api_key(&api_key) {
                    info!("Loaded API key is valid");
                    config.api_key = Some(api_key);
                    debug!("API key successfully set in the configuration");
                } else {
                    warn!("Loaded API key is invalid");
                    config.api_key = None;
                    debug!("API key was not set in the configuration due to invalidity");
                }
            }
            Ok(None) => {
                info!("No API key found in keyring");
            }
            Err(e) => {
                error!("Failed to load API key from keyring: {}", e);
            }
        }

        debug!("Exiting Config::load()");
        Ok(config)
    }
    pub fn get_api_key() -> Result<Option<String>> {
        let entry = Entry::new("com.example.VoiceTranscriber", "api_key")?;
        match entry.get_password() {
            Ok(password) => {
                info!("Successfully retrieved API key from keyring");
                Ok(Some(password))
            }
            Err(keyring::Error::NoEntry) => {
                info!("No API key entry found in keyring");
                Ok(None)
            }
            Err(e) => {
                error!("Failed to get API key from keyring: {}", e);
                Err(anyhow::anyhow!(e)).context("Failed to get API key")
            }
        }
    }

    pub fn set_api_key(api_key: &str) -> Result<()> {
        let entry = Entry::new("com.example.VoiceTranscriber", "api_key")?;
        match entry.set_password(api_key) {
            Ok(_) => {
                info!("API key successfully saved to keyring");
                Ok(())
            }
            Err(e) => {
                error!("Failed to save API key to keyring: {}", e);
                Err(anyhow::anyhow!(e)).context("Failed to set API key")
            }
        }
    }

    pub fn remove_api_key() -> Result<()> {
        let entry = Entry::new("com.example.VoiceTranscriber", "api_key")?;
        match entry.delete_password() {
            Ok(()) => {
                info!("API key successfully removed from keyring");
                Ok(())
            }
            Err(keyring::Error::NoEntry) => {
                info!("No API key to remove from keyring");
                Ok(())
            }
            Err(e) => {
                error!("Failed to remove API key from keyring: {}", e);
                Err(anyhow::anyhow!(e)).context("Failed to remove API key")
            }
        }
    }

    pub fn max_recording_duration(&self) -> f64 {
        self.max_file_size_bytes as f64 / (self.sample_rate as f64 * self.channels as f64 * 4.0)
    }

    fn is_valid_api_key(api_key: &str) -> bool {
        // For now, we'll just check if it starts with "gsk_" and is at least 20 characters long
        // correct API key check can be made by calling the API with request for available models.
        //  It must return a 200 OK response with some JSON data if key is valid and accepted.
        let is_valid = api_key.starts_with("gsk_") && api_key.len() >= 20;
        if is_valid {
            info!("API key validation successful");
        } else {
            warn!("API key validation failed");
        }
        is_valid
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_endpoint: "https://api.groq.com/openai/v1/audio/transcriptions".to_string(),
            sample_rate: 44100,
            channels: 1,
            max_file_size_bytes: 25 * 1024 * 1024, // 25 MB
            show_remove_api_key_button: false,
            api_key: None,
        }
    }
}

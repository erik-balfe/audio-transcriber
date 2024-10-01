use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::{Result, Context};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub api_endpoint: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub max_file_size_bytes: usize,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("voice_transcriber")
            .join("config.toml");

        if config_path.exists() {
            let config_str = fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            let config: Config = toml::from_str(&config_str)
                .context("Failed to parse config file")?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("voice_transcriber");
        fs::create_dir_all(&config_path)
            .context("Failed to create config directory")?;
        let config_path = config_path.join("config.toml");

        let config_str = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(&config_path, config_str)
            .context("Failed to write config file")?;
        Ok(())
    }

    pub fn max_recording_duration(&self) -> f64 {
        let bytes_per_sample = std::mem::size_of::<f32>() * self.channels as usize;
        let bytes_per_second = self.sample_rate as usize * bytes_per_sample;
        self.max_file_size_bytes as f64 / bytes_per_second as f64
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_endpoint: "https://api.groq.com/openai/v1/audio/transcriptions".to_string(),
            sample_rate: 44100,
            channels: 1,
            max_file_size_bytes: 25 * 1024 * 1024, // 25 MB
        }
    }
}
use crate::config::Config;
use anyhow::Result;
use log::{debug, error, info, warn};
use reqwest;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct AppState {
    is_recording: bool,
    transcribed_text: String,
    api_key: Option<String>,
    audio_data: Vec<f32>,
    recording_stop_sender: Option<broadcast::Sender<()>>,
    config: Config,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            is_recording: false,
            transcribed_text: String::new(),
            api_key: config.api_key.clone(),
            audio_data: Vec::new(),
            recording_stop_sender: None,
            config,
        }
    }
}

#[derive(Clone)]
pub struct StateManager {
    state: Arc<Mutex<AppState>>,
}

impl StateManager {
    pub fn new(config: Config) -> Self {
        Self {
            state: Arc::new(Mutex::new(AppState::new(config))),
        }
    }

    // Update the get_api_key method
    pub fn get_api_key(&self) -> Option<String> {
        self.state.lock().unwrap().api_key.clone()
    }

    // Update the set_api_key method
    pub fn set_api_key(&self, key: Option<String>) {
        self.state.lock().unwrap().api_key = key;
    }

    pub fn is_recording(&self) -> bool {
        self.state.lock().unwrap().is_recording
    }

    pub fn set_recording(&self, is_recording: bool) {
        self.state.lock().unwrap().is_recording = is_recording;
    }

    pub fn get_transcribed_text(&self) -> String {
        self.state.lock().unwrap().transcribed_text.clone()
    }

    pub fn set_transcribed_text(&self, text: String) {
        self.state.lock().unwrap().transcribed_text = text;
    }

    pub fn get_audio_data(&self) -> Vec<f32> {
        self.state.lock().unwrap().audio_data.clone()
    }

    pub fn set_audio_data(&self, data: Vec<f32>) {
        self.state.lock().unwrap().audio_data = data;
    }

    pub fn clear_audio_data(&self) {
        self.state.lock().unwrap().audio_data.clear();
    }

    pub fn start_recording(&self) -> broadcast::Receiver<()> {
        let mut state = self.state.lock().unwrap();
        let (tx, rx) = broadcast::channel(1);
        state.recording_stop_sender = Some(tx);
        state.is_recording = true;
        rx
    }

    pub fn stop_recording(&self) {
        let mut state = self.state.lock().unwrap();
        if let Some(sender) = state.recording_stop_sender.take() {
            let _ = sender.send(());
        }
        state.is_recording = false;
    }

    pub fn get_config(&self) -> Config {
        self.state.lock().unwrap().config.clone()
    }

    pub fn is_recording_sync(&self) -> bool {
        self.state.lock().unwrap().is_recording
    }
    pub async fn validate_and_save_api_key(&self, api_key: &str) -> Result<bool> {
        debug!("Attempting to validate and save API key");
        let is_valid = api_key.starts_with("gsk_") && api_key.len() >= 20;

        if is_valid {
            debug!("API key format is valid, attempting to validate with Groq API");
            match self.validate_api_key_with_groq(api_key).await {
                Ok(true) => {
                    info!("API key validated successfully with Groq");
                    Config::set_api_key(api_key)?;
                    let mut state = self.state.lock().unwrap();
                    state.api_key = Some(api_key.to_string());
                    state.config.api_key = Some(api_key.to_string());
                    info!("API key saved successfully");
                    Ok(true)
                }
                Ok(false) => {
                    warn!("API key format is valid but rejected by Groq API");
                    Ok(false)
                }
                Err(e) => {
                    error!("Error validating API key with Groq: {:?}", e);
                    Ok(false)
                }
            }
        } else {
            error!("Invalid API key format");
            Ok(false)
        }
    }

    async fn validate_api_key_with_groq(&self, api_key: &str) -> Result<bool> {
        debug!("Sending request to Groq API to validate key");
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.groq.com/openai/v1/models")
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;

        let status = response.status();
        debug!("Received response from Groq API with status: {}", status);
        Ok(status.is_success())
    }

    // Add this method to the StateManager implementation
    pub fn append_audio_data(&self, data: &[f32]) {
        let mut state = self.state.lock().unwrap();
        state.audio_data.extend_from_slice(data);
    }

    pub fn has_api_key(&self) -> bool {
        Config::get_api_key()
            .map(|key| key.is_some())
            .unwrap_or(false)
    }

    pub fn remove_api_key(&self) -> Result<()> {
        Config::remove_api_key()?;
        let mut state = self.state.lock().unwrap();
        state.api_key = None;
        state.config.api_key = None;
        Ok(())
    }
}

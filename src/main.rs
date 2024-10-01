mod ui;
mod audio;
mod config;
mod state;

use adw::prelude::*;
use anyhow::Result;
use crate::state::StateManager;
use crate::config::Config;
use log::info;
use gstreamer as gst;

const APP_ID: &str = "com.example.VoiceTranscriber";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting Voice Transcriber application");

    // Initialize GStreamer
    gst::init()?;
    info!("GStreamer initialized");

    let config = Config::load()?;
    info!("Loaded configuration: {:?}", config);

    let app = adw::Application::builder().application_id(APP_ID).build();
    let state_manager = StateManager::new(config);

    app.connect_startup(|_| {
        info!("Application startup");
        adw::init().expect("Failed to initialize libadwaita");
    });

    // Build the UI
    ui::build_ui(&app, state_manager).await?;

    app.run();

    Ok(())
}
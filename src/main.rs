mod audio;
mod config;
mod state;
mod ui;

use crate::config::Config;
use crate::state::StateManager;
use crate::ui::builder::build_ui;
use anyhow::Result;
use gstreamer as gst;
use gtk::prelude::*;
use gtk::{self, Application};
use log::{debug, info, LevelFilter};
use std::sync::Arc;

const APP_ID: &str = "com.example.VoiceTranscriber";

#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Debug)
        .init();

    info!("Starting Voice Transcriber application");
    debug!("Debug logging enabled");

    // Initialize GStreamer
    gst::init()?;
    info!("GStreamer initialized");

    debug!("Loading configuration...");
    let config = Config::load()?;
    info!("Loaded configuration: {:?}", config);

    debug!(
        "API Key status: {}",
        if config.api_key.is_some() {
            "Present"
        } else {
            "Not found"
        }
    );

    let app = Application::builder().application_id(APP_ID).build();
    let state_manager = Arc::new(StateManager::new(config));

    app.connect_startup(|_| {
        info!("Application startup");
        adw::init().expect("Failed to initialize libadwaita");
    });

    let state_manager_clone = Arc::clone(&state_manager);
    app.connect_activate(move |app| {
        build_ui(app, &state_manager_clone);
    });

    info!("Running the application");
    app.run();

    Ok(())
}

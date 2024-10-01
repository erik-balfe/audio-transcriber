use adw::prelude::*;
use anyhow::Result;
use crate::state::StateManager;
use super::components::MainWindowBuilder;
use log::{info, error};
use gtk::glib;
use std::rc::Rc;

pub async fn build_ui(app: &adw::Application, state_manager: StateManager) -> Result<()> {
    // Create the window builder and wrap it in an Rc
    let window_builder = Rc::new(MainWindowBuilder::new(app, state_manager.clone()));
    
    // Connect to the activate signal
    let window_builder_clone = window_builder.clone();
    let state_manager_clone = state_manager.clone();
    app.connect_activate(move |_app| {
        // Build the window when the application is activated
        let window = window_builder_clone.build();
        
        // Add API key validation button
        let header_bar = window.content().and_then(|content| content.first_child())
            .and_downcast::<adw::HeaderBar>()
            .expect("Failed to get HeaderBar");
        
        let validate_button = gtk::Button::with_label("Validate API Key");
        header_bar.pack_start(&validate_button);

        let state_manager = state_manager_clone.clone();
        validate_button.connect_clicked(move |_| {
            let state_manager = state_manager.clone();
            glib::MainContext::default().spawn_local(async move {
                match state_manager.validate_api_key().await {
                    Ok(is_valid) => {
                        if is_valid {
                            info!("API key is valid");
                            // Enable recording button or other UI elements
                        } else {
                            error!("API key is invalid");
                            // Show error message to user
                        }
                    },
                    Err(e) => error!("Error validating API key: {:?}", e),
                }
            });
        });

        // Present the window
        window.present();
    });

    Ok(())
}
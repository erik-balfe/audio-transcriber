use crate::audio::{play_audio, record_audio};
use crate::state::StateManager;
use adw::prelude::*;
use gtk::glib::clone;
use gtk::{self, glib};
use log::{debug, error, info, warn};
use std::sync::Arc;

pub fn build_ui(app: &gtk::Application, state_manager: &Arc<StateManager>) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Voice Transcriber")
        .default_width(400)
        .default_height(300)
        .build();

    let header_bar = adw::HeaderBar::new();
    let content = build_content(state_manager);

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    main_box.append(&header_bar);
    main_box.append(&content);

    window.set_content(Some(&main_box));
    window.present();
}

fn build_content(state_manager: &Arc<StateManager>) -> gtk::Box {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let api_key_section = build_api_key_section(state_manager);
    content.append(&api_key_section);

    let menu = build_menu(state_manager, &api_key_section);
    content.append(&menu);

    let (button_box, text_view, error_label) = build_button_box(state_manager);
    content.append(&button_box);

    let scrolled_window = gtk::ScrolledWindow::new();
    scrolled_window.set_child(Some(&text_view));
    scrolled_window.set_vexpand(true);
    content.append(&scrolled_window);

    content.append(&error_label);

    let max_duration_label = build_max_duration_label(state_manager);
    content.append(&max_duration_label);

    content
}

fn build_api_key_section(state_manager: &Arc<StateManager>) -> gtk::Box {
    let api_key_box = gtk::Box::new(gtk::Orientation::Vertical, 6);

    let api_key_entry = gtk::PasswordEntry::new();
    api_key_entry.set_placeholder_text(Some("Enter Groq API Key"));
    api_key_entry.set_show_peek_icon(true);

    let save_button = gtk::Button::with_label("Save API Key");
    save_button.set_sensitive(false);

    let error_label = gtk::Label::new(None);
    error_label.set_markup("<span color=\"red\"></span>");
    error_label.set_visible(false);

    api_key_box.append(&api_key_entry);
    api_key_box.append(&save_button);
    api_key_box.append(&error_label);

    let state_manager_clone = Arc::clone(state_manager);
    api_key_entry.connect_changed(clone!(@weak save_button => move |entry| {
        save_button.set_sensitive(!entry.text().is_empty());
    }));

    let save_api_key = clone!(@weak api_key_entry, @weak error_label, @weak api_key_box, @strong state_manager_clone => move || {
        let api_key = api_key_entry.text().to_string();
        glib::MainContext::default().spawn_local(clone!(@weak error_label, @weak api_key_box, @strong state_manager_clone => async move {
            match state_manager_clone.validate_and_save_api_key(&api_key).await {
                Ok(true) => {
                    api_key_box.set_visible(false);
                    // Enable main app functionality here
                },
                Ok(false) => {
                    error_label.set_markup("<span color=\"red\">Invalid API key. Please try again.</span>");
                    error_label.set_visible(true);
                },
                Err(e) => {
                    error_label.set_markup(&format!("<span color=\"red\">Error: {}</span>", e));
                    error_label.set_visible(true);
                },
            }
        }));
    });

    save_button.connect_clicked(clone!(@strong save_api_key => move |_| {
        save_api_key();
    }));

    api_key_entry.connect_activate(clone!(@strong save_api_key => move |_| {
        save_api_key();
    }));

    if state_manager.has_api_key() {
        api_key_box.set_visible(false);
    }

    api_key_box
}

fn build_menu(state_manager: &Arc<StateManager>, api_key_box: &gtk::Box) -> gtk::Box {
    let menu_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);

    let remove_api_key_button = gtk::Button::with_label("Remove API Key");
    menu_box.append(&remove_api_key_button);

    let state_manager_clone = Arc::clone(state_manager);
    let api_key_box_clone = api_key_box.clone();

    // Get the current configuration
    let config = state_manager.get_config();

    // Set the visibility of the remove button based on the presence of an API key
    remove_api_key_button.set_visible(config.api_key.is_some());

    remove_api_key_button.connect_clicked(
        clone!(@weak api_key_box_clone, @weak remove_api_key_button, @strong state_manager_clone => move |_| {
            if let Err(e) = state_manager_clone.remove_api_key() {
                error!("Failed to remove API key: {}", e);
                // Show error message to user
            } else {
                api_key_box_clone.set_visible(true);
                // Hide the remove button after removing the API key
                remove_api_key_button.set_visible(false);
                // Disable main app functionality here
            }
        }),
    );

    menu_box
}

fn build_button_box(state_manager: &Arc<StateManager>) -> (gtk::Box, gtk::TextView, gtk::Label) {
    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);

    let record_button = gtk::Button::with_label("Start Recording");
    let transcribe_button = gtk::Button::with_label("Transcribe");
    let copy_button = gtk::Button::with_label("Copy to Clipboard");
    let play_button = gtk::Button::with_label("Play Recording");
    let reset_button = gtk::Button::with_label("Reset");

    button_box.append(&record_button);
    button_box.append(&transcribe_button);
    button_box.append(&copy_button);
    button_box.append(&play_button);
    button_box.append(&reset_button);

    let text_view = build_text_view();
    let error_label = gtk::Label::new(None);
    error_label.set_markup("<span color=\"red\"></span>");
    error_label.set_visible(false);

    setup_button_handlers(
        Arc::clone(state_manager),
        record_button,
        transcribe_button,
        copy_button,
        play_button,
        reset_button,
        text_view.clone(),
        error_label.clone(),
    );

    (button_box, text_view, error_label)
}

fn build_text_view() -> gtk::TextView {
    let text_view = gtk::TextView::new();
    text_view.set_wrap_mode(gtk::WrapMode::Word);
    text_view.set_editable(false);
    text_view
}

fn build_max_duration_label(state_manager: &Arc<StateManager>) -> gtk::Label {
    let label = gtk::Label::new(None);
    let label_clone = label.clone();
    let state_manager_clone = Arc::clone(state_manager);
    glib::MainContext::default().spawn_local(async move {
        let config = state_manager_clone.get_config();
        let max_duration = config.max_recording_duration();
        label_clone.set_text(&format!(
            "Maximum recording duration: {:.2} seconds",
            max_duration
        ));
    });
    label
}

fn setup_button_handlers(
    state_manager: Arc<StateManager>,
    record_button: gtk::Button,
    transcribe_button: gtk::Button,
    copy_button: gtk::Button,
    play_button: gtk::Button,
    reset_button: gtk::Button,
    text_view: gtk::TextView,
    error_label: gtk::Label,
) {
    transcribe_button.connect_clicked(clone!(@strong state_manager, @strong text_view, @strong error_label => move |_| {
        glib::MainContext::default().spawn_local(clone!(@strong state_manager, @strong text_view, @strong error_label => async move {
            match state_manager.transcribe_audio().await {
                Ok(transcription) => {
                    info!("Transcription successful: {}", transcription);
                    state_manager.set_transcribed_text(transcription.clone());
                    text_view.buffer().set_text(&transcription);
                }
                Err(e) => {
                    error!("Error during transcription: {:?}", e);
                    error_label.set_text(&format!("Transcription error: {}", e));
                    error_label.set_visible(true);
                }
            }
        }));
    }));

    record_button.connect_clicked(clone!(@strong state_manager, @strong transcribe_button, @strong play_button => move |button| {
        glib::MainContext::default().spawn_local(clone!(@strong state_manager, @strong button, @strong transcribe_button, @strong play_button => async move {
            if !state_manager.is_recording() {
                state_manager.set_recording(true);
                button.set_label("Stop Recording");
                state_manager.clear_audio_data();
                transcribe_button.set_sensitive(false);
                play_button.set_sensitive(false);

                // Spawn the recording task in a separate Tokio task
                tokio::spawn(clone!(@strong state_manager => async move {
                    if let Err(e) = record_audio(state_manager).await {
                        error!("Error during recording: {}", e);
                    }
                }));
            } else {
                state_manager.stop_recording();
                button.set_label("Start Recording");
                transcribe_button.set_sensitive(true);
                play_button.set_sensitive(true);

                let audio_data = state_manager.get_audio_data();
                info!("Recorded audio data length: {} samples", audio_data.len());
                debug!(
                    "First 10 samples: {:?}",
                    &audio_data[..10.min(audio_data.len())]
                );
                debug!(
                    "Last 10 samples: {:?}",
                    &audio_data[audio_data.len().saturating_sub(10)..]
                );
            }
        }));
    }));

    play_button.connect_clicked(clone!(@strong state_manager => move |_| {
        glib::MainContext::default().spawn_local(clone!(@strong state_manager => async move {
            let audio_data = state_manager.get_audio_data();
            if audio_data.is_empty() {
                warn!("Error: No audio data to play");
                return;
            }
            info!("Attempting to play {} samples", audio_data.len());
            match play_audio(audio_data) {
                Ok(_) => info!("Audio playback completed successfully"),
                Err(e) => error!("Error playing audio: {}", e),
            }
        }));
    }));

    copy_button.connect_clicked(clone!(@strong state_manager => move |_| {
        glib::MainContext::default().spawn_local(clone!(@strong state_manager => async move {
            let transcribed_text = state_manager.get_transcribed_text();
            if !transcribed_text.is_empty() {
                // TODO: Implement clipboard functionality
                info!("Copied to clipboard: {}", transcribed_text);
            } else {
                warn!("No transcribed text to copy");
            }
        }));
    }));

    reset_button.connect_clicked(clone!(@strong state_manager, @strong record_button, @strong transcribe_button, @strong copy_button, @strong play_button, @strong text_view, @strong error_label => move |_| {
        glib::MainContext::default().spawn_local(clone!(@strong state_manager, @strong record_button, @strong transcribe_button, @strong copy_button, @strong play_button, @strong text_view, @strong error_label => async move {
            // Reset state
            state_manager.clear_audio_data();
            state_manager.set_transcribed_text(String::new());
            if state_manager.is_recording() {
                state_manager.stop_recording();
            }

            // Reset UI elements
            record_button.set_label("Start Recording");
            record_button.set_sensitive(true);
            transcribe_button.set_sensitive(false);
            copy_button.set_sensitive(false);
            play_button.set_sensitive(false);
            text_view.buffer().set_text("");
            error_label.set_text("");
            error_label.set_visible(false);

            info!("Reset completed");
        }));
    }));
}

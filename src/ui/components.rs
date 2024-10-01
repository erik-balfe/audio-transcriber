use crate::audio::{play_audio, record_audio};
use crate::state::StateManager;
use adw::prelude::*;
use gtk::{self, glib};
use log::{debug, error, info, warn};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone)]
pub struct MainWindowBuilder {
    app: Rc<adw::Application>,
    state_manager: Arc<StateManager>,
}

impl MainWindowBuilder {
    pub fn new(app: Rc<adw::Application>, state_manager: Arc<StateManager>) -> Self {
        Self { app, state_manager }
    }

    pub fn build(&self) -> adw::Window {
        let window = adw::Window::builder()
            .application(&*self.app)
            .title("Voice Transcriber")
            .default_width(400)
            .default_height(300)
            .build();

        let header_bar = self.build_header_bar();
        let content = self.build_content();

        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        main_box.append(&header_bar);
        main_box.append(&content);

        window.set_content(Some(&main_box));

        window
    }

    fn build_header_bar(&self) -> adw::HeaderBar {
        adw::HeaderBar::new()
    }

    fn build_content(&self) -> gtk::Box {
        let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        let api_key_section = self.build_api_key_section();
        content.append(&api_key_section);

        let menu = self.build_menu();
        content.append(&menu);

        let button_box = self.build_button_box();
        content.append(&button_box);

        let text_view = self.build_text_view();
        let scrolled_window = gtk::ScrolledWindow::new();
        scrolled_window.set_child(Some(&text_view));
        scrolled_window.set_vexpand(true);
        content.append(&scrolled_window);

        let max_duration_label = self.build_max_duration_label();
        content.append(&max_duration_label);

        content
    }

    fn build_api_key_section(&self) -> gtk::Box {
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

        let state_manager = Arc::clone(&self.state_manager);
        api_key_entry.connect_changed(glib::clone!(@weak save_button => move |entry| {
            save_button.set_sensitive(!entry.text().is_empty());
        }));

        let save_api_key = glib::clone!(@weak api_key_entry, @weak error_label, @weak api_key_box, @strong state_manager => move || {
            let api_key = api_key_entry.text().to_string();
            glib::MainContext::default().spawn_local(glib::clone!(@weak error_label, @weak api_key_box, @strong state_manager => async move {
                match state_manager.validate_and_save_api_key(&api_key).await {
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

        save_button.connect_clicked(glib::clone!(@strong save_api_key => move |_| {
            save_api_key();
        }));

        api_key_entry.connect_activate(glib::clone!(@strong save_api_key => move |_| {
            save_api_key();
        }));

        if self.state_manager.has_api_key() {
            api_key_box.set_visible(false);
        }

        api_key_box
    }

    fn build_menu(&self) -> gtk::Box {
        let menu_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);

        let change_api_key_button = gtk::Button::with_label("Change API Key");
        let remove_api_key_button = gtk::Button::with_label("Remove API Key");

        menu_box.append(&change_api_key_button);
        menu_box.append(&remove_api_key_button);

        let state_manager = Arc::clone(&self.state_manager);
        let api_key_box = self.build_api_key_section();

        change_api_key_button.connect_clicked(glib::clone!(@weak api_key_box => move |_| {
            api_key_box.set_visible(true);
        }));

        remove_api_key_button.connect_clicked(
            glib::clone!(@weak api_key_box, @strong state_manager => move |_| {
                if let Err(e) = state_manager.remove_api_key() {
                    error!("Failed to remove API key: {}", e);
                    // Show error message to user
                } else {
                    api_key_box.set_visible(true);
                    // Disable main app functionality here
                }
            }),
        );

        menu_box
    }

    fn build_button_box(&self) -> gtk::Box {
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

        self.setup_button_handlers(
            &record_button,
            &transcribe_button,
            &copy_button,
            &play_button,
            &reset_button,
        );

        button_box
    }

    fn build_text_view(&self) -> gtk::TextView {
        let text_view = gtk::TextView::new();
        text_view.set_wrap_mode(gtk::WrapMode::Word);
        text_view.set_editable(false);
        text_view
    }

    fn build_max_duration_label(&self) -> gtk::Label {
        let label = gtk::Label::new(None);
        let label_clone = label.clone();
        let state_manager = self.state_manager.clone();
        glib::MainContext::default().spawn_local(async move {
            let config = state_manager.get_config();
            let max_duration = config.max_recording_duration();
            label_clone.set_text(&format!(
                "Maximum recording duration: {:.2} seconds",
                max_duration
            ));
        });
        label
    }

    fn setup_button_handlers(
        &self,
        record_button: &gtk::Button,
        transcribe_button: &gtk::Button,
        copy_button: &gtk::Button,
        play_button: &gtk::Button,
        reset_button: &gtk::Button,
    ) {
        let state_manager = self.state_manager.clone();
        let transcribe_button_clone = transcribe_button.clone();
        let play_button_clone = play_button.clone();

        record_button.connect_clicked(move |button| {
            let state_manager = state_manager.clone();
            let button = button.clone();
            let transcribe_button = transcribe_button_clone.clone();
            let play_button = play_button_clone.clone();
            glib::MainContext::default().spawn_local(async move {
                if !state_manager.is_recording() {
                    state_manager.set_recording(true);
                    button.set_label("Stop Recording");
                    state_manager.clear_audio_data();
                    transcribe_button.set_sensitive(false);
                    play_button.set_sensitive(false);

                    // Spawn the recording task in a separate Tokio task
                    tokio::spawn(async move {
                        if let Err(e) = record_audio(Arc::clone(&state_manager)).await {
                            error!("Error during recording: {}", e);
                        }
                    });
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
            });
        });

        let state_manager = self.state_manager.clone();
        play_button.connect_clicked(move |_| {
            let state_manager = state_manager.clone();
            glib::MainContext::default().spawn_local(async move {
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
            });
        });

        let state_manager = self.state_manager.clone();
        transcribe_button.connect_clicked(move |_| {
            let state_manager = state_manager.clone();
            glib::MainContext::default().spawn_local(async move {
                let audio_data = state_manager.get_audio_data();
                if audio_data.is_empty() {
                    warn!("No audio data to transcribe");
                    return;
                }

                // TODO: Implement actual transcription logic here
                // For now, let's just set some dummy text
                let dummy_transcription = "This is a dummy transcription.".to_string();
                state_manager.set_transcribed_text(dummy_transcription);

                info!("Transcription completed");
            });
        });

        let state_manager = self.state_manager.clone();
        copy_button.connect_clicked(move |_| {
            let state_manager = state_manager.clone();
            glib::MainContext::default().spawn_local(async move {
                let transcribed_text = state_manager.get_transcribed_text();
                if !transcribed_text.is_empty() {
                    // TODO: Implement clipboard functionality
                    info!("Copied to clipboard: {}", transcribed_text);
                } else {
                    warn!("No transcribed text to copy");
                }
            });
        });

        let state_manager = self.state_manager.clone();
        reset_button.connect_clicked(move |_| {
            let state_manager = state_manager.clone();
            glib::MainContext::default().spawn_local(async move {
                state_manager.clear_audio_data();
                state_manager.set_transcribed_text(String::new());
                // TODO: Reset UI elements (e.g., clear text view, reset button states)
                info!("Reset button clicked");
            });
        });
    }
}

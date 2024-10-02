use crate::audio::{play_audio, record_audio};
use crate::config::Config;
use crate::state::{AppStateEnum, StateManager};
use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, RelmApp, RelmWidgetExt};
use std::sync::Arc;

mod audio;
mod config;
mod state;

struct AppModel {
    state_manager: Arc<StateManager>,
}

#[derive(Debug)]
enum AppMsg {
    Record,
    Play,
    Transcribe,
    Reset,
    UpdateState(AppStateEnum),
    SetApiKey(String),
    RemoveApiKey,
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = Arc<StateManager>;
    type Input = AppMsg;
    type Output = ();

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Voice Transcriber"),
            set_default_width: 400,
            set_default_height: 300,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 6,
                set_margin_all: 12,

                #[name = "api_key_entry"]
                gtk::PasswordEntry {
                    set_placeholder_text: Some("Enter Groq API Key"),
                    set_show_peek_icon: true,
                },

                #[name = "save_api_key_button"]
                gtk::Button {
                    set_label: "Save API Key",
                    connect_clicked[sender, api_key_entry] => move |_| {
                        let api_key = api_key_entry.text().to_string();
                        sender.input(AppMsg::SetApiKey(api_key));
                    },
                },

                #[name = "remove_api_key_button"]
                gtk::Button {
                    set_label: "Remove API Key",
                    connect_clicked[sender] => move |_| {
                        sender.input(AppMsg::RemoveApiKey);
                    },
                },

                #[name = "record_button"]
                gtk::Button {
                    set_label: "Start Recording",
                    connect_clicked[sender] => move |_| {
                        sender.input(AppMsg::Record);
                    },
                },

                #[name = "transcribe_button"]
                gtk::Button {
                    set_label: "Transcribe",
                    connect_clicked[sender] => move |_| {
                        sender.input(AppMsg::Transcribe);
                    },
                },

                #[name = "play_button"]
                gtk::Button {
                    set_label: "Play Recording",
                    connect_clicked[sender] => move |_| {
                        sender.input(AppMsg::Play);
                    },
                },

                #[name = "reset_button"]
                gtk::Button {
                    set_label: "Reset",
                    connect_clicked[sender] => move |_| {
                        sender.input(AppMsg::Reset);
                    },
                },

                #[name = "text_view"]
                gtk::TextView {
                    set_editable: false,
                    set_wrap_mode: gtk::WrapMode::Word,
                },

                #[name = "error_label"]
                gtk::Label {
                    set_markup: "<span color=\"red\"></span>",
                    set_visible: false,
                }
            }
        }
    }

    fn init(
        state_manager: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AppModel { state_manager };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: AppMsg, sender: ComponentSender<Self>) {
        match msg {
            AppMsg::Record => {
                if !self.state_manager.is_recording() {
                    self.state_manager.set_app_state(AppStateEnum::Recording);
                    sender.input(AppMsg::UpdateState(AppStateEnum::Recording));
                    let state_manager = Arc::clone(&self.state_manager);
                    let sender_clone = sender.clone();
                    tokio::spawn(async move {
                        if let Err(e) = record_audio(state_manager).await {
                            eprintln!("Error during recording: {}", e);
                        }
                        sender_clone.input(AppMsg::UpdateState(AppStateEnum::Recorded));
                    });
                } else {
                    self.state_manager.stop_recording();
                    sender.input(AppMsg::UpdateState(AppStateEnum::Recorded));
                }
            }
            AppMsg::Play => {
                if self.state_manager.get_app_state() != AppStateEnum::Playing {
                    self.state_manager.set_app_state(AppStateEnum::Playing);
                    sender.input(AppMsg::UpdateState(AppStateEnum::Playing));
                    let state_manager = Arc::clone(&self.state_manager);
                    let sender_clone = sender.clone();
                    tokio::spawn(async move {
                        if let Err(e) = play_audio(state_manager).await {
                            eprintln!("Error playing audio: {}", e);
                        }
                        sender_clone.input(AppMsg::UpdateState(AppStateEnum::Recorded));
                    });
                } else {
                    self.state_manager.stop_playing();
                    sender.input(AppMsg::UpdateState(AppStateEnum::Recorded));
                }
            }
            AppMsg::Transcribe => {
                let state_manager = Arc::clone(&self.state_manager);
                let sender_clone = sender.clone();
                tokio::spawn(async move {
                    match state_manager.transcribe_audio().await {
                        Ok(transcription) => {
                            state_manager.set_transcribed_text(transcription);
                            sender_clone.input(AppMsg::UpdateState(AppStateEnum::Transcribed));
                        }
                        Err(e) => {
                            eprintln!("Transcription error: {}", e);
                            // TODO: Update error label
                        }
                    }
                });
            }
            AppMsg::Reset => {
                self.state_manager.clear_audio_data();
                self.state_manager.set_transcribed_text(String::new());
                self.state_manager.set_app_state(AppStateEnum::Initial);
                sender.input(AppMsg::UpdateState(AppStateEnum::Initial));
            }
            AppMsg::UpdateState(_state) => {
                // TODO: Implement UI update logic
            }
            AppMsg::SetApiKey(api_key) => {
                let state_manager = Arc::clone(&self.state_manager);
                let sender_clone = sender.clone();
                tokio::spawn(async move {
                    match state_manager.validate_and_save_api_key(&api_key).await {
                        Ok(true) => {
                            sender_clone.input(AppMsg::UpdateState(AppStateEnum::Initial));
                        }
                        Ok(false) => {
                            // TODO: Update error label
                        }
                        Err(e) => {
                            eprintln!("Error saving API key: {}", e);
                            // TODO: Update error label
                        }
                    }
                });
            }
            AppMsg::RemoveApiKey => {
                if let Err(e) = self.state_manager.remove_api_key() {
                    eprintln!("Error removing API key: {}", e);
                    // TODO: Update error label
                }
            }
        }
    }
}

fn main() {
    let config = Config::load().expect("Failed to load config");
    let state_manager = Arc::new(StateManager::new(config));

    let app = RelmApp::new("com.example.VoiceTranscriber");
    app.run::<AppModel>(state_manager);
}

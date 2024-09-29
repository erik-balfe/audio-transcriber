use adw::prelude::*;
use adw::{Application, ApplicationWindow};
use std::sync::Arc;
use tokio::sync::Mutex;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use anyhow::Result;
use serde_json::Value;
use glib;

const APP_ID: &str = "com.example.VoiceTranscriber";

struct AppState {
    is_recording: bool,
    transcribed_text: String,
    api_key: String,
    audio_data: Vec<f32>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
    Ok(())
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Voice Transcriber")
        .default_width(400)
        .default_height(300)
        .build();

    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let api_key_entry = gtk::Entry::new();
    api_key_entry.set_placeholder_text(Some("Enter Groq API Key"));
    content.append(&api_key_entry);

    let text_view = gtk::TextView::new();
    text_view.set_wrap_mode(gtk::WrapMode::Word);
    text_view.set_editable(false);
    let scrolled_window = gtk::ScrolledWindow::new();
    scrolled_window.set_child(Some(&text_view));
    scrolled_window.set_vexpand(true);
    content.append(&scrolled_window);

    let record_button = gtk::Button::with_label("Start Recording");
    let transcribe_button = gtk::Button::with_label("Transcribe");
    let copy_button = gtk::Button::with_label("Copy to Clipboard");

    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    button_box.append(&record_button);
    button_box.append(&transcribe_button);
    button_box.append(&copy_button);
    content.append(&button_box);

    window.set_content(Some(&content));

    let app_state = Arc::new(Mutex::new(AppState {
        is_recording: false,
        transcribed_text: String::new(),
        api_key: String::new(),
        audio_data: Vec::new(),
    }));

    let app_state_clone = app_state.clone();
    let text_buffer = text_view.buffer();
    record_button.connect_clicked(move |button| {
        let app_state = app_state_clone.clone();
        tokio::spawn(async move {
            let mut state = app_state.lock().await;
            state.is_recording = !state.is_recording;
            if state.is_recording {
                button.set_label("Stop Recording");
                state.audio_data.clear();
                record_audio(app_state.clone()).await.unwrap();
            } else {
                button.set_label("Start Recording");
                println!("Recording stopped, audio data length: {}", state.audio_data.len());
            }
        });
    });

    let app_state_clone = app_state.clone();
    let api_key_entry_clone = api_key_entry.clone();
    let text_buffer_clone = text_buffer.clone();
    transcribe_button.connect_clicked(move |_| {
        let app_state = app_state_clone.clone();
        let api_key = api_key_entry_clone.text().to_string();
        let text_buffer = text_buffer_clone.clone();
        tokio::spawn(async move {
            let mut state = app_state.lock().await;
            state.api_key = api_key;
            let result = transcribe_audio(app_state.clone()).await;
            let text = match result {
                Ok(transcription) => transcription,
                Err(e) => format!("Error: {}", e),
            };
            state.transcribed_text = text.clone();
            glib::idle_add_local_once(move || {
                text_buffer.set_text(&text);
            });
        });
    });

    let app_state_clone = app_state.clone();
    let window_clone = window.clone();
    copy_button.connect_clicked(move |_| {
        let state = app_state_clone.blocking_lock();
        let clipboard = window_clone.clipboard();
        clipboard.set_text(&state.transcribed_text);
        println!("Copied to clipboard: {}", state.transcribed_text);
    });

    window.present();
}

async fn record_audio(app_state: Arc<Mutex<AppState>>) -> Result<()> {
    let host = cpal::default_host();
    let device = host.default_input_device().expect("No input device available");
    let config = device.default_input_config().unwrap();

    let app_state_clone = app_state.clone();
    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut state = app_state_clone.blocking_lock();
            if state.is_recording {
                state.audio_data.extend_from_slice(data);
            }
        },
        |err| eprintln!("An error occurred on the input audio stream: {}", err),
        None,
    )?;

    stream.play()?;

    while app_state.lock().await.is_recording {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    Ok(())
}

async fn transcribe_audio(app_state: Arc<Mutex<AppState>>) -> Result<String> {
    let state = app_state.lock().await;
    let api_key = state.api_key.clone();
    let audio_data = state.audio_data.clone();
    drop(state);

    // Save audio data to a temporary WAV file
    let temp_file = tempfile::NamedTempFile::new()?;
    let path = temp_file.path().to_str().unwrap();
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(path, spec)?;
    for sample in audio_data {
        writer.write_sample(sample)?;
    }
    writer.finalize()?;

    // Send the audio file to the Groq API
    let client = reqwest::Client::new();
    let form = reqwest::multipart::Form::new()
        .file("file", path)?
        .text("model", "distil-whisper-large-v3-en")
        .text("response_format", "json");

    let response = client
        .post("https://api.groq.com/openai/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await?;

    if response.status().is_success() {
        let json: Value = response.json().await?;
        Ok(json["text"].as_str().unwrap_or("").to_string())
    } else {
        Err(anyhow::anyhow!("API request failed: {}", response.status()))
    }
}
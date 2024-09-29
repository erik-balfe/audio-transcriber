use adw::prelude::*;
use adw::{Application, Window, HeaderBar};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use anyhow::Result;
use serde_json::Value;
use glib;
use gtk::glib::clone;
use ogg::writing::PacketWriter;
use vorbis::Encoder;
use rodio::{OutputStream, Sink};
use std::io::Cursor;
use ogg::PacketWriteEndInfo;

const APP_ID: &str = "com.example.VoiceTranscriber";
const SAMPLE_RATE: u32 = 16000;

struct AppState {
    is_recording: bool,
    transcribed_text: String,
    api_key: String,
    audio_data: Vec<f32>,
    recording_stop_sender: Option<mpsc::Sender<()>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
    Ok(())
}

fn build_ui(app: &Application) {
    let window = Window::builder()
        .application(app)
        .title("Voice Transcriber")
        .default_width(400)
        .default_height(300)
        .build();

    let header_bar = HeaderBar::new();
    
    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let api_key_entry = gtk::Entry::new();
    api_key_entry.set_placeholder_text(Some("Enter Groq API Key"));
    content.append(&api_key_entry);

    let api_key_error_label = gtk::Label::new(None);
    api_key_error_label.set_markup("<span color=\"red\"></span>");
    api_key_error_label.set_visible(false);
    content.append(&api_key_error_label);

    let validate_button = gtk::Button::with_label("Validate API Key");
    content.append(&validate_button);

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

    let play_button = gtk::Button::with_label("Play Recording");
    button_box.append(&play_button);
    play_button.set_sensitive(false);

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    main_box.append(&header_bar);
    main_box.append(&content);

    window.set_content(Some(&main_box));

    let app_state = Arc::new(Mutex::new(AppState {
        is_recording: false,
        transcribed_text: String::new(),
        api_key: String::new(),
        audio_data: Vec::new(),
        recording_stop_sender: None,
    }));

    // Initially disable all buttons except the validate button
    record_button.set_sensitive(false);
    transcribe_button.set_sensitive(false);
    copy_button.set_sensitive(false);

    let app_state_clone = app_state.clone();
    let api_key_entry_clone = api_key_entry.clone();
    let record_button_clone = record_button.clone();
    let api_key_error_label_clone = api_key_error_label.clone();
    let validate_api_key = move || {
        let app_state = app_state_clone.clone();
        let api_key_entry = api_key_entry_clone.clone();
        let record_button = record_button_clone.clone();
        let api_key_error_label = api_key_error_label_clone.clone();
        glib::MainContext::default().spawn_local(async move {
            let api_key = api_key_entry.text().to_string();
            println!("Attempting to validate API key: {}", api_key);
            if is_valid_api_key(&api_key) {
                println!("API key is valid");
                let mut state = app_state.lock().await;
                state.api_key = api_key;
                record_button.set_sensitive(true);
                api_key_entry.set_sensitive(false);
                api_key_error_label.set_visible(false);
            } else {
                println!("API key is invalid");
                api_key_error_label.set_markup("<span color=\"red\">Invalid API Key. Please enter a valid Groq API Key.</span>");
                api_key_error_label.set_visible(true);
            }
        });
    };

    api_key_entry.connect_activate(clone!(@strong validate_api_key => move |_| {
        validate_api_key();
    }));

    validate_button.connect_clicked(clone!(@strong validate_api_key => move |_| {
        validate_api_key();
    }));

    let app_state_clone = app_state.clone();
    let text_buffer = text_view.buffer();
    let transcribe_button_clone = transcribe_button.clone();
    let play_button_clone = play_button.clone();
    record_button.connect_clicked(move |button| {
        let app_state = app_state_clone.clone();
        let button = button.clone();
        let transcribe_button = transcribe_button_clone.clone();
        let play_button = play_button_clone.clone();
        glib::MainContext::default().spawn_local(clone!(@strong app_state => async move {
            let mut state = app_state.lock().await;
            if !state.is_recording {
                state.is_recording = true;
                button.set_label("Stop Recording");
                state.audio_data.clear();
                transcribe_button.set_sensitive(false);
                play_button.set_sensitive(false);
                drop(state);  // Release the lock before calling record_audio
                if let Err(e) = record_audio(app_state.clone()).await {
                    eprintln!("Error during recording: {}", e);
                }
            } else {
                if let Some(sender) = state.recording_stop_sender.take() {
                    let _ = sender.send(()).await;
                }
                state.is_recording = false;
                button.set_label("Start Recording");
                println!("Recording stopped, audio data length: {}", state.audio_data.len());
                transcribe_button.set_sensitive(true);
                play_button.set_sensitive(true);
            }
        }));
    });

    let app_state_clone = app_state.clone();
    play_button.connect_clicked(move |_| {
        let app_state = app_state_clone.clone();
        glib::MainContext::default().spawn_local(async move {
            let state = app_state.lock().await;
            if let Err(e) = play_audio(&state.audio_data) {
                eprintln!("Error playing audio: {}", e);
            }
        });
    });

    let app_state_clone = app_state.clone();
    let text_buffer_clone = text_buffer.clone();
    let copy_button_clone = copy_button.clone();
    transcribe_button.connect_clicked(move |_| {
        let app_state = app_state_clone.clone();
        let text_buffer = text_buffer_clone.clone();
        let copy_button = copy_button_clone.clone();
        glib::MainContext::default().spawn_local(async move {
            let state = app_state.lock().await;
            if state.audio_data.is_empty() {
                drop(state);
                text_buffer.set_text("Error: No audio data. Please record some audio before transcribing.");
                return;
            }
            drop(state);
            text_buffer.set_text("Transcribing... Please wait.");
            let result = transcribe_audio(app_state.clone()).await;
            let text = match result {
                Ok(transcription) => {
                    println!("Transcription successful: {}", transcription);
                    transcription
                },
                Err(e) => {
                    println!("Error during transcription: {}", e);
                    format!("Error during transcription: {}", e)
                },
            };
            text_buffer.set_text(&text);
            copy_button.set_sensitive(!text.is_empty());
            
            // Update the AppState with the transcribed text
            let mut state = app_state.lock().await;
            state.transcribed_text = text;
        });
    });

    let app_state_clone = app_state.clone();
    let window_clone = window.clone();
    copy_button.connect_clicked(move |_| {
        let app_state = app_state_clone.clone();
        let window = window_clone.clone();
        glib::MainContext::default().spawn_local(async move {
            let state = app_state.lock().await;
            let clipboard = window.clipboard();
            clipboard.set_text(&state.transcribed_text);
            println!("Copied to clipboard: {}", state.transcribed_text);
        });
    });

    let reset_button = gtk::Button::with_label("Reset");
    button_box.append(&reset_button);

    let app_state_clone = app_state.clone();
    let text_buffer_clone = text_buffer.clone();
    let transcribe_button_clone = transcribe_button.clone();
    let copy_button_clone = copy_button.clone();
    reset_button.connect_clicked(move |_| {
        let app_state = app_state_clone.clone();
        let text_buffer = text_buffer_clone.clone();
        let transcribe_button = transcribe_button_clone.clone();
        let copy_button = copy_button_clone.clone();
        glib::MainContext::default().spawn_local(async move {
            let mut state = app_state.lock().await;
            state.audio_data.clear();
            state.transcribed_text.clear();
            text_buffer.set_text("");
            transcribe_button.set_sensitive(false);
            copy_button.set_sensitive(false);
        });
    });

    window.present();
}

fn is_valid_api_key(api_key: &str) -> bool {
    // Check if the key starts with "gsk_" and is at least 20 characters long
    api_key.starts_with("gsk_") && api_key.len() >= 20
}

async fn record_audio(app_state: Arc<Mutex<AppState>>) -> Result<()> {
    let host = cpal::default_host();
    let device = host.default_input_device().expect("No input device available");
    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(SAMPLE_RATE),
        buffer_size: cpal::BufferSize::Default,
    };

    let (stop_sender, mut stop_receiver) = mpsc::channel(1);

    let app_state_clone = app_state.clone();
    let stream = device.build_input_stream(
        &config,
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

    {
        let mut state = app_state.lock().await;
        state.recording_stop_sender = Some(stop_sender);
    }

    tokio::select! {
        _ = stop_receiver.recv() => {
            println!("Stopping recording");
        }
    }

    stream.pause()?;

    let state = app_state.lock().await;
    let duration = state.audio_data.len() as f32 / SAMPLE_RATE as f32;
    println!("Recorded {} samples ({:.2} seconds) at {} Hz", state.audio_data.len(), duration, SAMPLE_RATE);

    Ok(())
}

fn play_audio(audio_data: &[f32]) -> Result<()> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    let source = rodio::buffer::SamplesBuffer::new(1, SAMPLE_RATE, audio_data);
    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}

async fn transcribe_audio(app_state: Arc<Mutex<AppState>>) -> Result<String> {
    let state = app_state.lock().await;
    let api_key = state.api_key.clone();
    let audio_data = state.audio_data.clone();
    drop(state);

    println!("Starting transcription process...");
    println!("Audio data length: {} samples ({:.2} seconds)", audio_data.len(), audio_data.len() as f32 / SAMPLE_RATE as f32);

    // Encode audio data to OGG format
    let mut writer = Cursor::new(Vec::new());
    let mut packet_writer = PacketWriter::new(&mut writer);
    let mut encoder = Encoder::new(1, SAMPLE_RATE as u64, vorbis::VorbisQuality::Midium)?;

    // Convert f32 samples to i16
    let samples: Vec<i16> = audio_data.iter().map(|&s| (s * 32767.0) as i16).collect();
    let encoded_data = encoder.encode(&samples)?;

    println!("Encoded data type: {:?}", std::any::type_name_of_val(&encoded_data));
    println!("Encoded data: {:?}", encoded_data);

    println!("Encoded {} bytes", encoded_data.len());

    // Write the entire encoded data as a single packet
    packet_writer.write_packet(encoded_data.into_boxed_slice(), 1, PacketWriteEndInfo::EndPage, 0)?;

    let ogg_data = writer.into_inner();
    println!("OGG file created in memory. Size: {} bytes", ogg_data.len());

    // Send the audio file to the Groq API
    let client = reqwest::Client::new();
    let part = reqwest::multipart::Part::bytes(ogg_data.clone())  // Clone here
        .file_name("audio.ogg")
        .mime_str("audio/ogg")?;
    let form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("model", "distil-whisper-large-v3-en")
        .text("response_format", "json")
        .text("language", "en")
        .text("temperature", "0");

    println!("Sending request to Groq API...");
    println!("Request details:");
    println!("  - Endpoint: https://api.groq.com/openai/v1/audio/transcriptions");
    println!("  - Model: distil-whisper-large-v3-en");
    println!("  - File size: {} bytes", ogg_data.len());

    let response = client
        .post("https://api.groq.com/openai/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await?;

    let status = response.status();
    println!("Received response from Groq API. Status: {}", status);

    if status.is_success() {
        let response_text = response.text().await?;
        println!("Full API Response: {}", response_text);

        let json: Value = serde_json::from_str(&response_text)?;
        println!("Successfully parsed JSON response");
        println!("Full JSON response: {}", serde_json::to_string_pretty(&json)?);

        let transcribed_text = json["text"].as_str().unwrap_or("").to_string();
        println!("Transcribed text: {}", transcribed_text);
        
        // Update the AppState with the transcribed text
        let mut state = app_state.lock().await;
        state.transcribed_text = transcribed_text.clone();
        
        Ok(transcribed_text)
    } else {
        let error_text = response.text().await?;
        println!("API request failed. Error: {}", error_text);
        Err(anyhow::anyhow!("API request failed: {}. Error: {}", status, error_text))
    }
}
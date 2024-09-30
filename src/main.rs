use adw::prelude::*;
use adw::{Application, Window, HeaderBar};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use anyhow::Result;
use glib;
use gtk::glib::clone;
use ogg::writing::PacketWriter;
use vorbis::Encoder;
use rodio::{OutputStream, Sink, Source};
use std::io::Cursor;
use ogg::PacketWriteEndInfo;
use rand::Rng;
use gstreamer as gst;
use gstreamer_app as gst_app;
use gstreamer_audio as gst_audio;
use bytemuck;
use gstreamer::prelude::*;
use cpal::traits::{DeviceTrait, HostTrait};
use std::time::Duration;
use hound;
use serde_json;

const APP_ID: &str = "com.example.VoiceTranscriber";

struct AppState {
    is_recording: bool,
    transcribed_text: String,
    api_key: String,
    audio_data: Vec<f32>,
    recording_stop_sender: Option<mpsc::Sender<()>>,
}

struct AudioBuffer {
    samples: Vec<f32>,
}

fn init_gstreamer() -> Result<()> {
    gst::init()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    init_gstreamer()?;
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
    text_view.set_editable(true);  // Make the text view editable
    let scrolled_window = gtk::ScrolledWindow::new();
    scrolled_window.set_child(Some(&text_view));
    scrolled_window.set_vexpand(true);
    content.append(&scrolled_window);

    // Add an error label
    let error_label = gtk::Label::new(None);
    error_label.set_markup("<span color=\"red\"></span>");
    error_label.set_visible(false);
    content.append(&error_label);

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

    let test_sound_button = gtk::Button::with_label("Play Test Sound");
    button_box.append(&test_sound_button);

    let test_recording_button = gtk::Button::with_label("Play Test Recording");
    button_box.append(&test_recording_button);

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
    let app_state_clone = app_state.clone();
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
                state.is_recording = false;
                if let Some(sender) = state.recording_stop_sender.take() {
                    let _ = sender.send(()).await;
                }
                button.set_label("Start Recording");
                transcribe_button.set_sensitive(true);
                play_button.set_sensitive(true);
                
                println!("Recorded audio data length: {} samples", state.audio_data.len());
                println!("First 10 samples: {:?}", &state.audio_data[..10.min(state.audio_data.len())]);
                println!("Last 10 samples: {:?}", &state.audio_data[state.audio_data.len().saturating_sub(10)..]);
            }
        }));
    });

    let app_state_clone = app_state.clone();
    play_button.connect_clicked(move |_| {
        let app_state = app_state_clone.clone();
        glib::MainContext::default().spawn_local(async move {
            let state = app_state.lock().await;
            if state.audio_data.is_empty() {
                println!("Error: No audio data to play");
                return;
            }
            let audio_data = state.audio_data.clone();
            drop(state);
            println!("Attempting to play {} samples", audio_data.len());
            match play_audio(audio_data) {
                Ok(_) => println!("Audio playback completed successfully"),
                Err(e) => eprintln!("Error playing audio: {}", e),
            }
        });
    });

    let app_state_clone = app_state.clone();
    let text_buffer_clone = text_buffer.clone();
    let copy_button_clone = copy_button.clone();
    let error_label_clone = error_label.clone();
    transcribe_button.connect_clicked(move |_| {
        let app_state = app_state_clone.clone();
        let text_buffer = text_buffer_clone.clone();
        let copy_button = copy_button_clone.clone();
        let error_label = error_label_clone.clone();
        glib::MainContext::default().spawn_local(async move {
            let state = app_state.lock().await;
            if state.audio_data.is_empty() {
                drop(state);
                error_label.set_markup("<span color=\"red\">Error: No audio data. Please record some audio before transcribing.</span>");
                error_label.set_visible(true);
                return;
            }
            drop(state);
            error_label.set_visible(false);
            let result = transcribe_audio(app_state.clone()).await;
            match result {
                Ok(transcription) => {
                    println!("Transcription successful: {}", transcription);
                    text_buffer.set_text(&transcription);
                    copy_button.set_sensitive(true);
                    
                    // Update the AppState with the transcribed text
                    let mut state = app_state.lock().await;
                    state.transcribed_text = transcription;
                },
                Err(e) => {
                    println!("Error during transcription: {}", e);
                    error_label.set_markup(&format!("<span color=\"red\">Error during transcription: {}</span>", e));
                    error_label.set_visible(true);
                },
            };
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

    test_sound_button.connect_clicked(move |_| {
        glib::MainContext::default().spawn_local(async move {
            match play_test_sound() {
                Ok(_) => println!("Test sound played successfully"),
                Err(e) => eprintln!("Error playing test sound: {}", e),
            }
        });
    });

    let app_state_clone = app_state.clone();
    test_recording_button.connect_clicked(move |_| {
        let app_state = app_state_clone.clone();
        glib::MainContext::default().spawn_local(async move {
            let test_audio = generate_test_recording();
            match play_audio(test_audio) {
                Ok(_) => println!("Test recording playback completed successfully"),
                Err(e) => eprintln!("Error playing test recording: {}", e),
            }
        });
    });

    // Add a new button for recording a test tone
    let record_test_tone_button = gtk::Button::with_label("Record Test Tone");
    button_box.append(&record_test_tone_button);

    let app_state_clone = app_state.clone();
    record_test_tone_button.connect_clicked(move |_| {
        let app_state = app_state_clone.clone();
        glib::MainContext::default().spawn_local(async move {
            if let Err(e) = record_test_tone(app_state) {
                eprintln!("Error recording test tone: {}", e);
            }
        });
    });

    window.present();
}

fn is_valid_api_key(api_key: &str) -> bool {
    // Check if the key starts with "gsk_" and is at least 20 characters long
    api_key.starts_with("gsk_") && api_key.len() >= 20
}

async fn record_audio(app_state: Arc<Mutex<AppState>>) -> Result<()> {
    let pipeline = gst::parse_launch(
        "autoaudiosrc ! audioconvert ! audioresample ! audio/x-raw,rate=44100,channels=1,format=F32LE ! appsink name=sink"
    )?;

    let sink = pipeline.downcast_ref::<gst::Bin>().unwrap()
        .by_name("sink")
        .expect("Sink element not found")
        .downcast::<gst_app::AppSink>()
        .expect("Sink element is not an AppSink");

    let app_state_clone = app_state.clone();
    sink.set_callbacks(
        gst_app::AppSinkCallbacks::builder()
            .new_sample(move |sink| {
                let sample = sink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                let buffer = sample.buffer().ok_or_else(|| gst::FlowError::Error)?;
                let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;
                
                let mut state = app_state_clone.blocking_lock();
                state.audio_data.extend_from_slice(bytemuck::cast_slice::<u8, f32>(&map));
                
                if state.is_recording {
                    Ok(gst::FlowSuccess::Ok)
                } else {
                    Err(gst::FlowError::Eos)
                }
            })
            .build()
    );

    pipeline.set_state(gst::State::Playing)?;

    // Wait for the recording to stop
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    {
        let mut state = app_state.lock().await;
        state.recording_stop_sender = Some(tx);
    }

    tokio::select! {
        _ = rx.recv() => {
            println!("Stopping recording...");
        }
    }

    pipeline.set_state(gst::State::Null)?;

    Ok(())
}

fn play_audio(audio_data: Vec<f32>) -> Result<()> {
    let pipeline = gst::parse_launch(
        "appsrc name=src ! audioconvert ! audioresample ! autoaudiosink"
    )?;

    let src = pipeline.downcast_ref::<gst::Bin>().unwrap()
        .by_name("src")
        .expect("Source element not found")
        .downcast::<gst_app::AppSrc>()
        .expect("Source element is not an AppSrc");

    src.set_caps(Some(&gst_audio::AudioInfo::builder(gst_audio::AudioFormat::F32le, 44100, 1).build().expect("Failed to build AudioInfo").to_caps().expect("Failed to convert AudioInfo to caps")));
    src.set_format(gst::Format::Time);

    pipeline.set_state(gst::State::Playing)?;

    // Calculate the duration
    let duration = gst::ClockTime::from_nseconds((audio_data.len() as u64 * 1_000_000_000) / 44100);

    // Create a new Buffer from the audio data
    let byte_data: Vec<u8> = audio_data.into_iter().flat_map(|f| f.to_le_bytes()).collect();
    let mut buffer = gst::Buffer::from_mut_slice(byte_data);

    // Set the duration on the buffer
    {
        let buffer_ref = buffer.get_mut().unwrap();
        buffer_ref.set_duration(duration);
    }

    src.push_buffer(buffer)?;
    src.end_of_stream()?;

    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;
        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                pipeline.set_state(gst::State::Null)?;
                return Err(anyhow::anyhow!("Error: {:?}", err));
            }
            _ => (),
        }
    }

    pipeline.set_state(gst::State::Null)?;
    Ok(())
}

fn play_test_sound() -> Result<()> {
    println!("Playing test sound...");
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    // Generate a simple sine wave
    let source = rodio::source::SineWave::new(440.0) // 440 Hz
        .take_duration(std::time::Duration::from_secs(1))
        .amplify(0.20);

    sink.append(source);
    sink.sleep_until_end();

    println!("Test sound playback finished");
    Ok(())
}

async fn transcribe_audio(app_state: Arc<Mutex<AppState>>) -> Result<String> {
    let state = app_state.lock().await;
    let api_key = state.api_key.clone();
    let audio_data = state.audio_data.clone();
    drop(state);

    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("No input device available");
    let config = input_device.default_input_config()?;
    let sample_rate = config.sample_rate().0;

    println!("Starting transcription process...");
    println!("Audio data length: {} samples ({:.2} seconds)", audio_data.len(), audio_data.len() as f32 / sample_rate as f32);
    println!("Sample rate: {} Hz", sample_rate);

    // WAV encoding
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut wav_buffer = Cursor::new(Vec::new());
    let mut wav_writer = hound::WavWriter::new(&mut wav_buffer, spec)?;
    for &sample in &audio_data {
        wav_writer.write_sample((sample * 32767.0) as i16)?;
    }
    wav_writer.finalize()?;
    let wav_data = wav_buffer.into_inner();

    let file_part = reqwest::multipart::Part::bytes(wav_data)
        .file_name("audio.wav")
        .mime_str("audio/wav")?;

    let form = reqwest::multipart::Form::new()
        .part("file", file_part)
        .text("model", "distil-whisper-large-v3-en")
        .text("temperature", "0")
        .text("response_format", "json")
        .text("language", "en");

    println!("Sending WAV file to Groq API...");
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.groq.com/openai/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await?;

    let status = response.status();
    println!("Response status: {}", status);

    if status.is_success() {
        let response_text = response.text().await?;
        println!("Response body: {}", response_text);
        let json: serde_json::Value = serde_json::from_str(&response_text)?;
        let transcribed_text = json["text"].as_str().unwrap_or("").to_string();
        Ok(transcribed_text)
    } else {
        let error_text = response.text().await?;
        println!("Error response body: {}", error_text);
        Err(anyhow::anyhow!("API request failed: {}. Error: {}", status, error_text))
    }
}

fn generate_test_recording() -> Vec<f32> {
    let duration_seconds = 3.0;
    let sample_rate = 44100;
    let num_samples = (sample_rate as f32 * duration_seconds) as usize;
    let mut rng = rand::thread_rng();

    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            let sine = (t * 440.0 * 2.0 * std::f32::consts::PI).sin();
            let noise = rng.gen_range(-0.1..0.1);
            (sine + noise) * 0.5
        })
        .collect()
}

// Add this new function
fn record_test_tone(app_state: Arc<Mutex<AppState>>) -> Result<()> {
    let duration_seconds = 3.0;
    let sample_rate = 44100;
    let num_samples = (sample_rate as f32 * duration_seconds) as usize;
    
    let test_tone: Vec<f32> = (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            (t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 0.5
        })
        .collect();

    let mut state = tokio::task::block_in_place(|| app_state.blocking_lock());
    state.audio_data = test_tone;
    
    println!("Recorded test tone: {} samples", state.audio_data.len());
    Ok(())
}

// Update the function signature
fn encode_to_mp3(audio_data: &[f32], _sample_rate: u32) -> Result<Vec<u8>> {
    // This is a placeholder function. In a real-world scenario, you'd use a proper MP3 encoding library.
    // For now, we'll just convert the f32 samples to i16 and pretend it's MP3 data.
    let mp3_data: Vec<u8> = audio_data
        .iter()
        .flat_map(|&sample| {
            let sample_i16 = (sample * 32767.0) as i16;
            sample_i16.to_le_bytes().to_vec()
        })
        .collect();

    Ok(mp3_data)
}
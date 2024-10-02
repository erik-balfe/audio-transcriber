# Voice Transcriber

## Version 0.2.0

A GTK4-based application for recording audio and transcribing it using the Groq API, built with Rust.

## Current Features

- Secure API key management using system keyring
- Audio recording functionality using GStreamer
- Audio playback capability
- User interface built with relm4 (GTK4 Rust bindings)
- Extensive logging for debugging and auditing
- Error handling with anyhow
- Asynchronous operations using tokio

## Current Project State

As of the latest update, the project is in a transitional phase:

- **Compilation**: The project compiles successfully without errors.
- **User Interface**:
  - The main window launches and displays correctly.
  - UI elements such as buttons and the API key input field are present and visible.
- **Functionality**:
  - While the codebase contains implementations for recording, playing, saving, loading, and API key validation, these features are currently not operational.
  - The application is in a minimal working state, primarily showcasing the UI structure.
- **User Feedback**:
  - The output field for transcribed text is not yet implemented in the current UI.
  - Error and warning labels are not currently displayed in the window.

In summary, the project has a solid foundation with a working UI framework, but the core functionalities are temporarily inactive as we refactor and improve the codebase. We are actively working on reconnecting these features and enhancing the user experience.

## Planned Features

- Audio recording with pause and resume functionality
- Audio transcription using the Groq API
- Playback of recorded audio
- Clipboard integration for transcribed text
- Error handling for API interactions
- Keyboard shortcuts for common actions
- Global shortcut for quick voice input from any application
- Settings panel for customization
- Local transcription option using Whisper models

## User Guide

1. **API Key Management**:

   - On first run, enter your Groq API key when prompted.
   - The key is securely stored in your system's keyring.
   - Use the "Remove API Key" button to delete the stored key.

2. **Recording Audio**:

   - Click the "Start Recording" button to begin recording.
   - Click "Stop Recording" to end the recording session.

3. **Playing Audio**:

   - After recording, click the "Play Recording" button to listen to the recorded audio.

4. **Transcription**:

   - (Coming soon) Click the "Transcribe" button to send the recorded audio to the Groq API for transcription.

5. **Logging**:
   - Set the `RUST_LOG` environment variable to control log verbosity:
     ```
     RUST_LOG=debug cargo run
     ```
   - Logs include detailed information about API key operations, app state, and GStreamer operations.

## Development Setup

1. Ensure you have Rust and Cargo installed.
2. Install GStreamer development libraries:
   - On Ubuntu/Debian: `sudo apt-get install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev gstreamer1.0-plugins-good`
   - On Fedora: `sudo dnf install gstreamer1-devel gstreamer1-plugins-base-devel gstreamer1-plugins-good`
3. Clone the repository.
4. Install dependencies: `cargo build`
5. Run the application: `cargo run`

## Contributing

Please see the [CONTRIBUTING.md](CONTRIBUTING.md) file for guidelines on contributing to this project.

## License

[MIT License](LICENSE)

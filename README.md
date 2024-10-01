# Voice Transcriber

WIP. Version 0.1

A GTK4-based application for recording audio and transcribing it using the Groq API.

## Current Features

- Secure API key management using system keyring
- Dynamic UI that reflects the presence or absence of an API key
- Extensive logging for debugging and auditing
- Basic UI structure for audio recording and transcription (functionality not yet implemented)

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
   - Use the "Remove API Key" button to delete the stored key when visible.

2. **Logging**:
   - Set the `RUST_LOG` environment variable to control log verbosity:
     ```
     RUST_LOG=debug cargo run
     ```
   - Logs include detailed information about API key operations and app state.

## Development Setup

1. Ensure you have Rust and Cargo installed.
2. Clone the repository.
3. Install dependencies: `cargo build`
4. Run the application: `cargo run`

## Contributing

Please see the [CONTRIBUTING.md](CONTRIBUTING.md) file for guidelines on contributing to this project.

## License

[Add license information here]

# Voice Transcriber
WIP. Version 0.1

A GTK4-based application for recording audio and transcribing it using the Groq API.

## Features

- Record audio from your microphone with pause and resume functionality
- Transcribe audio using the Groq API
- Copy transcribed text to clipboard
- Play back recorded audio
- Robust state management
- Asynchronous operations with proper cancellation

## User Guide

1. **Enter API Key**: 
   - Launch the application.
   - Enter your Groq API key in the provided field.
   - Click "Validate API Key" or press Enter.

2. **Record Audio**:
   - Click "Start Recording" or press Space to begin recording.
   - Click "Pause Recording" to pause, and "Resume Recording" to continue.
   - Click "Stop Recording" or press Space again when finished.

3. **Transcribe**:
   - Click the "Transcribe" button to send the audio to Groq API for transcription.
   - The transcribed text will appear in the text area.

4. **Copy Text**:
   - Click "Copy to Clipboard" or press Ctrl+C to copy the transcribed text.

5. **Play Recording**:
   - Click "Play Recording" to listen to your recorded audio.

6. **Reset**:
   - Click "Reset" or press Ctrl+Backspace to clear all data and start over.

## Planned Features for First Release

1. **API Key Management**:
   - Save API key for future use
   - Add a button to reset and re-enter API key

2. **Improved Recording Functionality**:
   - Implement pausable and resumable recording
   - Allow recording in multiple parts
   - Add ability to rollback and cancel the latest recording part

3. **Error Handling**:
   - Handle common API errors (e.g., rate limits, max size limits)
   - Add notice about maximum audio time or size limitations

4. **Keyboard Shortcuts**:
   - Space: Start/Stop recording
   - Ctrl+Backspace: Reset
   - Ctrl+C: Copy to clipboard

5. **User Interface Improvements**:
   - Clear indication of recording status and parts
   - Visual feedback for API interactions and errors


6. **Global Shortcut Transcription**:
   - Use a global shortcut (e.g., Ctrl+Shift+R) to start voice input from anywhere
   - An indicator in the system tray shows recording status
   - Press another shortcut (e.g., Ctrl+Enter) to stop recording and initiate transcription
   - Transcription result is automatically inserted into the active text field or copied to clipboard

7. **App Settings**:
   - Access a dedicated settings screen from the app menu
   - Configure language settings (changes model to multilingual as needed)
   - Customize global and in-app shortcuts
   - Adjust audio settings (sample rate, channels, etc.)
   - Set preferences for transcription output (insert to active field vs. copy to clipboard)
   - Choose light/dark theme
   - Configure API-related settings (endpoint URL, timeout, etc.)
   - Option to enable/disable automatic updates

8. **Local Transcription with Whisper**:
   - Implement offline transcription using Whisper models
   - Provide option to use Whisper locally without relying on API
   - Include choice of different Whisper model sizes:
     - Base model for fast, efficient transcription on most modern PCs
     - Medium and large models for improved accuracy when needed
   - Utilize whisper-rs for Rust integration of Whisper
   - Allow users to switch between API and local transcription in settings
   - Display model download progress and estimated transcription time
   - Offer guidance on model selection based on device capabilities

## Installation

[Add installation instructions here]

## Configuration

To enable logging, set the `RUST_LOG` environment variable before running the application:

```
RUST_LOG=debug cargo run
```

Adjust the log level as needed (e.g., `info`, `warn`, `error`).

## Troubleshooting

- Ensure your microphone is properly connected and configured.
- Check your internet connection if transcription fails.
- Verify that your Groq API key is correct and has the necessary permissions.

## Privacy Note

This application sends audio data to the Groq API for transcription. Please be aware of any sensitive information you may record and transmit.

## Contributing

For information on building and contributing to this project, please see the [CONTRIBUTING.md](CONTRIBUTING.md) file.

## License

[Add license information here]
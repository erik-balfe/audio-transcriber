# Voice Transcriber

A GTK4-based application for recording audio and transcribing it using the Groq API.

## User Guide

Follow these steps to use the Voice Transcriber:

1. **Enter API Key**: 
   - Launch the application.
   - Enter your Groq API key in the text field at the top of the window.
   - Click the "Validate API Key" button.
   - If the key is valid, the "Start Recording" button will be enabled.

2. **Record Audio**:
   - Click the "Start Recording" button to begin recording audio.
   - Speak into your microphone.
   - Click the "Stop Recording" button (same button, now relabeled) when you're finished.

3. **Transcribe Audio**:
   - After stopping the recording, the "Transcribe" button will be enabled.
   - Click the "Transcribe" button to send the audio to the Groq API for transcription.
   - The transcribed text will appear in the text area.

4. **Copy Transcription** (optional):
   - If the transcription is successful, the "Copy to Clipboard" button will be enabled.
   - Click this button to copy the transcribed text to your clipboard.

## Troubleshooting

- If you encounter an error message, read it carefully for instructions on how to proceed.
- Ensure your microphone is properly connected and functioning.
- Check your internet connection if transcription fails.
- Verify that your Groq API key is correct and has the necessary permissions.

## Privacy Note

This application sends audio data to the Groq API for transcription. Please be aware of any sensitive information you may record and transmit.

## Development

For information on building and contributing to this project, please see the CONTRIBUTING.md file.
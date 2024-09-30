# Voice Transcriber

A GTK4-based application for recording audio and transcribing it using the Groq API.

## User Guide

Follow these steps to use the Voice Transcriber:

1. **Enter API Key**: 
   - Launch the application.
   - Enter your Groq API key in the provided field.
   - Click "Validate API Key" or press Enter.

2. **Record Audio**:
   - Once the API key is validated, click "Start Recording".
   - Speak into your microphone.
   - Click "Stop Recording" when finished.

3. **Transcribe**:
   - Click the "Transcribe" button to send the audio to Groq API for transcription.
   - The transcribed text will appear in the text area.

4. **Copy Text**:
   - Click "Copy to Clipboard" to copy the transcribed text.

## Troubleshooting

- Ensure your microphone is properly connected and configured.
- Check your internet connection if transcription fails.
- Verify that your Groq API key is correct and has the necessary permissions.

## Privacy Note

This application sends audio data to the Groq API for transcription. Please be aware of any sensitive information you may record and transmit.

## Development

For information on building and contributing to this project, please see the CONTRIBUTING.md file.

## Development Challenges and Approaches

During the development of this application, several challenges were encountered and various approaches were attempted. Here's a summary of what worked and what didn't:

### What Worked:

1. **GUI Framework**: 
   - Using GTK4 for the user interface proved successful.

2. **Asynchronous Operations**: 
   - Implementing `tokio` for asynchronous operations improved responsiveness.
   - Using `Arc<Mutex<>>` for shared state management across asynchronous boundaries was effective.

3. **API Integration**: 
   - Successfully implemented OGG encoding for the audio data to meet Groq API requirements.

4. **Debugging**: 
   - Adding extensive debug logging helped track the flow of data through the application.
   - Implementing test sound and test recording features was useful for isolating audio handling issues.

### What Didn't Work Well:

1. **Audio Recording and Playback**: 
   - Using `cpal` for both recording and playback has been problematic.
   - The fixed-duration recording (5 seconds) is a limitation.
   - Properly stopping the recording on button press is still an issue.

2. **Audio Format Compatibility**: 
   - Attempts to use `rodio` for playback faced issues with format compatibility.

3. **GUI Responsiveness**: 
   - Despite improvements, keeping the GUI fully responsive during long-running operations remains a challenge.

4. **Error Handling**: 
   - The current error handling approach is not comprehensive enough, leading to potential stability issues.

5. **User Feedback**: 
   - The application lacks sufficient feedback mechanisms for users during operations like recording and transcription.

### Areas for Improvement:

1. Implement a more robust audio recording and playback system, possibly exploring alternatives to `cpal`.
2. Develop a better mechanism for stopping recording on demand.
3. Enhance error handling and user feedback throughout the application.
4. Optimize the transcription process to handle larger audio files more efficiently.
5. Implement proper cleanup of resources, especially for audio streams.

These insights should guide future development efforts, focusing on resolving the persistent issues with audio handling and improving overall user experience.
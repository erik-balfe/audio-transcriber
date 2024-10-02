# Contributing to Voice Transcriber

## Project Architecture

- **Modular Design**: Separated into audio, config, state, and ui modules.
- **State Management**: Centralized `StateManager` for consistent state across components.
- **Asynchronous Operations**: Using Rust's async/await and tokio for I/O-bound tasks.
- **Error Handling**: Utilizing `anyhow` for flexible error handling and propagation.
- **Configuration**: `Config` struct manages app settings and API key.
- **UI**: Built with relm4, providing a reactive approach to GTK4 development in Rust.
- **Dependency Injection**: Dependencies like `StateManager` are passed to components.
- **Secure Storage**: API keys stored in the system's keyring.
- **Logging**: Consistent logging throughout using the `log` crate.
- **Audio Handling**: GStreamer for audio recording and playback.

## Current Focus Areas

1. Implementing audio transcription functionality with the Groq API.
2. Enhancing error handling and user notifications.
3. Improving UI responsiveness and feedback.
4. Adding clipboard integration for transcribed text.
5. Implementing global shortcuts for quick access.

## Development Workflow

1. Fork the repository and create a feature branch for each new feature or bug fix.
2. Write unit tests for new functionality.
3. Ensure all tests pass before submitting a pull request.
4. Update documentation, including inline comments and this file if necessary.
5. Submit a pull request for code review.

## Code Style and Best Practices

1. Follow Rust's official style guide. Run `rustfmt` before committing.
2. Use `Arc` for thread-safe reference counting, `Rc` for single-threaded scenarios.
3. Prefer borrowing over cloning when possible.
4. Utilize Rust's type system to prevent runtime errors (e.g., `Option`, `Result`).
5. Use descriptive variable and function names.
6. Keep functions small and focused on a single task.
7. Use `anyhow::Result` for error propagation in public functions.

## Error Handling and Logging

- Use `anyhow::Result` for error propagation.
- Add context to errors with `.context()` or `.with_context()`.
- Log at appropriate levels: error, warn, info, debug, trace.
- Never log sensitive information like full API keys.

## Current Focus Areas

- Write unit tests for all new functionality.
- Use integration tests for testing the interaction between different modules.
- Aim for high test coverage, especially for critical paths.

## Documentation

- Keep the README.md up to date with new features and changes.
- Document public APIs with rustdoc comments.
- Update this CONTRIBUTING.md file when development practices change.

## Getting Started

1. Familiarize yourself with the project structure and existing code.
2. Check the issue tracker for open tasks or bug reports.
3. For major changes, open an issue first to discuss your proposed changes.

Your contributions to improving the Voice Transcriber are greatly appreciated!

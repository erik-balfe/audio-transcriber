# Contributing to Voice Transcriber

## Project Architecture

- **Modular Design**: Separated into audio, config, state, and ui modules.
- **State Management**: Centralized `StateManager` for consistent state across components.
- **Asynchronous Operations**: Using Rust's async/await and tokio for I/O-bound tasks.
- **Error Handling**: Utilizing `anyhow` for flexible error handling and propagation.
- **Configuration**: `Config` struct manages app settings and API key.
- **UI**: Built with GTK4 and libadwaita, following a builder pattern.
- **Dependency Injection**: Dependencies like `StateManager` are passed to components.
- **Secure Storage**: API keys stored in the system's keyring.
- **Logging**: Consistent logging throughout using the `log` crate.

## Development Workflow

1. Create a feature branch for each new feature or bug fix.
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

## Error Handling and Logging

- Use `anyhow::Result` for error propagation.
- Add context to errors with `.context()` or `.with_context()`.
- Log at appropriate levels: error, warn, info, debug.
- Never log sensitive information like full API keys.

## Current Focus Areas

1. Implementing audio recording functionality.
2. Integrating with the Groq API for transcription.
3. Improving UI responsiveness and feedback.
4. Enhancing error handling and user notifications.

## Getting Started

1. Familiarize yourself with the project structure and existing code.
2. Check the issue tracker for open tasks or bug reports.
3. For major changes, open an issue first to discuss your proposed changes.

Your contributions to improving the Voice Transcriber are greatly appreciated!

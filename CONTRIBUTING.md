# Contributing to Voice Transcriber

Thank you for your interest in contributing to the Voice Transcriber project! This document provides guidelines and information for contributors.

## Development Setup

[Add instructions for setting up the development environment]

## Building the Project

[Add instructions for building the project]

## Running Tests

[Add instructions for running tests, if applicable]

## Code Style and Guidelines

[Add any specific code style guidelines or linting instructions]

## Submitting Changes

[Add instructions for submitting pull requests or patches]

## Development Challenges and Solutions

### 1. Move Errors in Closures

**Problem:** The compiler was reporting move errors related to the `validate_api_key` closure being moved when called within the `connect_activate` and `connect_clicked` closures.

**Solution:** We restructured the code to clone necessary variables before the `validate_api_key` closure definition. This allows the closure to capture cloned versions of the variables, preventing move errors.

**Lesson Learned:** When working with closures that need to be called multiple times or from different contexts, it's often necessary to clone the captured variables to avoid ownership issues.

### 2. Unused Variables

**Problem:** The compiler was warning about unused variables `app_state_clone` and `app_state`.

**Solution:** We removed the unused variables and adjusted the code to use the necessary cloned versions directly where needed.

**Lesson Learned:** Always pay attention to compiler warnings about unused variables. They can often indicate redundant code or potential logical errors in the program flow.

### 3. Multipart Form File Upload

**Problem:** The compiler was reporting an error when trying to use `reqwest::multipart::Part::file()` method, as it didn't satisfy the required trait bounds.

**Solution:** We changed the approach to read the file contents into memory and use `reqwest::multipart::Part::bytes()` instead. This allows us to create the multipart form data without relying on the `file()` method.

**Lesson Learned:** When working with external libraries, it's important to check the available methods and their requirements. Sometimes, a different approach might be necessary to achieve the same goal.

## Best Practices for This Project

1. **Error Handling:** Use `anyhow::Result` for error propagation throughout the application.
2. **Asynchronous Programming:** Utilize `tokio` for asynchronous operations.
3. **State Management:** Use `Arc<Mutex<>>` for shared state across different threads or asynchronous tasks.
4. **Logging:** Implement comprehensive logging throughout the application.
5. **Resource Management:** Ensure proper cleanup of resources, particularly for audio streams and temporary files.
6. **User Feedback:** Provide clear and immediate feedback to the user for all operations.
7. **Code Organization:** Keep related functionality grouped together.
8. **Testing:** Implement unit tests for core functionality and integration tests for API interactions.
9. **Configuration:** Use environment variables or a configuration file for settings like API endpoints.
10. **Documentation:** Maintain clear and up-to-date documentation.

## Rust-Specific Challenges and Best Practices

### Ownership and Borrowing Issues

When working with GTK widgets in async closures, we encountered issues with ownership and borrowing. GTK widgets don't implement the `Copy` trait and can't be moved across thread boundaries.

**Solution:** Clone the necessary widgets before moving them into the async closure. This allows us to have separate references to the widgets that can be safely moved into the closure.

**Best Practices:**

1. **Clone Widgets Before Moving:** When you need to use GTK widgets in async closures, clone them before moving:

   ```rust
   let widget_clone = widget.clone();
   glib::MainContext::default().spawn_local(async move {
       widget_clone.do_something();
   });
   ```

2. **Use Strong References:** When cloning widgets, you're creating strong references. Be mindful of potential circular references that could lead to memory leaks.

3. **Minimize Cloning:** Only clone the widgets you need within the closure to minimize memory usage.

4. **Consider Weak References:** For long-lived closures, consider using weak references to widgets to avoid keeping them alive longer than necessary.

5. **Update UI on Main Thread:** Remember that GTK is not thread-safe. Always update UI elements on the main thread, which `glib::MainContext::default().spawn_local()` ensures.

## Areas for Improvement

1. Implement a more robust audio recording and playback system.
2. Develop a better mechanism for stopping recording on demand.
3. Enhance error handling and user feedback throughout the application.
4. Optimize the transcription process to handle larger audio files more efficiently.
5. Implement proper cleanup of resources, especially for audio streams.
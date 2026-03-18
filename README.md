# Update

[0.2.0.alpha] 
2025.03.18 - added binary IPC, migrate framework to Tokio async architecture, fixed some bugs, updated documentation.

[0.1.0.alpha] 
2025.03.14 - added Zeroize and region (mlock) protection to the Rust-side storage.

# Alakit

Alakit is a hybrid desktop application framework that combines the power of Rust with the flexibility of web technologies. The goal of this project is to provide an environment where you can build your interface using HTML and CSS without being forced to write JavaScript for the application logic.

## Why Alakit?

Many frameworks are either too complex for rapid prototyping or produce excessively large binaries. Alakit tries to find a middle ground: Rust provides the safety and performance, the Webview handles the rendering, and a declarative, attribute-based system manages the bridge between them.

## Key Features

* No-JS approach: Button events, form submissions, and UI value updates are tied directly to Rust code via HTML attributes.
* Protected backend Store: Sensitive data is stored in memory using AES-256-GCM encryption on the Rust side. (Note: Due to JS runtime limitations, data displayed in the Webview exists as plaintext in RAM).
* Low resource footprint: We focus on keeping binaries small and optimizing memory usage during runtime.
* Flexible controller system: Thanks to macros, adding new functionality (controllers) is automatic, removing the need for manual registration.

## How it works

The interface uses special attributes to communicate with the backend:
- alakit-cmd: Specifies which Rust function to execute.
- alakit-bind: Connects an HTML element to a key in the encrypted store.
- alakit-form: Gathers data from an entire container in JSON format for the Rust side.

In the background, Rust controllers process messages and update the state, which is immediately reflected in the UI.

## Installation and Usage

The project is currently in the development phase. You will need a Rust environment to build it. Example applications can be started by navigating to their respective folders and running 'cargo run'.

Detailed documentation and examples are located in the 'doc' folder.

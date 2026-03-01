pub use makepad_widgets;

pub mod api;
pub mod app;
pub mod chat;
pub mod dataflow;

// Tools module only available on native platforms (uses shell commands)
#[cfg(not(target_arch = "wasm32"))]
pub mod tools;

// OTLP telemetry client module only available on native platforms
#[cfg(not(target_arch = "wasm32"))]
pub mod otlp;

// Traces panel module only available on native platforms
#[cfg(not(target_arch = "wasm32"))]
pub mod traces;

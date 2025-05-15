// src/offline/llm/providers/mod.rs
//! Provider implementations for different LLM backends

pub mod ollama;
pub mod localai;

// Re-export providers for easier access
pub use ollama::OllamaProvider;
pub use localai::LocalAIProvider;
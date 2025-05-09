//! Providers for local LLM integration
//!
//! This module contains concrete implementations of the `LocalLLMProvider` trait
//! for different local LLM providers such as Ollama, LocalAI, etc.

pub mod ollama;
pub mod localai;

// Re-export providers for easier access
pub use ollama::OllamaProvider;
pub use localai::LocalAIProvider;

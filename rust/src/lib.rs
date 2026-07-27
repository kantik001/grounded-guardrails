//! Grounded Guardrails — token-level verification primitives.
//!
//! This crate is the performance-critical core of the guardrails service.
//! Higher layers (Go gRPC on `:50052`, Python clients, vLLM adapters) call into
//! these primitives; they do not reimplement numeric/PII logic.
//!
//! # Modules
//!
//! - [`buffer`] — fixed-capacity ring buffer for streaming token IDs (zero growth after init)

pub mod buffer;

pub use buffer::TokenRingBuffer;

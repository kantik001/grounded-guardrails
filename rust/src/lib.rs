//! Grounded Guardrails — token-level verification primitives.
//!
//! This crate is the performance-critical core of the guardrails service.
//! Higher layers (Go gRPC on `:50052`, Python clients, vLLM adapters) call into
//! these primitives; they do not reimplement numeric/PII logic.
//!
//! # Modules
//!
//! - [`buffer`] — fixed-capacity ring buffer for streaming token IDs
//! - [`pii`] — regex PII detection + masking
//! - [`numeric`] — numeric extract + tolerance verify (canonical algorithm)

pub mod buffer;
pub mod numeric;
pub mod pii;

pub use buffer::TokenRingBuffer;
pub use numeric::{
    DEFAULT_TOLERANCE, NumericValue, extract_numerics, unmatched_numerics,
    verify_answer_against_context, verify_numeric,
};
pub use pii::{PiiDetector, PiiMatch, PiiType};

//! End-to-End Tests
//!
//! Full system tests that require a running Dora instance.
//! Run with: cargo test --test e2e -- --ignored

mod dataflow_lifecycle;
mod full_workflow;

// Re-export test utilities
pub use super::fixtures::*;
pub use super::mocks::*;

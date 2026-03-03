//! Dora Studio Test Suite
//!
//! Comprehensive test suite including unit tests, integration tests, and E2E tests.
//!
//! ## Running Tests
//!
//! ```bash
//! # Run all unit and integration tests
//! cargo test --workspace
//!
//! # Run integration tests only
//! cargo test --test integration
//!
//! # Run E2E tests (requires running Dora)
//! cargo test --test e2e -- --ignored
//!
//! # Run with coverage
//! cargo tarpaulin --out Html
//! ```
//!
//! ## Test Organization
//!
//! - `mocks/` - Mock implementations of external dependencies
//! - `fixtures/` - Test data factories and helpers
//! - `integration/` - Cross-component integration tests
//! - `e2e/` - Full system end-to-end tests
//!
//! ## See Also
//!
//! - [TEST_PLAN.md](../TEST_PLAN.md) - Comprehensive test specifications
//! - [ISSUES.md](../ISSUES.md) - Trackable issues with test criteria

pub mod e2e;
pub mod fixtures;
pub mod integration;
pub mod mocks;

// Re-export commonly used items
pub use fixtures::*;
pub use mocks::*;

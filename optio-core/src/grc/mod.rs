//! GRC (Governance, Risk, Compliance) Module
//!
//! Provides interactive audit, gap analysis, and policy generation
//! supporting NIST CSF 2.0, SOC 2 Type II, and GDPR frameworks.

pub mod models;
pub mod frameworks;
pub mod repository;

pub use models::*;
pub use frameworks::*;
pub use repository::*;

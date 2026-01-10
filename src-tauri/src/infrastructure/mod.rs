//! Infrastructure & Migration Module
//!
//! Provides cloud readiness assessment, Kubernetes hardening audits,
//! and FinOps cost calculations for migration planning.

pub mod models;
pub mod cloud_readiness;
pub mod k8s_hardening;
pub mod finops;

pub use models::*;
pub use cloud_readiness::*;
pub use k8s_hardening::*;
pub use finops::*;

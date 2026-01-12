//! Intelligent Reporting Module
//!
//! Provides comprehensive report generation including executive summaries,
//! technical assessments, compliance reports, and PDF export.

pub mod models;
pub mod generator;
pub mod templates;
pub mod pdf_generator;

pub use models::*;
pub use generator::*;
pub use templates::*;
pub use pdf_generator::*;

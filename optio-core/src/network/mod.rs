//! Network Intelligence Module
//!
//! Provides network discovery, Nmap integration, and asset inventory management.
//! Enables consultants to map client networks and track discovered assets.

pub mod models;
pub mod scanner;
pub mod inventory;

pub use models::*;
pub use scanner::*;
pub use inventory::*;

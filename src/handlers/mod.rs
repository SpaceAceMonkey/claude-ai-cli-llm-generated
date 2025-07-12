// src/handlers/mod.rs
pub mod api;
pub mod history;
pub mod file_ops;
pub mod events;

// Test modules
#[cfg(test)]
mod api_tests;
#[cfg(test)]
mod file_ops_tests;
#[cfg(test)]
mod file_ops_module_tests;
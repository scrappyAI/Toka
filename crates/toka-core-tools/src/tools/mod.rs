//! Core tool implementations
//!
//! This module contains the core tool implementations that follow the
//! kernel enforcement pattern and CoreBaseline guidelines.

pub mod file_reader;
pub mod file_writer;
pub mod command_executor;
pub mod http_client;

// Re-export tool constructors for convenience
pub use file_reader::create_tool_definition as file_reader_tool;
pub use file_writer::create_tool_definition as file_writer_tool;
pub use command_executor::create_tool_definition as command_executor_tool;
pub use http_client::create_tool_definition as http_client_tool; 
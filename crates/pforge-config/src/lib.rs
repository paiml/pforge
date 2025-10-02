//! # pforge-config
//!
//! Configuration parsing and validation for pforge MCP servers.
//!
//! This crate provides YAML-based configuration with compile-time type safety
//! and comprehensive validation.
//!
//! ## Quick Start
//!
//! ```rust
//! use pforge_config::{parse_config_from_str, validate_config};
//!
//! let yaml = r#"
//! forge:
//!   name: my-server
//!   version: 0.1.0
//!   transport: stdio
//!
//! tools:
//!   - type: native
//!     name: greet
//!     description: "Greet a person"
//!     handler:
//!       path: "handlers::greet"
//!     params: {}
//! "#;
//!
//! let config = parse_config_from_str(yaml).expect("valid config");
//! validate_config(&config).expect("validation passes");
//!
//! assert_eq!(config.forge.name, "my-server");
//! assert_eq!(config.tools.len(), 1);
//! ```
//!
//! ## Validation Rules
//!
//! - Tool names must be unique
//! - Native handlers must have valid handler paths (format: `module::function`)
//! - All required fields must be present
//! - Transport type must be valid (stdio, sse, websocket)

pub mod error;
pub mod parser;
pub mod types;
pub mod validator;

pub use error::{ConfigError, Result};
pub use parser::{parse_config, parse_config_from_str};
pub use types::*;
pub use validator::validate_config;

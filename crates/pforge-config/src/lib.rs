pub mod error;
pub mod parser;
pub mod types;
pub mod validator;

pub use error::{ConfigError, Result};
pub use parser::{parse_config, parse_config_from_str};
pub use types::*;
pub use validator::validate_config;

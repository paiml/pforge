use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

/// Core handler abstraction - zero-cost, compatible with pmcp TypedTool
#[async_trait]
pub trait Handler: Send + Sync + 'static {
    type Input: JsonSchema + DeserializeOwned + Send;
    type Output: JsonSchema + Serialize + Send;
    type Error: Into<crate::Error>;

    /// Execute the handler with type-safe input
    async fn handle(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;

    /// Generate JSON schema for input (override for custom schemas)
    fn input_schema() -> schemars::schema::RootSchema {
        schemars::schema_for!(Self::Input)
    }

    /// Generate JSON schema for output
    fn output_schema() -> schemars::schema::RootSchema {
        schemars::schema_for!(Self::Output)
    }
}

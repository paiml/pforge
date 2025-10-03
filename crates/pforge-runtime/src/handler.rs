use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

/// Core handler abstraction - zero-cost, compatible with pmcp TypedTool.
///
/// The `Handler` trait provides a type-safe interface for implementing MCP tools.
/// It leverages Rust's type system for compile-time validation and automatic
/// JSON Schema generation.
///
/// # Type Parameters
///
/// - `Input`: The input type, must be deserializable and have a JSON schema
/// - `Output`: The output type, must be serializable and have a JSON schema
/// - `Error`: Error type that can be converted into `pforge_runtime::Error`
///
/// # Examples
///
/// ## Basic Handler
///
/// ```rust
/// use pforge_runtime::{Handler, Result};
/// use serde::{Deserialize, Serialize};
/// use schemars::JsonSchema;
///
/// #[derive(Debug, Deserialize, JsonSchema)]
/// struct GreetInput {
///     name: String,
/// }
///
/// #[derive(Debug, Serialize, JsonSchema)]
/// struct GreetOutput {
///     message: String,
/// }
///
/// struct GreetHandler;
///
/// #[async_trait::async_trait]
/// impl Handler for GreetHandler {
///     type Input = GreetInput;
///     type Output = GreetOutput;
///     type Error = pforge_runtime::Error;
///
///     async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
///         Ok(GreetOutput {
///             message: format!("Hello, {}!", input.name),
///         })
///     }
/// }
/// ```
///
/// ## Handler with Validation
///
/// ```rust
/// use pforge_runtime::{Handler, Result, Error};
/// use serde::{Deserialize, Serialize};
/// use schemars::JsonSchema;
///
/// #[derive(Debug, Deserialize, JsonSchema)]
/// struct AgeInput {
///     age: i32,
/// }
///
/// #[derive(Debug, Serialize, JsonSchema)]
/// struct AgeOutput {
///     category: String,
/// }
///
/// struct AgeHandler;
///
/// #[async_trait::async_trait]
/// impl Handler for AgeHandler {
///     type Input = AgeInput;
///     type Output = AgeOutput;
///     type Error = Error;
///
///     async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
///         if input.age < 0 {
///             return Err(Error::Handler("Age cannot be negative".to_string()));
///         }
///
///         let category = match input.age {
///             0..=12 => "child",
///             13..=19 => "teenager",
///             20..=64 => "adult",
///             _ => "senior",
///         }.to_string();
///
///         Ok(AgeOutput { category })
///     }
/// }
/// ```
#[async_trait]
pub trait Handler: Send + Sync + 'static {
    /// Input type for the handler
    type Input: JsonSchema + DeserializeOwned + Send;

    /// Output type for the handler
    type Output: JsonSchema + Serialize + Send;

    /// Error type that can be converted to `pforge_runtime::Error`
    type Error: Into<crate::Error>;

    /// Execute the handler with type-safe input.
    ///
    /// This is the core method where tool logic is implemented.
    /// Input is automatically deserialized and output is automatically serialized.
    async fn handle(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;

    /// Generate JSON schema for input (override for custom schemas).
    ///
    /// By default, this derives the schema from the `Input` type using schemars.
    /// Override this method to provide custom validation rules or documentation.
    fn input_schema() -> schemars::schema::RootSchema {
        schemars::schema_for!(Self::Input)
    }

    /// Generate JSON schema for output.
    ///
    /// By default, this derives the schema from the `Output` type using schemars.
    fn output_schema() -> schemars::schema::RootSchema {
        schemars::schema_for!(Self::Output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, JsonSchema)]
    struct TestInput {
        value: i32,
    }

    #[derive(Debug, Serialize, JsonSchema)]
    struct TestOutput {
        result: i32,
    }

    struct TestHandler;

    #[async_trait]
    impl Handler for TestHandler {
        type Input = TestInput;
        type Output = TestOutput;
        type Error = crate::Error;

        async fn handle(&self, input: Self::Input) -> crate::Result<Self::Output> {
            Ok(TestOutput {
                result: input.value * 2,
            })
        }
    }

    #[tokio::test]
    async fn test_handler_schema_not_default() {
        // Test that schemas are NOT Default::default() - kills the mutant
        let input_schema = TestHandler::input_schema();
        let default_schema = schemars::schema::RootSchema::default();

        assert_ne!(
            serde_json::to_string(&input_schema).unwrap(),
            serde_json::to_string(&default_schema).unwrap(),
            "Handler::input_schema() must not return Default::default()"
        );

        let output_schema = TestHandler::output_schema();
        assert_ne!(
            serde_json::to_string(&output_schema).unwrap(),
            serde_json::to_string(&default_schema).unwrap(),
            "Handler::output_schema() must not return Default::default()"
        );
    }

    #[tokio::test]
    async fn test_handler_execution() {
        let handler = TestHandler;
        let input = TestInput { value: 21 };
        let result = handler.handle(input).await.unwrap();
        assert_eq!(result.result, 42);
    }
}

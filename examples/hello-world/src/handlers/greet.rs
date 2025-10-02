use pforge_runtime::{Handler, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GreetInput {
    pub name: String,
    #[serde(default = "default_greeting")]
    pub greeting: String,
}

fn default_greeting() -> String {
    "Hello".to_string()
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GreetOutput {
    pub message: String,
}

pub struct GreetHandler;

#[async_trait::async_trait]
impl Handler for GreetHandler {
    type Input = GreetInput;
    type Output = GreetOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        Ok(GreetOutput {
            message: format!("{}, {}!", input.greeting, input.name),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_greet_default() {
        let handler = GreetHandler;
        let input = GreetInput {
            name: "World".to_string(),
            greeting: "Hello".to_string(),
        };

        let result = handler.handle(input).await.unwrap();
        assert_eq!(result.message, "Hello, World!");
    }

    #[tokio::test]
    async fn test_greet_custom() {
        let handler = GreetHandler;
        let input = GreetInput {
            name: "Alice".to_string(),
            greeting: "Hi".to_string(),
        };

        let result = handler.handle(input).await.unwrap();
        assert_eq!(result.message, "Hi, Alice!");
    }
}

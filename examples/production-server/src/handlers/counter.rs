use pforge_runtime::{Handler, Result, StateManager, MemoryStateManager};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CounterInput {
    pub name: String,
    #[serde(default = "default_increment")]
    pub increment: i64,
}

fn default_increment() -> i64 {
    1
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CounterOutput {
    pub name: String,
    pub value: i64,
    pub previous_value: i64,
    pub increment: i64,
}

pub struct CounterHandler {
    state: Arc<MemoryStateManager>,
}

impl CounterHandler {
    pub fn new(state: Arc<MemoryStateManager>) -> Self {
        Self { state }
    }
}

#[async_trait::async_trait]
impl Handler for CounterHandler {
    type Input = CounterInput;
    type Output = CounterOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        // Get current value from state
        let previous_value = match self.state.get(&input.name).await {
            Ok(Some(value)) => String::from_utf8_lossy(&value)
                .parse::<i64>()
                .unwrap_or(0),
            _ => 0,
        };

        // Calculate new value
        let new_value = previous_value + input.increment;

        // Store new value
        self.state
            .set(&input.name, new_value.to_string().as_bytes().to_vec(), None)
            .await
            .map_err(|e| pforge_runtime::Error::Handler(format!("State error: {}", e)))?;

        Ok(CounterOutput {
            name: input.name,
            value: new_value,
            previous_value,
            increment: input.increment,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_counter_increment() {
        let state = Arc::new(MemoryStateManager::new());
        let handler = CounterHandler::new(state);

        let input = CounterInput {
            name: "test".to_string(),
            increment: 5,
        };

        let result = handler.handle(input).await.unwrap();
        assert_eq!(result.value, 5);
        assert_eq!(result.previous_value, 0);
        assert_eq!(result.increment, 5);
    }

    #[tokio::test]
    async fn test_counter_persistence() {
        let state = Arc::new(MemoryStateManager::new());
        let handler = CounterHandler::new(state);

        // First increment
        let input1 = CounterInput {
            name: "persistent".to_string(),
            increment: 10,
        };
        let result1 = handler.handle(input1).await.unwrap();
        assert_eq!(result1.value, 10);

        // Second increment
        let input2 = CounterInput {
            name: "persistent".to_string(),
            increment: 5,
        };
        let result2 = handler.handle(input2).await.unwrap();
        assert_eq!(result2.value, 15);
        assert_eq!(result2.previous_value, 10);
    }
}

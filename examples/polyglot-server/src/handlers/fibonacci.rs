use pforge_runtime::{Handler, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FibonacciInput {
    pub n: u32,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct FibonacciOutput {
    pub n: u32,
    pub value: u64,
    pub sequence: Vec<u64>,
    pub language: String,
}

pub struct FibonacciHandler;

#[async_trait::async_trait]
impl Handler for FibonacciHandler {
    type Input = FibonacciInput;
    type Output = FibonacciOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        if input.n > 50 {
            return Err(pforge_runtime::Error::Handler(
                "n must be <= 50 to prevent overflow".to_string(),
            ));
        }

        let sequence = calculate_fibonacci_sequence(input.n);
        let value = sequence.last().copied().unwrap_or(0);

        Ok(FibonacciOutput {
            n: input.n,
            value,
            sequence,
            language: "Rust".to_string(),
        })
    }
}

fn calculate_fibonacci_sequence(n: u32) -> Vec<u64> {
    let mut sequence = Vec::with_capacity((n + 1) as usize);

    if n == 0 {
        sequence.push(0);
        return sequence;
    }

    sequence.push(0);
    sequence.push(1);

    for i in 2..=n {
        let next = sequence[(i - 1) as usize] + sequence[(i - 2) as usize];
        sequence.push(next);
    }

    sequence
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci_sequence() {
        assert_eq!(calculate_fibonacci_sequence(0), vec![0]);
        assert_eq!(calculate_fibonacci_sequence(1), vec![0, 1]);
        assert_eq!(calculate_fibonacci_sequence(5), vec![0, 1, 1, 2, 3, 5]);
        assert_eq!(calculate_fibonacci_sequence(10), vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55]);
    }

    #[tokio::test]
    async fn test_handler() {
        let handler = FibonacciHandler;
        let input = FibonacciInput { n: 10 };

        let result = handler.handle(input).await.unwrap();
        assert_eq!(result.n, 10);
        assert_eq!(result.value, 55);
        assert_eq!(result.language, "Rust");
        assert_eq!(result.sequence.len(), 11);
    }

    #[tokio::test]
    async fn test_validation() {
        let handler = FibonacciHandler;
        let input = FibonacciInput { n: 51 };

        let result = handler.handle(input).await;
        assert!(result.is_err());
    }
}

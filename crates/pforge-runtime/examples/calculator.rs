// Example: Simple Calculator Handler
//
// Run with: cargo run --example calculator

use pforge_runtime::{Handler, HandlerRegistry, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct CalculatorInput {
    operation: String,
    a: f64,
    b: f64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct CalculatorOutput {
    result: f64,
}

struct CalculatorHandler;

#[async_trait::async_trait]
impl Handler for CalculatorHandler {
    type Input = CalculatorInput;
    type Output = CalculatorOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        let result = match input.operation.as_str() {
            "add" => input.a + input.b,
            "subtract" => input.a - input.b,
            "multiply" => input.a * input.b,
            "divide" => {
                if input.b == 0.0 {
                    return Err(pforge_runtime::Error::Handler(
                        "Division by zero".to_string(),
                    ));
                }
                input.a / input.b
            }
            _ => {
                return Err(pforge_runtime::Error::Handler(format!(
                    "Unknown operation: {}",
                    input.operation
                )))
            }
        };

        Ok(CalculatorOutput { result })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧮 Calculator Handler Example\n");

    // Create handler registry
    let mut registry = HandlerRegistry::new();

    // Register calculator handler
    registry.register("calculate", CalculatorHandler);
    println!("✅ Registered 'calculate' handler");

    // Test operations
    let test_cases = vec![
        ("add", 5.0, 3.0),
        ("subtract", 10.0, 4.0),
        ("multiply", 6.0, 7.0),
        ("divide", 20.0, 4.0),
    ];

    println!("\n📊 Running test cases:\n");

    for (op, a, b) in test_cases {
        let input = CalculatorInput {
            operation: op.to_string(),
            a,
            b,
        };

        let input_json = serde_json::to_vec(&input)?;

        match registry.dispatch("calculate", &input_json).await {
            Ok(output) => {
                let result: CalculatorOutput = serde_json::from_slice(&output)?;
                println!("  {} {} {} = {}", a, op, b, result.result);
            }
            Err(e) => {
                println!("  ❌ Error: {}", e);
            }
        }
    }

    // Test error case
    println!("\n🔍 Testing error handling:\n");
    let error_input = CalculatorInput {
        operation: "divide".to_string(),
        a: 10.0,
        b: 0.0,
    };
    let input_json = serde_json::to_vec(&error_input)?;

    match registry.dispatch("calculate", &input_json).await {
        Ok(_) => println!("  ❌ Should have failed!"),
        Err(e) => println!("  ✅ Correctly caught error: {}", e),
    }

    println!("\n✨ Example complete!\n");

    Ok(())
}

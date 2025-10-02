// Handler trait implementations for CLI and HTTP handlers
use crate::handlers::cli::{CliHandler, CliInput, CliOutput};
use crate::handlers::http::{HttpHandler, HttpInput, HttpOutput};
use crate::{Error, Handler, Result};
use async_trait::async_trait;

// CLI Handler Wrapper
#[async_trait]
impl Handler for CliHandler {
    type Input = CliInput;
    type Output = CliOutput;
    type Error = Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        self.execute(input).await
    }
}

// HTTP Handler Wrapper
#[async_trait]
impl Handler for HttpHandler {
    type Input = HttpInput;
    type Output = HttpOutput;
    type Error = Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        self.execute(input).await
    }
}

pub mod cli;
// `http` pulls in reqwest (and a TLS stack) — real weight for a consumer that
// only wants CLI/pipeline handlers, which is the whole point of making it
// optional. Declaring the module unconditionally made the feature optional in
// name only: `--no-default-features` failed to build (paiml/pforge#10).
#[cfg(feature = "http-handlers")]
pub mod http;
pub mod pipeline;
mod wrappers;

pub use cli::CliHandler;
#[cfg(feature = "http-handlers")]
pub use http::HttpHandler;
pub use pipeline::PipelineHandler;

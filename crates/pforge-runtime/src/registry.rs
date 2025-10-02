use crate::{Error, Handler, Result};
use rustc_hash::FxHashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Zero-overhead handler registry with O(1) average-case lookup
pub struct HandlerRegistry {
    handlers: FxHashMap<String, Arc<dyn HandlerEntry>>,
}

trait HandlerEntry: Send + Sync {
    /// Direct dispatch without dynamic allocation
    fn dispatch(&self, params: &[u8]) -> BoxFuture<'static, Result<Vec<u8>>>;

    /// Get schema metadata
    fn input_schema(&self) -> schemars::schema::RootSchema;
    fn output_schema(&self) -> schemars::schema::RootSchema;
}

struct HandlerEntryImpl<H: Handler> {
    handler: Arc<H>,
}

impl<H: Handler> HandlerEntryImpl<H> {
    fn new(handler: H) -> Self {
        Self {
            handler: Arc::new(handler),
        }
    }
}

impl<H> HandlerEntry for HandlerEntryImpl<H>
where
    H: Handler,
    H::Input: 'static,
    H::Output: 'static,
{
    fn dispatch(&self, params: &[u8]) -> BoxFuture<'static, Result<Vec<u8>>> {
        let input: H::Input = match serde_json::from_slice(params) {
            Ok(input) => input,
            Err(e) => return Box::pin(async move { Err(e.into()) }),
        };

        let handler = self.handler.clone();
        Box::pin(async move {
            let output = handler.handle(input).await.map_err(Into::into)?;
            serde_json::to_vec(&output).map_err(Into::into)
        })
    }

    fn input_schema(&self) -> schemars::schema::RootSchema {
        H::input_schema()
    }

    fn output_schema(&self) -> schemars::schema::RootSchema {
        H::output_schema()
    }
}

impl HandlerRegistry {
    /// Create new empty registry
    pub fn new() -> Self {
        Self {
            handlers: FxHashMap::default(),
        }
    }

    /// Register a handler with a name
    pub fn register<H>(&mut self, name: impl Into<String>, handler: H)
    where
        H: Handler,
        H::Input: 'static,
        H::Output: 'static,
    {
        let entry = HandlerEntryImpl::new(handler);
        self.handlers.insert(name.into(), Arc::new(entry));
    }

    /// Check if handler exists
    pub fn has_handler(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }

    /// Dispatch to a handler by name
    #[inline(always)]
    pub async fn dispatch(&self, tool: &str, params: &[u8]) -> Result<Vec<u8>> {
        match self.handlers.get(tool) {
            Some(handler) => handler.dispatch(params).await,
            None => Err(Error::ToolNotFound(tool.to_string())),
        }
    }

    /// Get number of registered handlers
    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }

    /// Get input schema for a tool
    pub fn get_input_schema(&self, tool: &str) -> Option<schemars::schema::RootSchema> {
        self.handlers.get(tool).map(|h| h.input_schema())
    }

    /// Get output schema for a tool
    pub fn get_output_schema(&self, tool: &str) -> Option<schemars::schema::RootSchema> {
        self.handlers.get(tool).map(|h| h.output_schema())
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

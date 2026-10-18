//! An explicit synchronous facade owning its Tokio runtime.
use crate::{Answers, Error, ErrorKind, Provider, Request, Result, ask};
/// A provider plus an owned runtime. Async callers use the provider directly.
pub struct Blocking<P: Provider> {
    provider: P,
    runtime: Option<tokio::runtime::Runtime>,
}
impl<P: Provider> Blocking<P> {
    /// Build a private runtime with timers and I/O enabled.
    ///
    /// # Errors
    /// Returns `Runtime` if the owned Tokio runtime cannot be created.
    pub fn new(provider: P) -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|_| Error::new(ErrorKind::Runtime, "unable to create blocking runtime"))?;
        Ok(Self {
            provider,
            runtime: Some(runtime),
        })
    }
    /// Run one checked request. Reject nesting inside an active Tokio runtime.
    ///
    /// # Errors
    /// Returns `Runtime` inside an active Tokio runtime or if this facade has
    /// shut down. Otherwise propagates request, provider, and response-validation
    /// errors from the checked async call.
    pub fn ask(&self, request: &Request) -> Result<Answers> {
        if tokio::runtime::Handle::try_current().is_ok() {
            return Err(Error::new(
                ErrorKind::Runtime,
                "blocking facade cannot run inside a Tokio runtime",
            ));
        }
        self.runtime
            .as_ref()
            .ok_or_else(|| Error::new(ErrorKind::Runtime, "runtime is shut down"))?
            .block_on(ask(&self.provider, request))
    }
}
impl<P: Provider> Drop for Blocking<P> {
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
}

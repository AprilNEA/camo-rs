#[cfg(feature = "server")]
mod reqwest_impl;
#[cfg(feature = "server")]
pub use reqwest_impl::ReqwestClient;

#[cfg(feature = "worker")]
mod worker_impl;
#[cfg(feature = "worker")]
pub use worker_impl::WorkerFetchClient;

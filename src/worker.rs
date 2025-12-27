use crate::server::{
    config::Config,
    error::CamoError,
    http_client::WorkerFetchClient,
    router::{create_router, AppState},
};
use std::sync::Arc;
use tower_service::Service;
use worker::{event, Context, Env, HttpRequest, Result};

#[event(fetch)]
pub async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();
    Ok(create_router(Arc::new(AppState::from_worker_env(&env)?))
        .call(req)
        .await?)
}

impl Config {
    pub fn from_worker_env(env: &worker::Env) -> Result<Self, CamoError> {
        let key = env.secret("CAMO_KEY").map(|s| s.to_string()).ok();

        if key.is_none() || key.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
            return Err(CamoError::InvalidUrl("CAMO_KEY not set".into()));
        }

        let max_size = env
            .var("CAMO_MAX_SIZE")
            .map(|v| v.to_string().parse().unwrap_or(5 * 1024 * 1024))
            .unwrap_or(5 * 1024 * 1024);

        Ok(Config {
            key,
            listen: "0.0.0.0:8080".to_string(),
            max_size,
            max_redirects: 4,
            timeout: 10,
            allow_video: false,
            allow_audio: false,
            block_private: true,
            metrics: false,
            log_level: "info".to_string(),
        })
    }
}

impl AppState {
    pub fn from_worker_env(env: &worker::Env) -> Result<Self, CamoError> {
        let config = Config::from_worker_env(env)?;
        Ok(AppState::from_config(&config))
    }
}

impl From<CamoError> for worker::Error {
    fn from(err: CamoError) -> Self {
        worker::Error::RustError(err.to_string())
    }
}

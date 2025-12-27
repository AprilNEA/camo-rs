use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum CamoError {
    #[error("invalid digest")]
    InvalidDigest,

    #[error("invalid url encoding")]
    InvalidUrlEncoding,

    #[error("invalid url: {0}")]
    InvalidUrl(String),

    #[error("digest mismatch")]
    DigestMismatch,

    #[error("content type not allowed: {0}")]
    ContentTypeNotAllowed(String),

    #[error("content too large: {0} bytes")]
    ContentTooLarge(u64),

    #[error("too many redirects")]
    TooManyRedirects,

    #[error("request timeout")]
    Timeout,

    #[error("upstream error: {0}")]
    Upstream(#[from] reqwest::Error),

    #[error("private network not allowed")]
    PrivateNetworkNotAllowed,
}

impl IntoResponse for CamoError {
    fn into_response(self) -> Response {
        let status = match &self {
            CamoError::InvalidDigest
            | CamoError::InvalidUrlEncoding
            | CamoError::InvalidUrl(_)
            | CamoError::DigestMismatch => StatusCode::BAD_REQUEST,

            CamoError::ContentTypeNotAllowed(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,

            CamoError::ContentTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,

            CamoError::TooManyRedirects => StatusCode::BAD_GATEWAY,

            CamoError::Timeout => StatusCode::GATEWAY_TIMEOUT,

            CamoError::Upstream(_) => StatusCode::BAD_GATEWAY,

            CamoError::PrivateNetworkNotAllowed => StatusCode::FORBIDDEN,
        };

        (status, self.to_string()).into_response()
    }
}

pub type Result<T> = std::result::Result<T, CamoError>;
